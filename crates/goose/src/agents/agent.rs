use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use futures::stream::BoxStream;
use futures::{stream, FutureExt, Stream, StreamExt, TryStreamExt};
use mcp_core::protocol::JsonRpcMessage;

use crate::agents::final_output_tool::{FINAL_OUTPUT_CONTINUATION_MESSAGE, FINAL_OUTPUT_TOOL_NAME};
use crate::agents::sub_recipe_manager::SubRecipeManager;
use crate::config::{Config, ExtensionConfigManager, PermissionManager};
use crate::message::Message;
use crate::permission::permission_judge::check_tool_permissions;
use crate::permission::PermissionConfirmation;
use crate::providers::base::Provider;
use crate::providers::errors::ProviderError;
use crate::recipe::{Author, Recipe, Response, Settings, SubRecipe};
use crate::scheduler_trait::SchedulerTrait;
use crate::tool_monitor::{ToolCall, ToolMonitor};
use regex::Regex;
use serde_json::Value;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, instrument};

use crate::agents::extension::{ExtensionConfig, ExtensionError, ExtensionResult, ToolInfo};
use crate::agents::extension_manager::{get_parameter_names, ExtensionManager};
use crate::agents::platform_tools::{
    PLATFORM_LIST_RESOURCES_TOOL_NAME, PLATFORM_MANAGE_EXTENSIONS_TOOL_NAME,
    PLATFORM_MANAGE_SCHEDULE_TOOL_NAME, PLATFORM_READ_RESOURCE_TOOL_NAME,
    PLATFORM_SEARCH_AVAILABLE_EXTENSIONS_TOOL_NAME,
};
use crate::agents::prompt_manager::PromptManager;
use crate::agents::router_tool_selector::{
    create_tool_selector, RouterToolSelectionStrategy, RouterToolSelector,
};
use crate::agents::router_tools::{ROUTER_LLM_SEARCH_TOOL_NAME, ROUTER_VECTOR_SEARCH_TOOL_NAME};
use crate::agents::tool_router_index_manager::ToolRouterIndexManager;
use crate::agents::tool_vectordb::generate_table_id;
use crate::agents::types::SessionConfig;
use crate::agents::types::{FrontendTool, ToolResultReceiver};
use mcp_core::{
    prompt::Prompt, protocol::GetPromptResult, tool::Tool, Content, ToolError, ToolResult,
};

use crate::agents::subagent_tools::SUBAGENT_RUN_TASK_TOOL_NAME;

use super::final_output_tool::FinalOutputTool;
use super::platform_tools;
use super::router_tools;
use super::subagent_manager::SubAgentManager;
use super::subagent_tools;
use super::tool_execution::{ToolCallResult, CHAT_MODE_TOOL_SKIPPED_RESPONSE, DECLINED_RESPONSE};

const DEFAULT_MAX_TURNS: u32 = 1000;

/// The main goose Agent
pub struct Agent {
    pub(super) provider: Mutex<Option<Arc<dyn Provider>>>,
    pub(super) extension_manager: RwLock<ExtensionManager>,
    pub(super) sub_recipe_manager: Mutex<SubRecipeManager>,
    pub(super) final_output_tool: Mutex<Option<FinalOutputTool>>,
    pub(super) frontend_tools: Mutex<HashMap<String, FrontendTool>>,
    pub(super) frontend_instructions: Mutex<Option<String>>,
    pub(super) prompt_manager: Mutex<PromptManager>,
    pub(super) confirmation_tx: mpsc::Sender<(String, PermissionConfirmation)>,
    pub(super) confirmation_rx: Mutex<mpsc::Receiver<(String, PermissionConfirmation)>>,
    pub(super) tool_result_tx: mpsc::Sender<(String, ToolResult<Vec<Content>>)>,
    pub(super) tool_result_rx: ToolResultReceiver,
    pub(super) tool_monitor: Mutex<Option<ToolMonitor>>,
    pub(super) router_tool_selector: Mutex<Option<Arc<Box<dyn RouterToolSelector>>>>,
    pub(super) scheduler_service: Mutex<Option<Arc<dyn SchedulerTrait>>>,
    pub(super) subagent_manager: Mutex<Option<SubAgentManager>>,
    pub(super) mcp_notification_rx: Arc<Mutex<mpsc::Receiver<JsonRpcMessage>>>,
}

#[derive(Clone, Debug)]
pub enum AgentEvent {
    Message(Message),
    McpNotification((String, JsonRpcMessage)),
    ModelChange { model: String, mode: String },
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

pub enum ToolStreamItem<T> {
    Message(JsonRpcMessage),
    Result(T),
}

pub type ToolStream = Pin<Box<dyn Stream<Item = ToolStreamItem<ToolResult<Vec<Content>>>> + Send>>;

// tool_stream combines a stream of JsonRpcMessages with a future representing the
// final result of the tool call. MCP notifications are not request-scoped, but
// this lets us capture all notifications emitted during the tool call for
// simpler consumption
pub fn tool_stream<S, F>(rx: S, done: F) -> ToolStream
where
    S: Stream<Item = JsonRpcMessage> + Send + Unpin + 'static,
    F: Future<Output = ToolResult<Vec<Content>>> + Send + 'static,
{
    Box::pin(async_stream::stream! {
        tokio::pin!(done);
        let mut rx = rx;

        loop {
            tokio::select! {
                Some(msg) = rx.next() => {
                    yield ToolStreamItem::Message(msg);
                }
                r = &mut done => {
                    yield ToolStreamItem::Result(r);
                    break;
                }
            }
        }
    })
}

impl Agent {
    pub fn new() -> Self {
        // Create channels with buffer size 32 (adjust if needed)
        let (confirm_tx, confirm_rx) = mpsc::channel(32);
        let (tool_tx, tool_rx) = mpsc::channel(32);
        // Add MCP notification channel
        let (mcp_tx, mcp_rx) = mpsc::channel(100);

        Self {
            provider: Mutex::new(None),
            extension_manager: RwLock::new(ExtensionManager::new()),
            sub_recipe_manager: Mutex::new(SubRecipeManager::new()),
            final_output_tool: Mutex::new(None),
            frontend_tools: Mutex::new(HashMap::new()),
            frontend_instructions: Mutex::new(None),
            prompt_manager: Mutex::new(PromptManager::new()),
            confirmation_tx: confirm_tx,
            confirmation_rx: Mutex::new(confirm_rx),
            tool_result_tx: tool_tx,
            tool_result_rx: Arc::new(Mutex::new(tool_rx)),
            tool_monitor: Mutex::new(None),
            router_tool_selector: Mutex::new(None),
            scheduler_service: Mutex::new(None),
            // Initialize with MCP notification support
            subagent_manager: Mutex::new(Some(SubAgentManager::new(mcp_tx))),
            mcp_notification_rx: Arc::new(Mutex::new(mcp_rx)),
        }
    }

    pub async fn configure_tool_monitor(&self, max_repetitions: Option<u32>) {
        let mut tool_monitor = self.tool_monitor.lock().await;
        *tool_monitor = Some(ToolMonitor::new(max_repetitions));
    }

    pub async fn get_tool_stats(&self) -> Option<HashMap<String, u32>> {
        let tool_monitor = self.tool_monitor.lock().await;
        tool_monitor.as_ref().map(|monitor| monitor.get_stats())
    }

    pub async fn reset_tool_monitor(&self) {
        if let Some(monitor) = self.tool_monitor.lock().await.as_mut() {
            monitor.reset();
        }
    }

    /// Set the scheduler service for this agent
    pub async fn set_scheduler(&self, scheduler: Arc<dyn SchedulerTrait>) {
        let mut scheduler_service = self.scheduler_service.lock().await;
        *scheduler_service = Some(scheduler);
    }

    /// Get a reference count clone to the provider
    pub async fn provider(&self) -> Result<Arc<dyn Provider>, anyhow::Error> {
        match &*self.provider.lock().await {
            Some(provider) => Ok(Arc::clone(provider)),
            None => Err(anyhow!("Provider not set")),
        }
    }

    /// Check if a tool is a frontend tool
    pub async fn is_frontend_tool(&self, name: &str) -> bool {
        self.frontend_tools.lock().await.contains_key(name)
    }

    /// Get a reference to a frontend tool
    pub async fn get_frontend_tool(&self, name: &str) -> Option<FrontendTool> {
        self.frontend_tools.lock().await.get(name).cloned()
    }

    /// Get all tools from all clients with proper prefixing
    pub async fn get_prefixed_tools(&self) -> ExtensionResult<Vec<Tool>> {
        let mut tools = self
            .extension_manager
            .read()
            .await
            .get_prefixed_tools(None)
            .await?;

        // Add frontend tools directly - they don't need prefixing since they're already uniquely named
        let frontend_tools = self.frontend_tools.lock().await;
        for frontend_tool in frontend_tools.values() {
            tools.push(frontend_tool.tool.clone());
        }

        Ok(tools)
    }

    pub async fn add_final_output_tool(&self, response: Response) {
        let mut final_output_tool = self.final_output_tool.lock().await;
        let created_final_output_tool = FinalOutputTool::new(response);
        let final_output_system_prompt = created_final_output_tool.system_prompt();
        *final_output_tool = Some(created_final_output_tool);
        self.extend_system_prompt(final_output_system_prompt).await;
    }

    pub async fn add_sub_recipes(&self, sub_recipes: Vec<SubRecipe>) {
        let mut sub_recipe_manager = self.sub_recipe_manager.lock().await;
        sub_recipe_manager.add_sub_recipe_tools(sub_recipes);
    }

    /// Dispatch a single tool call to the appropriate client
    #[instrument(skip(self, tool_call, request_id), fields(input, output))]
    pub async fn dispatch_tool_call(
        &self,
        tool_call: mcp_core::tool::ToolCall,
        request_id: String,
    ) -> (String, Result<ToolCallResult, ToolError>) {
        // Check if this tool call should be allowed based on repetition monitoring
        if let Some(monitor) = self.tool_monitor.lock().await.as_mut() {
            let tool_call_info = ToolCall::new(tool_call.name.clone(), tool_call.arguments.clone());

            if !monitor.check_tool_call(tool_call_info) {
                return (
                    request_id,
                    Err(ToolError::ExecutionError(
                        "Tool call rejected: exceeded maximum allowed repetitions".to_string(),
                    )),
                );
            }
        }

        if tool_call.name == PLATFORM_MANAGE_SCHEDULE_TOOL_NAME {
            let result = self
                .handle_schedule_management(tool_call.arguments, request_id.clone())
                .await;
            return (request_id, Ok(ToolCallResult::from(result)));
        }

        if tool_call.name == PLATFORM_MANAGE_EXTENSIONS_TOOL_NAME {
            let extension_name = tool_call
                .arguments
                .get("extension_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let action = tool_call
                .arguments
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let (request_id, result) = self
                .manage_extensions(action, extension_name, request_id)
                .await;

            return (request_id, Ok(ToolCallResult::from(result)));
        }

        if tool_call.name == FINAL_OUTPUT_TOOL_NAME {
            if let Some(final_output_tool) = self.final_output_tool.lock().await.as_mut() {
                let result = final_output_tool.execute_tool_call(tool_call.clone()).await;
                return (request_id, Ok(result));
            } else {
                return (
                    request_id,
                    Err(ToolError::ExecutionError(
                        "Final output tool not defined".to_string(),
                    )),
                );
            }
        }

        let extension_manager = self.extension_manager.read().await;
        let sub_recipe_manager = self.sub_recipe_manager.lock().await;

        let result: ToolCallResult = if sub_recipe_manager.is_sub_recipe_tool(&tool_call.name) {
            sub_recipe_manager
                .dispatch_sub_recipe_tool_call(&tool_call.name, tool_call.arguments.clone())
                .await
        } else if tool_call.name == PLATFORM_READ_RESOURCE_TOOL_NAME {
            // Check if the tool is read_resource and handle it separately
            ToolCallResult::from(
                extension_manager
                    .read_resource(tool_call.arguments.clone())
                    .await,
            )
        } else if tool_call.name == PLATFORM_LIST_RESOURCES_TOOL_NAME {
            ToolCallResult::from(
                extension_manager
                    .list_resources(tool_call.arguments.clone())
                    .await,
            )
        } else if tool_call.name == PLATFORM_SEARCH_AVAILABLE_EXTENSIONS_TOOL_NAME {
            ToolCallResult::from(extension_manager.search_available_extensions().await)
        } else if tool_call.name == SUBAGENT_RUN_TASK_TOOL_NAME {
            ToolCallResult::from(
                self.handle_run_subagent_task(tool_call.arguments.clone())
                    .await,
            )
        } else if self.is_frontend_tool(&tool_call.name).await {
            // For frontend tools, return an error indicating we need frontend execution
            ToolCallResult::from(Err(ToolError::ExecutionError(
                "Frontend tool execution required".to_string(),
            )))
        } else if tool_call.name == ROUTER_VECTOR_SEARCH_TOOL_NAME
            || tool_call.name == ROUTER_LLM_SEARCH_TOOL_NAME
        {
            let selector = self.router_tool_selector.lock().await.clone();
            let selected_tools = match selector.as_ref() {
                Some(selector) => match selector.select_tools(tool_call.arguments.clone()).await {
                    Ok(tools) => tools,
                    Err(e) => {
                        return (
                            request_id,
                            Err(ToolError::ExecutionError(format!(
                                "Failed to select tools: {}",
                                e
                            ))),
                        )
                    }
                },
                None => {
                    return (
                        request_id,
                        Err(ToolError::ExecutionError(
                            "No tool selector available".to_string(),
                        )),
                    )
                }
            };
            ToolCallResult::from(Ok(selected_tools))
        } else {
            // Clone the result to ensure no references to extension_manager are returned
            let result = extension_manager
                .dispatch_tool_call(tool_call.clone())
                .await;
            match result {
                Ok(call_result) => call_result,
                Err(e) => ToolCallResult::from(Err(ToolError::ExecutionError(e.to_string()))),
            }
        };

        (
            request_id,
            Ok(ToolCallResult {
                notification_stream: result.notification_stream,
                result: Box::new(
                    result
                        .result
                        .map(super::large_response_handler::process_tool_response),
                ),
            }),
        )
    }

    pub(super) async fn manage_extensions(
        &self,
        action: String,
        extension_name: String,
        request_id: String,
    ) -> (String, Result<Vec<Content>, ToolError>) {
        let mut extension_manager = self.extension_manager.write().await;

        let selector = self.router_tool_selector.lock().await.clone();
        if ToolRouterIndexManager::is_tool_router_enabled(&selector) {
            if let Some(selector) = selector {
                let selector_action = if action == "disable" { "remove" } else { "add" };
                let extension_manager = self.extension_manager.read().await;
                let selector = Arc::new(selector);
                if let Err(e) = ToolRouterIndexManager::update_extension_tools(
                    &selector,
                    &extension_manager,
                    &extension_name,
                    selector_action,
                )
                .await
                {
                    return (
                        request_id,
                        Err(ToolError::ExecutionError(format!(
                            "Failed to update vector index: {}",
                            e
                        ))),
                    );
                }
            }
        }

        if action == "disable" {
            let result = extension_manager
                .remove_extension(&extension_name)
                .await
                .map(|_| {
                    vec![Content::text(format!(
                        "The extension '{}' has been disabled successfully",
                        extension_name
                    ))]
                })
                .map_err(|e| ToolError::ExecutionError(e.to_string()));
            return (request_id, result);
        }

        let config = match ExtensionConfigManager::get_config_by_name(&extension_name) {
            Ok(Some(config)) => config,
            Ok(None) => {
                return (
                    request_id,
                    Err(ToolError::ExecutionError(format!(
                        "Extension '{}' not found. Please check the extension name and try again.",
                        extension_name
                    ))),
                )
            }
            Err(e) => {
                return (
                    request_id,
                    Err(ToolError::ExecutionError(format!(
                        "Failed to get extension config: {}",
                        e
                    ))),
                )
            }
        };

        let result = extension_manager
            .add_extension(config)
            .await
            .map(|_| {
                vec![Content::text(format!(
                    "The extension '{}' has been installed successfully",
                    extension_name
                ))]
            })
            .map_err(|e| ToolError::ExecutionError(e.to_string()));

        // Update vector index if operation was successful and vector routing is enabled
        if result.is_ok() {
            let selector = self.router_tool_selector.lock().await.clone();
            if ToolRouterIndexManager::is_tool_router_enabled(&selector) {
                if let Some(selector) = selector {
                    let vector_action = if action == "disable" { "remove" } else { "add" };
                    let extension_manager = self.extension_manager.read().await;
                    let selector = Arc::new(selector);
                    if let Err(e) = ToolRouterIndexManager::update_extension_tools(
                        &selector,
                        &extension_manager,
                        &extension_name,
                        vector_action,
                    )
                    .await
                    {
                        return (
                            request_id,
                            Err(ToolError::ExecutionError(format!(
                                "Failed to update vector index: {}",
                                e
                            ))),
                        );
                    }
                }
            }
        }
        (request_id, result)
    }

    pub async fn add_extension(&self, extension: ExtensionConfig) -> ExtensionResult<()> {
        match &extension {
            ExtensionConfig::Frontend {
                name: _,
                tools,
                instructions,
                bundled: _,
            } => {
                // For frontend tools, just store them in the frontend_tools map
                let mut frontend_tools = self.frontend_tools.lock().await;
                for tool in tools {
                    let frontend_tool = FrontendTool {
                        name: tool.name.clone(),
                        tool: tool.clone(),
                    };
                    frontend_tools.insert(tool.name.clone(), frontend_tool);
                }
                // Store instructions if provided, using "frontend" as the key
                let mut frontend_instructions = self.frontend_instructions.lock().await;
                if let Some(instructions) = instructions {
                    *frontend_instructions = Some(instructions.clone());
                } else {
                    // Default frontend instructions if none provided
                    *frontend_instructions = Some(
                        "The following tools are provided directly by the frontend and will be executed by the frontend when called.".to_string(),
                    );
                }
            }
            _ => {
                let mut extension_manager = self.extension_manager.write().await;
                extension_manager.add_extension(extension.clone()).await?;
            }
        }

        // If vector tool selection is enabled, index the tools
        let selector = self.router_tool_selector.lock().await.clone();
        if ToolRouterIndexManager::is_tool_router_enabled(&selector) {
            if let Some(selector) = selector {
                let extension_manager = self.extension_manager.read().await;
                let selector = Arc::new(selector);
                if let Err(e) = ToolRouterIndexManager::update_extension_tools(
                    &selector,
                    &extension_manager,
                    &extension.name(),
                    "add",
                )
                .await
                {
                    return Err(ExtensionError::SetupError(format!(
                        "Failed to index tools for extension {}: {}",
                        extension.name(),
                        e
                    )));
                }
            }
        }

        Ok(())
    }

    pub async fn list_tools(&self, extension_name: Option<String>) -> Vec<Tool> {
        let extension_manager = self.extension_manager.read().await;
        let mut prefixed_tools = extension_manager
            .get_prefixed_tools(extension_name.clone())
            .await
            .unwrap_or_default();

        if extension_name.is_none() || extension_name.as_deref() == Some("platform") {
            // Add platform tools
            prefixed_tools.extend([
                platform_tools::search_available_extensions_tool(),
                platform_tools::manage_extensions_tool(),
                platform_tools::manage_schedule_tool(),
            ]);

            // Add subagent tool (only if ALPHA_FEATURES is enabled)
            let config = Config::global();
            if config.get_param::<bool>("ALPHA_FEATURES").unwrap_or(false) {
                prefixed_tools.push(subagent_tools::run_task_subagent_tool());
            }

            // Add resource tools if supported
            if extension_manager.supports_resources() {
                prefixed_tools.extend([
                    platform_tools::read_resource_tool(),
                    platform_tools::list_resources_tool(),
                ]);
            }
        }

        if extension_name.is_none() {
            let sub_recipe_manager = self.sub_recipe_manager.lock().await;
            prefixed_tools.extend(sub_recipe_manager.sub_recipe_tools.values().cloned());

            if let Some(final_output_tool) = self.final_output_tool.lock().await.as_ref() {
                prefixed_tools.push(final_output_tool.tool());
            }
        }

        prefixed_tools
    }

    pub async fn list_tools_for_router(
        &self,
        strategy: Option<RouterToolSelectionStrategy>,
    ) -> Vec<Tool> {
        let mut prefixed_tools = vec![];
        match strategy {
            Some(RouterToolSelectionStrategy::Vector) => {
                prefixed_tools.push(router_tools::vector_search_tool());
            }
            Some(RouterToolSelectionStrategy::Llm) => {
                prefixed_tools.push(router_tools::llm_search_tool());
            }
            None => {}
        }

        // Get recent tool calls from router tool selector if available
        let selector = self.router_tool_selector.lock().await.clone();
        if let Some(selector) = selector {
            if let Ok(recent_calls) = selector.get_recent_tool_calls(20).await {
                let extension_manager = self.extension_manager.read().await;
                // Add recent tool calls to the list, avoiding duplicates
                for tool_name in recent_calls {
                    // Find the tool in the extension manager's tools
                    if let Ok(extension_tools) = extension_manager.get_prefixed_tools(None).await {
                        if let Some(tool) = extension_tools.iter().find(|t| t.name == tool_name) {
                            // Only add if not already in prefixed_tools
                            if !prefixed_tools.iter().any(|t| t.name == tool.name) {
                                prefixed_tools.push(tool.clone());
                            }
                        }
                    }
                }
            }
        }

        prefixed_tools
    }

    pub async fn remove_extension(&self, name: &str) -> Result<()> {
        let mut extension_manager = self.extension_manager.write().await;
        extension_manager.remove_extension(name).await?;

        // If vector tool selection is enabled, remove tools from the index
        let selector = self.router_tool_selector.lock().await.clone();
        if ToolRouterIndexManager::is_tool_router_enabled(&selector) {
            if let Some(selector) = selector {
                let extension_manager = self.extension_manager.read().await;
                ToolRouterIndexManager::update_extension_tools(
                    &selector,
                    &extension_manager,
                    name,
                    "remove",
                )
                .await?;
            }
        }

        Ok(())
    }

    pub async fn list_extensions(&self) -> Vec<String> {
        let extension_manager = self.extension_manager.read().await;
        extension_manager
            .list_extensions()
            .await
            .expect("Failed to list extensions")
    }

    /// Handle a confirmation response for a tool request
    pub async fn handle_confirmation(
        &self,
        request_id: String,
        confirmation: PermissionConfirmation,
    ) {
        if let Err(e) = self.confirmation_tx.send((request_id, confirmation)).await {
            error!("Failed to send confirmation: {}", e);
        }
    }

    #[instrument(skip(self, messages, session), fields(user_message))]
    pub async fn reply(
        &self,
        messages: &[Message],
        session: Option<SessionConfig>,
    ) -> anyhow::Result<BoxStream<'_, anyhow::Result<AgentEvent>>> {
        let mut messages = messages.to_vec();
        let reply_span = tracing::Span::current();

        // Load settings from config
        let config = Config::global();

        // Setup tools and prompt
        let (mut tools, mut toolshim_tools, mut system_prompt) =
            self.prepare_tools_and_prompt().await?;

        // Get goose_mode from config, but override with execution_mode if provided in session config
        let mut goose_mode = config.get_param("GOOSE_MODE").unwrap_or("auto".to_string());

        // If this is a scheduled job with an execution_mode, override the goose_mode
        if let Some(session_config) = &session {
            if let Some(execution_mode) = &session_config.execution_mode {
                // Map "foreground" to "auto" and "background" to "chat"
                goose_mode = match execution_mode.as_str() {
                    "foreground" => "auto".to_string(),
                    "background" => "chat".to_string(),
                    _ => goose_mode,
                };
                tracing::info!(
                    "Using execution_mode '{}' which maps to goose_mode '{}'",
                    execution_mode,
                    goose_mode
                );
            }
        }

        let (tools_with_readonly_annotation, tools_without_annotation) =
            Self::categorize_tools_by_annotation(&tools);

        if let Some(content) = messages
            .last()
            .and_then(|msg| msg.content.first())
            .and_then(|c| c.as_text())
        {
            debug!("user_message" = &content);
        }

        Ok(Box::pin(async_stream::try_stream! {
            let _ = reply_span.enter();
            let mut turns_taken = 0u32;
            let max_turns = session
                .as_ref()
                .and_then(|s| s.max_turns)
                .unwrap_or_else(|| {
                    config.get_param("GOOSE_MAX_TURNS").unwrap_or(DEFAULT_MAX_TURNS)
                });

            loop {
                turns_taken += 1;
                if turns_taken > max_turns {
                    yield AgentEvent::Message(Message::assistant().with_text(
                        "I've reached the maximum number of actions I can do without user input. Would you like me to continue?"
                    ));
                    break;
                }

                // Check for MCP notifications from subagents
                let mcp_notifications = self.get_mcp_notifications().await;
                for notification in mcp_notifications {
                    // Extract subagent info from the notification data
                    if let JsonRpcMessage::Notification(ref notif) = notification {
                        if let Some(params) = &notif.params {
                            if let Some(data) = params.get("data") {
                                if let (Some(subagent_id), Some(_message)) = (
                                    data.get("subagent_id").and_then(|v| v.as_str()),
                                    data.get("message").and_then(|v| v.as_str())
                                ) {
                                    // Emit as McpNotification event
                                    yield AgentEvent::McpNotification((
                                        subagent_id.to_string(),
                                        notification.clone()
                                    ));
                                }
                            }
                        }
                    }
                }

                match Self::generate_response_from_provider(
                    self.provider().await?,
                    &system_prompt,
                    &messages,
                    &tools,
                    &toolshim_tools,
                ).await {
                    Ok((response, usage)) => {
                        // Emit model change event if provider is lead-worker
                        let provider = self.provider().await?;
                        if let Some(lead_worker) = provider.as_lead_worker() {
                            // The actual model used is in the usage
                            let active_model = usage.model.clone();
                            let (lead_model, worker_model) = lead_worker.get_model_info();
                            let mode = if active_model == lead_model {
                                "lead"
                            } else if active_model == worker_model {
                                "worker"
                            } else {
                                "unknown"
                            };

                            yield AgentEvent::ModelChange {
                                model: active_model,
                                mode: mode.to_string(),
                            };
                        }

                        // record usage for the session in the session file
                        if let Some(session_config) = session.clone() {
                            Self::update_session_metrics(session_config, &usage, messages.len()).await?;
                        }

                        // categorize the type of requests we need to handle
                        let (frontend_requests,
                            remaining_requests,
                            filtered_response) =
                            self.categorize_tool_requests(&response).await;

                        // Record tool calls in the router selector
                        let selector = self.router_tool_selector.lock().await.clone();
                        if let Some(selector) = selector {
                            // Record frontend tool calls
                            for request in &frontend_requests {
                                if let Ok(tool_call) = &request.tool_call {
                                    if let Err(e) = selector.record_tool_call(&tool_call.name).await {
                                        tracing::error!("Failed to record frontend tool call: {}", e);
                                    }
                                }
                            }
                            // Record remaining tool calls
                            for request in &remaining_requests {
                                if let Ok(tool_call) = &request.tool_call {
                                    if let Err(e) = selector.record_tool_call(&tool_call.name).await {
                                        tracing::error!("Failed to record tool call: {}", e);
                                    }
                                }
                            }
                        }
                        // Yield the assistant's response with frontend tool requests filtered out
                        yield AgentEvent::Message(filtered_response.clone());

                        tokio::task::yield_now().await;

                        let num_tool_requests = frontend_requests.len() + remaining_requests.len();
                        if num_tool_requests == 0 {
                            if let Some(final_output_tool) = self.final_output_tool.lock().await.as_ref() {
                                if final_output_tool.final_output.is_none() {
                                    tracing::warn!("Final output tool has not been called yet. Continuing agent loop.");
                                    yield AgentEvent::Message(Message::user().with_text(FINAL_OUTPUT_CONTINUATION_MESSAGE));
                                    continue;
                                } else {
                                    yield AgentEvent::Message(Message::assistant().with_text(final_output_tool.final_output.clone().unwrap()));
                                }
                            }
                            break;
                        }

                        // Process tool requests depending on frontend tools and then goose_mode
                        let message_tool_response = Arc::new(Mutex::new(Message::user()));

                        // First handle any frontend tool requests
                        let mut frontend_tool_stream = self.handle_frontend_tool_requests(
                            &frontend_requests,
                            message_tool_response.clone()
                        );

                        // we have a stream of frontend tools to handle, inside the stream
                        // execution is yeield back to this reply loop, and is of the same Message
                        // type, so we can yield that back up to be handled
                        while let Some(msg) = frontend_tool_stream.try_next().await? {
                            yield AgentEvent::Message(msg);
                        }

                        // Clone goose_mode once before the match to avoid move issues
                        let mode = goose_mode.clone();
                        if mode.as_str() == "chat" {
                            // Skip all tool calls in chat mode
                            for request in remaining_requests {
                                let mut response = message_tool_response.lock().await;
                                *response = response.clone().with_tool_response(
                                    request.id.clone(),
                                    Ok(vec![Content::text(CHAT_MODE_TOOL_SKIPPED_RESPONSE)]),
                                );
                            }
                        } else {
                            // At this point, we have handled the frontend tool requests and know goose_mode != "chat"
                            // What remains is handling the remaining tool requests (enable extension,
                            // regular tool calls) in goose_mode == ["auto", "approve" or "smart_approve"]
                            let mut permission_manager = PermissionManager::default();
                            let (permission_check_result, enable_extension_request_ids) = check_tool_permissions(
                                &remaining_requests,
                                &mode,
                                tools_with_readonly_annotation.clone(),
                                tools_without_annotation.clone(),
                                &mut permission_manager,
                                self.provider().await?).await;

                            // Handle pre-approved and read-only tools in parallel
                            let mut tool_futures: Vec<(String, ToolStream)> = Vec::new();

                            // Skip the confirmation for approved tools
                            for request in &permission_check_result.approved {
                                if let Ok(tool_call) = request.tool_call.clone() {
                                    let (req_id, tool_result) = self.dispatch_tool_call(tool_call, request.id.clone()).await;

                                    tool_futures.push((req_id, match tool_result {
                                        Ok(result) => tool_stream(
                                            result.notification_stream.unwrap_or_else(|| Box::new(stream::empty())),
                                            result.result,
                                        ),
                                        Err(e) => tool_stream(
                                            Box::new(stream::empty()),
                                            futures::future::ready(Err(e)),
                                        ),
                                    }));
                                }
                            }

                            for request in &permission_check_result.denied {
                                let mut response = message_tool_response.lock().await;
                                *response = response.clone().with_tool_response(
                                    request.id.clone(),
                                    Ok(vec![Content::text(DECLINED_RESPONSE)]),
                                );
                            }

                            // We need interior mutability in handle_approval_tool_requests
                            let tool_futures_arc = Arc::new(Mutex::new(tool_futures));

                            // Process tools requiring approval (enable extension, regular tool calls)
                            let mut tool_approval_stream = self.handle_approval_tool_requests(
                                &permission_check_result.needs_approval,
                                tool_futures_arc.clone(),
                                &mut permission_manager,
                                message_tool_response.clone()
                            );

                            // We have a stream of tool_approval_requests to handle
                            // Execution is yielded back to this reply loop, and is of the same Message
                            // type, so we can yield the Message back up to be handled and grab any
                            // confirmations or denials
                            while let Some(msg) = tool_approval_stream.try_next().await? {
                                yield AgentEvent::Message(msg);
                            }

                            tool_futures = {
                                // Lock the mutex asynchronously
                                let mut futures_lock = tool_futures_arc.lock().await;
                                // Drain the vector and collect into a new Vec
                                futures_lock.drain(..).collect::<Vec<_>>()
                            };

                            let with_id = tool_futures
                                .into_iter()
                                .map(|(request_id, stream)| {
                                    stream.map(move |item| (request_id.clone(), item))
                                })
                                .collect::<Vec<_>>();

                            let mut combined = stream::select_all(with_id);

                            let mut all_install_successful = true;

                            while let Some((request_id, item)) = combined.next().await {
                                match item {
                                    ToolStreamItem::Result(output) => {
                                        if enable_extension_request_ids.contains(&request_id) && output.is_err(){
                                            all_install_successful = false;
                                        }
                                        let mut response = message_tool_response.lock().await;
                                        *response = response.clone().with_tool_response(request_id, output);
                                    },
                                    ToolStreamItem::Message(msg) => {
                                        yield AgentEvent::McpNotification((request_id, msg))
                                    }
                                }
                            }

                            // Update system prompt and tools if installations were successful
                            if all_install_successful {
                                (tools, toolshim_tools, system_prompt) = self.prepare_tools_and_prompt().await?;
                            }
                        }

                        let final_message_tool_resp = message_tool_response.lock().await.clone();
                        yield AgentEvent::Message(final_message_tool_resp.clone());

                        messages.push(response);
                        messages.push(final_message_tool_resp);

                        // Check for MCP notifications from subagents again before next iteration
                        // Note: These are already handled as McpNotification events above,
                        // so we don't need to convert them to assistant messages here.
                        // This was causing duplicate plain-text notifications.
                        // let mcp_notifications = self.get_mcp_notifications().await;
                        // for notification in mcp_notifications {
                        //     // Extract subagent info from the notification data for assistant messages
                        //     if let JsonRpcMessage::Notification(ref notif) = notification {
                        //         if let Some(params) = &notif.params {
                        //             if let Some(data) = params.get("data") {
                        //                 if let (Some(subagent_id), Some(message)) = (
                        //                     data.get("subagent_id").and_then(|v| v.as_str()),
                        //                     data.get("message").and_then(|v| v.as_str())
                        //                 ) {
                        //                     yield AgentEvent::Message(
                        //                         Message::assistant().with_text(
                        //                             format!("Subagent {}: {}", subagent_id, message)
                        //                         )
                        //                     );
                        //                 }
                        //             }
                        //         }
                        //     }
                        // }
                    },
                    Err(ProviderError::ContextLengthExceeded(_)) => {
                        // At this point, the last message should be a user message
                        // because call to provider led to context length exceeded error
                        // Immediately yield a special message and break
                        yield AgentEvent::Message(Message::assistant().with_context_length_exceeded(
                            "The context length of the model has been exceeded. Please start a new session and try again.",
                        ));
                        break;
                    },
                    Err(e) => {
                        // Create an error message & terminate the stream
                        error!("Error: {}", e);
                        yield AgentEvent::Message(Message::assistant().with_text(format!("Ran into this error: {e}.\n\nPlease retry if you think this is a transient or recoverable error.")));
                        break;
                    }
                }

                // Yield control back to the scheduler to prevent blocking
                tokio::task::yield_now().await;
            }
        }))
    }

    /// Extend the system prompt with one line of additional instruction
    pub async fn extend_system_prompt(&self, instruction: String) {
        let mut prompt_manager = self.prompt_manager.lock().await;
        prompt_manager.add_system_prompt_extra(instruction);
    }

    /// Get MCP notifications from subagents
    pub async fn get_mcp_notifications(&self) -> Vec<JsonRpcMessage> {
        let mut notifications = Vec::new();
        let mut rx = self.mcp_notification_rx.lock().await;

        while let Ok(notification) = rx.try_recv() {
            notifications.push(notification);
        }

        notifications
    }

    /// Update the provider
    pub async fn update_provider(&self, provider: Arc<dyn Provider>) -> Result<()> {
        let mut current_provider = self.provider.lock().await;
        *current_provider = Some(provider.clone());

        // Initialize subagent manager with MCP notification support
        // Need to recreate the MCP channel since we're replacing the manager
        let (mcp_tx, mcp_rx) = mpsc::channel(100);
        {
            let mut rx_guard = self.mcp_notification_rx.lock().await;
            *rx_guard = mcp_rx;
        }
        *self.subagent_manager.lock().await = Some(SubAgentManager::new(mcp_tx));

        self.update_router_tool_selector(Some(provider), None)
            .await?;
        Ok(())
    }

    pub async fn update_router_tool_selector(
        &self,
        provider: Option<Arc<dyn Provider>>,
        reindex_all: Option<bool>,
    ) -> Result<()> {
        let config = Config::global();
        let _extension_manager = self.extension_manager.read().await;
        let provider = match provider {
            Some(p) => p,
            None => self.provider().await?,
        };

        let router_tool_selection_strategy = config
            .get_param("GOOSE_ROUTER_TOOL_SELECTION_STRATEGY")
            .unwrap_or_else(|_| "default".to_string());

        let strategy = match router_tool_selection_strategy.to_lowercase().as_str() {
            "vector" => Some(RouterToolSelectionStrategy::Vector),
            "llm" => Some(RouterToolSelectionStrategy::Llm),
            _ => None,
        };

        let selector = match strategy {
            Some(RouterToolSelectionStrategy::Vector) => {
                let table_name = generate_table_id();
                let selector = create_tool_selector(strategy, provider.clone(), Some(table_name))
                    .await
                    .map_err(|e| anyhow!("Failed to create tool selector: {}", e))?;
                Arc::new(selector)
            }
            Some(RouterToolSelectionStrategy::Llm) => {
                let selector = create_tool_selector(strategy, provider.clone(), None)
                    .await
                    .map_err(|e| anyhow!("Failed to create tool selector: {}", e))?;
                Arc::new(selector)
            }
            None => return Ok(()),
        };

        // First index platform tools
        let extension_manager = self.extension_manager.read().await;
        ToolRouterIndexManager::index_platform_tools(&selector, &extension_manager).await?;

        if reindex_all.unwrap_or(false) {
            let enabled_extensions = extension_manager.list_extensions().await?;
            for extension_name in enabled_extensions {
                if let Err(e) = ToolRouterIndexManager::update_extension_tools(
                    &selector,
                    &extension_manager,
                    &extension_name,
                    "add",
                )
                .await
                {
                    tracing::error!(
                        "Failed to index tools for extension {}: {}",
                        extension_name,
                        e
                    );
                }
            }
        }

        // Update the selector
        *self.router_tool_selector.lock().await = Some(selector.clone());

        Ok(())
    }

    /// Override the system prompt with a custom template
    pub async fn override_system_prompt(&self, template: String) {
        let mut prompt_manager = self.prompt_manager.lock().await;
        prompt_manager.set_system_prompt_override(template);
    }

    pub async fn list_extension_prompts(&self) -> HashMap<String, Vec<Prompt>> {
        let extension_manager = self.extension_manager.read().await;
        extension_manager
            .list_prompts()
            .await
            .expect("Failed to list prompts")
    }

    pub async fn get_prompt(&self, name: &str, arguments: Value) -> Result<GetPromptResult> {
        let extension_manager = self.extension_manager.read().await;

        // First find which extension has this prompt
        let prompts = extension_manager
            .list_prompts()
            .await
            .map_err(|e| anyhow!("Failed to list prompts: {}", e))?;

        if let Some(extension) = prompts
            .iter()
            .find(|(_, prompt_list)| prompt_list.iter().any(|p| p.name == name))
            .map(|(extension, _)| extension)
        {
            return extension_manager
                .get_prompt(extension, name, arguments)
                .await
                .map_err(|e| anyhow!("Failed to get prompt: {}", e));
        }

        Err(anyhow!("Prompt '{}' not found", name))
    }

    pub async fn get_plan_prompt(&self) -> anyhow::Result<String> {
        let extension_manager = self.extension_manager.read().await;
        let tools = extension_manager.get_prefixed_tools(None).await?;
        let tools_info = tools
            .into_iter()
            .map(|tool| {
                ToolInfo::new(
                    &tool.name,
                    &tool.description,
                    get_parameter_names(&tool),
                    None,
                )
            })
            .collect();

        let plan_prompt = extension_manager.get_planning_prompt(tools_info).await;

        Ok(plan_prompt)
    }

    pub async fn handle_tool_result(&self, id: String, result: ToolResult<Vec<Content>>) {
        if let Err(e) = self.tool_result_tx.send((id, result)).await {
            tracing::error!("Failed to send tool result: {}", e);
        }
    }

    pub async fn create_recipe(&self, mut messages: Vec<Message>) -> Result<Recipe> {
        let extension_manager = self.extension_manager.read().await;
        let extensions_info = extension_manager.get_extensions_info().await;

        // Get model name from provider
        let provider = self.provider().await?;
        let model_config = provider.get_model_config();
        let model_name = &model_config.model_name;

        let prompt_manager = self.prompt_manager.lock().await;
        let system_prompt = prompt_manager.build_system_prompt(
            extensions_info,
            self.frontend_instructions.lock().await.clone(),
            extension_manager.suggest_disable_extensions_prompt().await,
            Some(model_name),
            None,
        );

        let recipe_prompt = prompt_manager.get_recipe_prompt().await;
        let tools = extension_manager.get_prefixed_tools(None).await?;

        messages.push(Message::user().with_text(recipe_prompt));

        let (result, _usage) = self
            .provider
            .lock()
            .await
            .as_ref()
            .unwrap()
            .complete(&system_prompt, &messages, &tools)
            .await?;

        let content = result.as_concat_text();

        // the response may be contained in ```json ```, strip that before parsing json
        let re = Regex::new(r"(?s)```[^\n]*\n(.*?)\n```").unwrap();
        let clean_content = re
            .captures(&content)
            .and_then(|caps| caps.get(1).map(|m| m.as_str()))
            .unwrap_or(&content)
            .trim()
            .to_string();

        // try to parse json response from the LLM
        let (instructions, activities) =
            if let Ok(json_content) = serde_json::from_str::<Value>(&clean_content) {
                let instructions = json_content
                    .get("instructions")
                    .ok_or_else(|| anyhow!("Missing 'instructions' in json response"))?
                    .as_str()
                    .ok_or_else(|| anyhow!("instructions' is not a string"))?
                    .to_string();

                let activities = json_content
                    .get("activities")
                    .ok_or_else(|| anyhow!("Missing 'activities' in json response"))?
                    .as_array()
                    .ok_or_else(|| anyhow!("'activities' is not an array'"))?
                    .iter()
                    .map(|act| {
                        act.as_str()
                            .map(|s| s.to_string())
                            .ok_or(anyhow!("'activities' array element is not a string"))
                    })
                    .collect::<Result<_, _>>()?;

                (instructions, activities)
            } else {
                // If we can't get valid JSON, try string parsing
                // Use split_once to get the content after "Instructions:".
                let after_instructions = content
                    .split_once("instructions:")
                    .map(|(_, rest)| rest)
                    .unwrap_or(&content);

                // Split once more to separate instructions from activities.
                let (instructions_part, activities_text) = after_instructions
                    .split_once("activities:")
                    .unwrap_or((after_instructions, ""));

                let instructions = instructions_part
                    .trim_end_matches(|c: char| c.is_whitespace() || c == '#')
                    .trim()
                    .to_string();
                let activities_text = activities_text.trim();

                // Regex to remove bullet markers or numbers with an optional dot.
                let bullet_re = Regex::new(r"^[•\-\*\d]+\.?\s*").expect("Invalid regex");

                // Process each line in the activities section.
                let activities: Vec<String> = activities_text
                    .lines()
                    .map(|line| bullet_re.replace(line, "").to_string())
                    .map(|s| s.trim().to_string())
                    .filter(|line| !line.is_empty())
                    .collect();

                (instructions, activities)
            };

        let extensions = ExtensionConfigManager::get_all().unwrap_or_default();
        let extension_configs: Vec<_> = extensions
            .iter()
            .filter(|e| e.enabled)
            .map(|e| e.config.clone())
            .collect();

        let author = Author {
            contact: std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .ok(),
            metadata: None,
        };

        // Ideally we'd get the name of the provider we are using from the provider itself
        // but it doesn't know and the plumbing looks complicated.
        let config = Config::global();
        let provider_name: String = config
            .get_param("GOOSE_PROVIDER")
            .expect("No provider configured. Run 'goose configure' first");

        let settings = Settings {
            goose_provider: Some(provider_name.clone()),
            goose_model: Some(model_name.clone()),
            temperature: Some(model_config.temperature.unwrap_or(0.0)),
        };

        let recipe = Recipe::builder()
            .title("Custom recipe from chat")
            .description("a custom recipe instance from this chat session")
            .instructions(instructions)
            .activities(activities)
            .extensions(extension_configs)
            .settings(settings)
            .author(author)
            .build()
            .expect("valid recipe");

        Ok(recipe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::Response;

    #[tokio::test]
    async fn test_add_final_output_tool() -> Result<()> {
        let agent = Agent::new();

        let response = Response {
            json_schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "result": {"type": "string"}
                }
            })),
        };

        agent.add_final_output_tool(response).await;

        let tools = agent.list_tools(None).await;
        let final_output_tool = tools
            .iter()
            .find(|tool| tool.name == FINAL_OUTPUT_TOOL_NAME);

        assert!(
            final_output_tool.is_some(),
            "Final output tool should be present after adding"
        );

        let prompt_manager = agent.prompt_manager.lock().await;
        let system_prompt =
            prompt_manager.build_system_prompt(vec![], None, serde_json::Value::Null, None, None);

        let final_output_tool_ref = agent.final_output_tool.lock().await;
        let final_output_tool_system_prompt =
            final_output_tool_ref.as_ref().unwrap().system_prompt();
        assert!(system_prompt.contains(&final_output_tool_system_prompt));
        Ok(())
    }
}
