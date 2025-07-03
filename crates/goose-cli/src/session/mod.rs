mod builder;
mod completion;
mod export;
mod input;
mod output;
mod prompt;
mod thinking;

pub use self::export::message_to_markdown;
pub use builder::{build_session, SessionBuilderConfig, SessionSettings};
use console::Color;
use goose::agents::AgentEvent;
use goose::permission::permission_confirmation::PrincipalType;
use goose::permission::Permission;
use goose::permission::PermissionConfirmation;
use goose::providers::base::Provider;
pub use goose::session::Identifier;

use anyhow::{Context, Result};
use completion::GooseCompleter;
use etcetera::{choose_app_strategy, AppStrategy};
use goose::agents::extension::{Envs, ExtensionConfig};
use goose::agents::{Agent, SessionConfig};
use goose::config::Config;
use goose::message::{Message, MessageContent};
use goose::session;
use input::InputResult;
use mcp_core::handler::ToolError;
use mcp_core::prompt::PromptMessage;
use mcp_core::protocol::JsonRpcMessage;
use mcp_core::protocol::JsonRpcNotification;

use rand::{distributions::Alphanumeric, Rng};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio;

pub enum RunMode {
    Normal,
    Plan,
}

pub struct Session {
    agent: Agent,
    messages: Vec<Message>,
    session_file: Option<PathBuf>,
    // Cache for completion data - using std::sync for thread safety without async
    completion_cache: Arc<std::sync::RwLock<CompletionCache>>,
    debug: bool, // New field for debug mode
    run_mode: RunMode,
    scheduled_job_id: Option<String>, // ID of the scheduled job that triggered this session
    max_turns: Option<u32>,
}

// Cache structure for completion data
struct CompletionCache {
    prompts: HashMap<String, Vec<String>>,
    prompt_info: HashMap<String, output::PromptInfo>,
    last_updated: Instant,
}

impl CompletionCache {
    fn new() -> Self {
        Self {
            prompts: HashMap::new(),
            prompt_info: HashMap::new(),
            last_updated: Instant::now(),
        }
    }
}

pub enum PlannerResponseType {
    Plan,
    ClarifyingQuestions,
}

/// Decide if the planner's reponse is a plan or a clarifying question
///
/// This function is called after the planner has generated a response
/// to the user's message. The response is either a plan or a clarifying
/// question.
pub async fn classify_planner_response(
    message_text: String,
    provider: Arc<dyn Provider>,
) -> Result<PlannerResponseType> {
    let prompt = format!("The text below is the output from an AI model which can either provide a plan or list of clarifying questions. Based on the text below, decide if the output is a \"plan\" or \"clarifying questions\".\n---\n{message_text}");

    // Generate the description
    let message = Message::user().with_text(&prompt);
    let (result, _usage) = provider
        .complete(
            "Reply only with the classification label: \"plan\" or \"clarifying questions\"",
            &[message],
            &[],
        )
        .await?;

    // println!("classify_planner_response: {result:?}\n"); // TODO: remove

    let predicted = result.as_concat_text();
    if predicted.to_lowercase().contains("plan") {
        Ok(PlannerResponseType::Plan)
    } else {
        Ok(PlannerResponseType::ClarifyingQuestions)
    }
}

impl Session {
    pub fn new(
        agent: Agent,
        session_file: Option<PathBuf>,
        debug: bool,
        scheduled_job_id: Option<String>,
        max_turns: Option<u32>,
    ) -> Self {
        let messages = if let Some(session_file) = &session_file {
            match session::read_messages(session_file) {
                Ok(msgs) => msgs,
                Err(e) => {
                    eprintln!("Warning: Failed to load message history: {}", e);
                    Vec::new()
                }
            }
        } else {
            // Don't try to read messages if we're not saving sessions
            Vec::new()
        };

        Session {
            agent,
            messages,
            session_file,
            completion_cache: Arc::new(std::sync::RwLock::new(CompletionCache::new())),
            debug,
            run_mode: RunMode::Normal,
            scheduled_job_id,
            max_turns,
        }
    }

    /// Helper function to summarize context messages
    async fn summarize_context_messages(
        messages: &mut Vec<Message>,
        agent: &Agent,
        message_suffix: &str,
    ) -> Result<()> {
        // Summarize messages to fit within context length
        let (summarized_messages, _) = agent.summarize_context(messages).await?;
        let msg = format!("Context maxed out\n{}\n{}", "-".repeat(50), message_suffix);
        output::render_text(&msg, Some(Color::Yellow), true);
        *messages = summarized_messages;

        Ok(())
    }

    /// Add a stdio extension to the session
    ///
    /// # Arguments
    /// * `extension_command` - Full command string including environment variables
    ///   Format: "ENV1=val1 ENV2=val2 command args..."
    pub async fn add_extension(&mut self, extension_command: String) -> Result<()> {
        let mut parts: Vec<&str> = extension_command.split_whitespace().collect();
        let mut envs = std::collections::HashMap::new();

        // Parse environment variables (format: KEY=value)
        while let Some(part) = parts.first() {
            if !part.contains('=') {
                break;
            }
            let env_part = parts.remove(0);
            let (key, value) = env_part.split_once('=').unwrap();
            envs.insert(key.to_string(), value.to_string());
        }

        if parts.is_empty() {
            return Err(anyhow::anyhow!("No command provided in extension string"));
        }

        let cmd = parts.remove(0).to_string();
        // Generate a random name for the ephemeral extension
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let config = ExtensionConfig::Stdio {
            name,
            cmd,
            args: parts.iter().map(|s| s.to_string()).collect(),
            envs: Envs::new(envs),
            env_keys: Vec::new(),
            description: Some(goose::config::DEFAULT_EXTENSION_DESCRIPTION.to_string()),
            // TODO: should set timeout
            timeout: Some(goose::config::DEFAULT_EXTENSION_TIMEOUT),
            bundled: None,
        };

        self.agent
            .add_extension(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start extension: {}", e))?;

        // Invalidate the completion cache when a new extension is added
        self.invalidate_completion_cache().await;

        Ok(())
    }

    /// Add a remote extension to the session
    ///
    /// # Arguments
    /// * `extension_url` - URL of the server
    pub async fn add_remote_extension(&mut self, extension_url: String) -> Result<()> {
        let name: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let config = ExtensionConfig::Sse {
            name,
            uri: extension_url,
            envs: Envs::new(HashMap::new()),
            env_keys: Vec::new(),
            description: Some(goose::config::DEFAULT_EXTENSION_DESCRIPTION.to_string()),
            // TODO: should set timeout
            timeout: Some(goose::config::DEFAULT_EXTENSION_TIMEOUT),
            bundled: None,
        };

        self.agent
            .add_extension(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start extension: {}", e))?;

        // Invalidate the completion cache when a new extension is added
        self.invalidate_completion_cache().await;

        Ok(())
    }

    /// Add a builtin extension to the session
    ///
    /// # Arguments
    /// * `builtin_name` - Name of the builtin extension(s), comma separated
    pub async fn add_builtin(&mut self, builtin_name: String) -> Result<()> {
        for name in builtin_name.split(',') {
            let config = ExtensionConfig::Builtin {
                name: name.trim().to_string(),
                display_name: None,
                // TODO: should set a timeout
                timeout: Some(goose::config::DEFAULT_EXTENSION_TIMEOUT),
                bundled: None,
            };
            self.agent
                .add_extension(config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to start builtin extension: {}", e))?;
        }

        // Invalidate the completion cache when a new extension is added
        self.invalidate_completion_cache().await;

        Ok(())
    }

    pub async fn list_prompts(
        &mut self,
        extension: Option<String>,
    ) -> Result<HashMap<String, Vec<String>>> {
        let prompts = self.agent.list_extension_prompts().await;

        // Early validation if filtering by extension
        if let Some(filter) = &extension {
            if !prompts.contains_key(filter) {
                return Err(anyhow::anyhow!("Extension '{}' not found", filter));
            }
        }

        // Convert prompts into filtered map of extension names to prompt names
        Ok(prompts
            .into_iter()
            .filter(|(ext, _)| extension.as_ref().is_none_or(|f| f == ext))
            .map(|(extension, prompt_list)| {
                let names = prompt_list.into_iter().map(|p| p.name).collect();
                (extension, names)
            })
            .collect())
    }

    pub async fn get_prompt_info(&mut self, name: &str) -> Result<Option<output::PromptInfo>> {
        let prompts = self.agent.list_extension_prompts().await;

        // Find which extension has this prompt
        for (extension, prompt_list) in prompts {
            if let Some(prompt) = prompt_list.iter().find(|p| p.name == name) {
                return Ok(Some(output::PromptInfo {
                    name: prompt.name.clone(),
                    description: prompt.description.clone(),
                    arguments: prompt.arguments.clone(),
                    extension: Some(extension),
                }));
            }
        }

        Ok(None)
    }

    pub async fn get_prompt(&mut self, name: &str, arguments: Value) -> Result<Vec<PromptMessage>> {
        let result = self.agent.get_prompt(name, arguments).await?;
        Ok(result.messages)
    }

    /// Process a single message and get the response
    async fn process_message(&mut self, message: String) -> Result<()> {
        self.messages.push(Message::user().with_text(&message));
        // Get the provider from the agent for description generation
        let provider = self.agent.provider().await?;

        // Persist messages with provider for automatic description generation
        if let Some(session_file) = &self.session_file {
            session::persist_messages_with_schedule_id(
                session_file,
                &self.messages,
                Some(provider),
                self.scheduled_job_id.clone(),
            )
            .await?;
        }

        // Track the current directory and last instruction in projects.json
        let session_id = self
            .session_file
            .as_ref()
            .and_then(|p| p.file_stem())
            .and_then(|s| s.to_str())
            .map(|s| s.to_string());

        if let Err(e) =
            crate::project_tracker::update_project_tracker(Some(&message), session_id.as_deref())
        {
            eprintln!(
                "Warning: Failed to update project tracker with instruction: {}",
                e
            );
        }

        self.process_agent_response(false).await?;
        Ok(())
    }

    /// Start an interactive session, optionally with an initial message
    pub async fn interactive(&mut self, message: Option<String>) -> Result<()> {
        // Process initial message if provided
        if let Some(msg) = message {
            self.process_message(msg).await?;
        }

        // Initialize the completion cache
        self.update_completion_cache().await?;

        // Create a new editor with our custom completer
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::Circular)
            .build();
        let mut editor =
            rustyline::Editor::<GooseCompleter, rustyline::history::DefaultHistory>::with_config(
                config,
            )?;

        // Set up the completer with a reference to the completion cache
        let completer = GooseCompleter::new(self.completion_cache.clone());
        editor.set_helper(Some(completer));

        // Create and use a global history file in ~/.config/goose directory
        // This allows command history to persist across different chat sessions
        // instead of being tied to each individual session's messages
        let strategy =
            choose_app_strategy(crate::APP_STRATEGY.clone()).expect("goose requires a home dir");
        let config_dir = strategy.config_dir();
        let history_file = config_dir.join("history.txt");

        // Ensure config directory exists
        if let Some(parent) = history_file.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Load history from the global file
        if history_file.exists() {
            if let Err(err) = editor.load_history(&history_file) {
                eprintln!("Warning: Failed to load command history: {}", err);
            }
        }

        // Helper function to save history after commands
        let save_history =
            |editor: &mut rustyline::Editor<GooseCompleter, rustyline::history::DefaultHistory>| {
                if let Err(err) = editor.save_history(&history_file) {
                    eprintln!("Warning: Failed to save command history: {}", err);
                }
            };

        output::display_greeting();
        loop {
            // Display context usage before each prompt
            self.display_context_usage().await?;

            match input::get_input(&mut editor)? {
                input::InputResult::Message(content) => {
                    match self.run_mode {
                        RunMode::Normal => {
                            save_history(&mut editor);

                            self.messages.push(Message::user().with_text(&content));

                            // Track the current directory and last instruction in projects.json
                            let session_id = self
                                .session_file
                                .as_ref()
                                .and_then(|p| p.file_stem())
                                .and_then(|s| s.to_str())
                                .map(|s| s.to_string());

                            if let Err(e) = crate::project_tracker::update_project_tracker(
                                Some(&content),
                                session_id.as_deref(),
                            ) {
                                eprintln!("Warning: Failed to update project tracker with instruction: {}", e);
                            }

                            // Get the provider from the agent for description generation
                            let provider = self.agent.provider().await?;

                            // Persist messages with provider for automatic description generation
                            if let Some(session_file) = &self.session_file {
                                session::persist_messages_with_schedule_id(
                                    session_file,
                                    &self.messages,
                                    Some(provider),
                                    self.scheduled_job_id.clone(),
                                )
                                .await?;
                            }

                            output::show_thinking();
                            self.process_agent_response(true).await?;
                            output::hide_thinking();
                        }
                        RunMode::Plan => {
                            let mut plan_messages = self.messages.clone();
                            plan_messages.push(Message::user().with_text(&content));
                            let reasoner = get_reasoner()?;
                            self.plan_with_reasoner_model(plan_messages, reasoner)
                                .await?;
                        }
                    }
                }
                input::InputResult::Exit => break,
                input::InputResult::AddExtension(cmd) => {
                    save_history(&mut editor);

                    match self.add_extension(cmd.clone()).await {
                        Ok(_) => output::render_extension_success(&cmd),
                        Err(e) => output::render_extension_error(&cmd, &e.to_string()),
                    }
                }
                input::InputResult::AddBuiltin(names) => {
                    save_history(&mut editor);

                    match self.add_builtin(names.clone()).await {
                        Ok(_) => output::render_builtin_success(&names),
                        Err(e) => output::render_builtin_error(&names, &e.to_string()),
                    }
                }
                input::InputResult::ToggleTheme => {
                    save_history(&mut editor);

                    let current = output::get_theme();
                    let new_theme = match current {
                        output::Theme::Light => {
                            println!("Switching to Dark theme");
                            output::Theme::Dark
                        }
                        output::Theme::Dark => {
                            println!("Switching to Ansi theme");
                            output::Theme::Ansi
                        }
                        output::Theme::Ansi => {
                            println!("Switching to Light theme");
                            output::Theme::Light
                        }
                    };
                    output::set_theme(new_theme);
                    continue;
                }
                input::InputResult::Retry => continue,
                input::InputResult::ListPrompts(extension) => {
                    save_history(&mut editor);

                    match self.list_prompts(extension).await {
                        Ok(prompts) => output::render_prompts(&prompts),
                        Err(e) => output::render_error(&e.to_string()),
                    }
                }
                input::InputResult::GooseMode(mode) => {
                    save_history(&mut editor);

                    let config = Config::global();
                    let mode = mode.to_lowercase();

                    // Check if mode is valid
                    if !["auto", "approve", "chat", "smart_approve"].contains(&mode.as_str()) {
                        output::render_error(&format!(
                            "Invalid mode '{}'. Mode must be one of: auto, approve, chat",
                            mode
                        ));
                        continue;
                    }

                    config
                        .set_param("GOOSE_MODE", Value::String(mode.to_string()))
                        .unwrap();
                    output::goose_mode_message(&format!("Goose mode set to '{}'", mode));
                    continue;
                }
                input::InputResult::Plan(options) => {
                    self.run_mode = RunMode::Plan;
                    output::render_enter_plan_mode();

                    let message_text = options.message_text;
                    if message_text.is_empty() {
                        continue;
                    }
                    let mut plan_messages = self.messages.clone();
                    plan_messages.push(Message::user().with_text(&message_text));

                    let reasoner = get_reasoner()?;
                    self.plan_with_reasoner_model(plan_messages, reasoner)
                        .await?;
                }
                input::InputResult::EndPlan => {
                    self.run_mode = RunMode::Normal;
                    output::render_exit_plan_mode();
                    continue;
                }
                input::InputResult::Clear => {
                    save_history(&mut editor);

                    self.messages.clear();
                    tracing::info!("Chat context cleared by user.");
                    output::render_message(
                        &Message::assistant().with_text("Chat context cleared."),
                        self.debug,
                    );
                    continue;
                }
                input::InputResult::PromptCommand(opts) => {
                    save_history(&mut editor);
                    self.handle_prompt_command(opts).await?;
                }
                InputResult::Recipe(filepath_opt) => {
                    println!("{}", console::style("Generating Recipe").green());

                    output::show_thinking();
                    let recipe = self.agent.create_recipe(self.messages.clone()).await;
                    output::hide_thinking();

                    match recipe {
                        Ok(recipe) => {
                            // Use provided filepath or default
                            let filepath_str = filepath_opt.as_deref().unwrap_or("recipe.yaml");
                            match self.save_recipe(&recipe, filepath_str) {
                                Ok(path) => println!(
                                    "{}",
                                    console::style(format!("Saved recipe to {}", path.display()))
                                        .green()
                                ),
                                Err(e) => {
                                    println!("{}", console::style(e).red());
                                }
                            }
                        }
                        Err(e) => {
                            println!(
                                "{}: {:?}",
                                console::style("Failed to generate recipe").red(),
                                e
                            );
                        }
                    }

                    continue;
                }
                InputResult::Summarize => {
                    save_history(&mut editor);

                    let prompt = "Are you sure you want to summarize this conversation? This will condense the message history.";
                    let should_summarize =
                        match cliclack::confirm(prompt).initial_value(true).interact() {
                            Ok(choice) => choice,
                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::Interrupted {
                                    false // If interrupted, set should_summarize to false
                                } else {
                                    return Err(e.into());
                                }
                            }
                        };

                    if should_summarize {
                        println!("{}", console::style("Summarizing conversation...").yellow());
                        output::show_thinking();

                        // Get the provider for summarization
                        let provider = self.agent.provider().await?;

                        // Call the summarize_context method which uses the summarize_messages function
                        let (summarized_messages, _) =
                            self.agent.summarize_context(&self.messages).await?;

                        // Update the session messages with the summarized ones
                        self.messages = summarized_messages;

                        // Persist the summarized messages
                        if let Some(session_file) = &self.session_file {
                            session::persist_messages_with_schedule_id(
                                session_file,
                                &self.messages,
                                Some(provider),
                                self.scheduled_job_id.clone(),
                            )
                            .await?;
                        }

                        output::hide_thinking();
                        println!(
                            "{}",
                            console::style("Conversation has been summarized.").green()
                        );
                        println!(
                            "{}",
                            console::style(
                                "Key information has been preserved while reducing context length."
                            )
                            .green()
                        );
                    } else {
                        println!("{}", console::style("Summarization cancelled.").yellow());
                    }

                    continue;
                }
            }
        }

        println!(
            "\nClosing session.{}",
            self.session_file
                .as_ref()
                .map(|p| format!(" Recorded to {}", p.display()))
                .unwrap_or_default()
        );
        Ok(())
    }

    async fn plan_with_reasoner_model(
        &mut self,
        plan_messages: Vec<Message>,
        reasoner: Arc<dyn Provider>,
    ) -> Result<(), anyhow::Error> {
        let plan_prompt = self.agent.get_plan_prompt().await?;
        output::show_thinking();
        let (plan_response, _usage) = reasoner.complete(&plan_prompt, &plan_messages, &[]).await?;
        output::render_message(&plan_response, self.debug);
        output::hide_thinking();
        let planner_response_type =
            classify_planner_response(plan_response.as_concat_text(), self.agent.provider().await?)
                .await?;

        match planner_response_type {
            PlannerResponseType::Plan => {
                println!();
                let should_act = match cliclack::confirm(
                    "Do you want to clear message history & act on this plan?",
                )
                .initial_value(true)
                .interact()
                {
                    Ok(choice) => choice,
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::Interrupted {
                            false // If interrupted, set should_act to false
                        } else {
                            return Err(e.into());
                        }
                    }
                };
                if should_act {
                    output::render_act_on_plan();
                    self.run_mode = RunMode::Normal;
                    // set goose mode: auto if that isn't already the case
                    let config = Config::global();
                    let curr_goose_mode =
                        config.get_param("GOOSE_MODE").unwrap_or("auto".to_string());
                    if curr_goose_mode != "auto" {
                        config
                            .set_param("GOOSE_MODE", Value::String("auto".to_string()))
                            .unwrap();
                    }

                    // clear the messages before acting on the plan
                    self.messages.clear();
                    // add the plan response as a user message
                    let plan_message = Message::user().with_text(plan_response.as_concat_text());
                    self.messages.push(plan_message);
                    // act on the plan
                    output::show_thinking();
                    self.process_agent_response(true).await?;
                    output::hide_thinking();

                    // Reset run & goose mode
                    if curr_goose_mode != "auto" {
                        config
                            .set_param("GOOSE_MODE", Value::String(curr_goose_mode.to_string()))
                            .unwrap();
                    }
                } else {
                    // add the plan response (assistant message) & carry the conversation forward
                    // in the next round, the user might wanna slightly modify the plan
                    self.messages.push(plan_response);
                }
            }
            PlannerResponseType::ClarifyingQuestions => {
                // add the plan response (assistant message) & carry the conversation forward
                // in the next round, the user will answer the clarifying questions
                self.messages.push(plan_response);
            }
        }

        Ok(())
    }

    /// Process a single message and exit
    pub async fn headless(&mut self, message: String) -> Result<()> {
        self.process_message(message).await
    }

    async fn process_agent_response(&mut self, interactive: bool) -> Result<()> {
        let session_config = self.session_file.as_ref().map(|s| {
            let session_id = session::Identifier::Path(s.clone());
            SessionConfig {
                id: session_id.clone(),
                working_dir: std::env::current_dir()
                    .expect("failed to get current session working directory"),
                schedule_id: self.scheduled_job_id.clone(),
                execution_mode: None,
                max_turns: self.max_turns,
            }
        });
        let mut stream = self
            .agent
            .reply(&self.messages, session_config.clone())
            .await?;

        let mut progress_bars = output::McpSpinners::new();

        use futures::StreamExt;
        loop {
            tokio::select! {
                result = stream.next() => {
                    match result {
                        Some(Ok(AgentEvent::Message(message))) => {
                            // If it's a confirmation request, get approval but otherwise do not render/persist
                            if let Some(MessageContent::ToolConfirmationRequest(confirmation)) = message.content.first() {
                                output::hide_thinking();

                                // Format the confirmation prompt
                                let prompt = "Goose would like to call the above tool, do you allow?".to_string();

                                // Get confirmation from user
                                let permission_result = cliclack::select(prompt)
                                    .item(Permission::AllowOnce, "Allow", "Allow the tool call once")
                                    .item(Permission::AlwaysAllow, "Always Allow", "Always allow the tool call")
                                    .item(Permission::DenyOnce, "Deny", "Deny the tool call")
                                    .item(Permission::Cancel, "Cancel", "Cancel the AI response and tool call")
                                    .interact();

                                let permission = match permission_result {
                                    Ok(p) => p, // If Ok, use the selected permission
                                    Err(e) => {
                                        // Check if the error is an interruption (Ctrl+C/Cmd+C, Escape)
                                        if e.kind() == std::io::ErrorKind::Interrupted {
                                            Permission::Cancel // If interrupted, set permission to Cancel
                                        } else {
                                            return Err(e.into()); // Otherwise, convert and propagate the original error
                                        }
                                    }
                                };

                                if permission == Permission::Cancel {
                                    output::render_text("Tool call cancelled. Returning to chat...", Some(Color::Yellow), true);

                                    let mut response_message = Message::user();
                                    response_message.content.push(MessageContent::tool_response(
                                        confirmation.id.clone(),
                                        Err(ToolError::ExecutionError("Tool call cancelled by user".to_string()))
                                    ));
                                    self.messages.push(response_message);
                                    if let Some(session_file) = &self.session_file {
                                        session::persist_messages_with_schedule_id(
                                            session_file,
                                            &self.messages,
                                            None,
                                            self.scheduled_job_id.clone(),
                                        )
                                        .await?;
                                    }

                                    drop(stream);
                                    break;
                                } else {
                                    self.agent.handle_confirmation(confirmation.id.clone(), PermissionConfirmation {
                                        principal_type: PrincipalType::Tool,
                                        permission,
                                    },).await;
                                }
                            } else if let Some(MessageContent::ContextLengthExceeded(_)) = message.content.first() {
                                output::hide_thinking();

                                // Check for user-configured default context strategy
                                let config = Config::global();
                                let context_strategy = config.get_param::<String>("GOOSE_CONTEXT_STRATEGY")
                                    .unwrap_or_else(|_| if interactive { "prompt".to_string() } else { "summarize".to_string() });

                                let selected = match context_strategy.as_str() {
                                    "clear" => "clear",
                                    "truncate" => "truncate",
                                    "summarize" => "summarize",
                                    _ => {
                                        if interactive {
                                            // In interactive mode with no default, ask the user what to do
                                            let prompt = "The model's context length is maxed out. You will need to reduce the # msgs. Do you want to?".to_string();
                                            cliclack::select(prompt)
                                                .item("clear", "Clear Session", "Removes all messages from Goose's memory")
                                                .item("truncate", "Truncate Messages", "Removes old messages till context is within limits")
                                                .item("summarize", "Summarize Session", "Summarize the session to reduce context length")
                                                .interact()?
                                        } else {
                                            // In headless mode, default to summarize
                                            "summarize"
                                        }
                                    }
                                };

                                match selected {
                                    "clear" => {
                                        self.messages.clear();
                                        let msg = if context_strategy == "clear" {
                                            format!("Context maxed out - automatically cleared session.\n{}", "-".repeat(50))
                                        } else {
                                            format!("Session cleared.\n{}", "-".repeat(50))
                                        };
                                        output::render_text(&msg, Some(Color::Yellow), true);
                                        break;  // exit the loop to hand back control to the user
                                    }
                                    "truncate" => {
                                        // Truncate messages to fit within context length
                                        let (truncated_messages, _) = self.agent.truncate_context(&self.messages).await?;
                                        let msg = if context_strategy == "truncate" {
                                            format!("Context maxed out - automatically truncated messages.\n{}\nGoose tried its best to truncate messages for you.", "-".repeat(50))
                                        } else {
                                            format!("Context maxed out\n{}\nGoose tried its best to truncate messages for you.", "-".repeat(50))
                                        };
                                        output::render_text("", Some(Color::Yellow), true);
                                        output::render_text(&msg, Some(Color::Yellow), true);
                                        self.messages = truncated_messages;
                                    }
                                    "summarize" => {
                                        // Use the helper function to summarize context
                                        let message_suffix = if context_strategy == "summarize" {
                                            "Goose automatically summarized messages for you."
                                        } else if interactive {
                                            "Goose summarized messages for you."
                                        } else {
                                            "Goose automatically summarized messages to continue processing."
                                        };
                                        Self::summarize_context_messages(&mut self.messages, &self.agent, message_suffix).await?;
                                    }
                                    _ => {
                                        unreachable!()
                                    }
                                }

                                // Restart the stream after handling ContextLengthExceeded
                                stream = self
                                    .agent
                                    .reply(
                                        &self.messages,
                                        session_config.clone(),
                                    )
                                    .await?;
                            }
                            // otherwise we have a model/tool to render
                            else {
                                self.messages.push(message.clone());

                                // No need to update description on assistant messages
                                if let Some(session_file) = &self.session_file {
                                    session::persist_messages_with_schedule_id(
                                        session_file,
                                        &self.messages,
                                        None,
                                        self.scheduled_job_id.clone(),
                                    )
                                    .await?;
                                }

                                if interactive {output::hide_thinking()};
                                let _ = progress_bars.hide();
                                output::render_message(&message, self.debug);
                                if interactive {output::show_thinking()};
                            }
                        }
                        Some(Ok(AgentEvent::McpNotification((_id, message)))) => {
                                if let JsonRpcMessage::Notification(JsonRpcNotification{
                                    method,
                                    params: Some(Value::Object(o)),
                                    ..
                                }) = message {
                                match method.as_str() {
                                    "notifications/message" => {
                                        let data = o.get("data").unwrap_or(&Value::Null);
                                        let (formatted_message, subagent_id, _notification_type) = match data {
                                            Value::String(s) => (s.clone(), None, None),
                                            Value::Object(o) => {
                                                // Check for subagent notification structure first
                                                if let Some(Value::String(msg)) = o.get("message") {
                                                    // Extract subagent info for better display
                                                    let subagent_id = o.get("subagent_id")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("unknown");
                                                    let notification_type = o.get("type")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("");

                                                    let formatted = match notification_type {
                                                        "subagent_created" | "completed" | "terminated" => {
                                                            format!("🤖 {}", msg)
                                                        }
                                                        "tool_usage" | "tool_completed" | "tool_error" => {
                                                            format!("🔧 {}", msg)
                                                        }
                                                        "message_processing" | "turn_progress" => {
                                                            format!("💭 {}", msg)
                                                        }
                                                        "response_generated" => {
                                                            // Check verbosity setting for subagent response content
                                                            let config = Config::global();
                                                            let min_priority = config
                                                                .get_param::<f32>("GOOSE_CLI_MIN_PRIORITY")
                                                                .ok()
                                                                .unwrap_or(0.5);

                                                            if min_priority > 0.1 && !self.debug {
                                                                // High/Medium verbosity: show truncated response
                                                                if let Some(response_content) = msg.strip_prefix("Responded: ") {
                                                                    if response_content.len() > 100 {
                                                                        format!("🤖 Responded: {}...", &response_content[..100])
                                                                    } else {
                                                                        format!("🤖 {}", msg)
                                                                    }
                                                                } else {
                                                                    format!("🤖 {}", msg)
                                                                }
                                                            } else {
                                                                // All verbosity or debug: show full response
                                                                format!("🤖 {}", msg)
                                                            }
                                                        }
                                                        _ => {
                                                            msg.to_string()
                                                        }
                                                    };
                                                    (formatted, Some(subagent_id.to_string()), Some(notification_type.to_string()))
                                                } else if let Some(Value::String(output)) = o.get("output") {
                                                    // Fallback for other MCP notification types
                                                    (output.to_owned(), None, None)
                                                } else {
                                                    (data.to_string(), None, None)
                                                }
                                            },
                                            v => {
                                                (v.to_string(), None, None)
                                            },
                                        };

                                        // Handle subagent notifications - show immediately
                                        if let Some(_id) = subagent_id {
                                            // Show subagent notifications immediately (no buffering) with compact spacing
                                            if interactive {
                                                let _ = progress_bars.hide();
                                                println!("{}", console::style(&formatted_message).green().dim());
                                            } else {
                                                progress_bars.log(&formatted_message);
                                            }
                                        } else {
                                            // Non-subagent notification, display immediately with compact spacing
                                            if interactive {
                                                let _ = progress_bars.hide();
                                                println!("{}", console::style(&formatted_message).green().dim());
                                            } else {
                                                progress_bars.log(&formatted_message);
                                            }
                                        }
                                    },
                                    "notifications/progress" => {
                                        let progress = o.get("progress").and_then(|v| v.as_f64());
                                        let token = o.get("progressToken").map(|v| v.to_string());
                                        let message = o.get("message").and_then(|v| v.as_str());
                                        let total = o
                                            .get("total")
                                            .and_then(|v| v.as_f64());
                                        if let (Some(progress), Some(token)) = (progress, token) {
                                            progress_bars.update(
                                                token.as_str(),
                                                progress,
                                                total,
                                                message,
                                            );
                                        }
                                    },
                                    _ => (),
                                }
                            }
                        }
                        Some(Ok(AgentEvent::ModelChange { model, mode })) => {
                            // Log model change if in debug mode
                            if self.debug {
                                eprintln!("Model changed to {} in {} mode", model, mode);
                            }
                        }

                        Some(Err(e)) => {
                            eprintln!("Error: {}", e);
                            drop(stream);
                            if let Err(e) = self.handle_interrupted_messages(false).await {
                                eprintln!("Error handling interruption: {}", e);
                            }
                            output::render_error(
                                "The error above was an exception we were not able to handle.\n\
                                These errors are often related to connection or authentication\n\
                                We've removed the conversation up to the most recent user message\n\
                                - depending on the error you may be able to continue",
                            );
                            break;
                        }
                        None => break,
                    }
                }
                _ = tokio::signal::ctrl_c() => {
                    drop(stream);
                    if let Err(e) = self.handle_interrupted_messages(true).await {
                        eprintln!("Error handling interruption: {}", e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    async fn handle_interrupted_messages(&mut self, interrupt: bool) -> Result<()> {
        // First, get any tool requests from the last message if it exists
        let tool_requests = self
            .messages
            .last()
            .filter(|msg| msg.role == mcp_core::role::Role::Assistant)
            .map_or(Vec::new(), |msg| {
                msg.content
                    .iter()
                    .filter_map(|content| {
                        if let MessageContent::ToolRequest(req) = content {
                            Some((req.id.clone(), req.tool_call.clone()))
                        } else {
                            None
                        }
                    })
                    .collect()
            });

        if !tool_requests.is_empty() {
            // Interrupted during a tool request
            // Create tool responses for all interrupted tool requests
            let mut response_message = Message::user();
            let last_tool_name = tool_requests
                .last()
                .and_then(|(_, tool_call)| tool_call.as_ref().ok().map(|tool| tool.name.clone()))
                .unwrap_or_else(|| "tool".to_string());

            let notification = if interrupt {
                "Interrupted by the user to make a correction".to_string()
            } else {
                "An uncaught error happened during tool use".to_string()
            };
            for (req_id, _) in &tool_requests {
                response_message.content.push(MessageContent::tool_response(
                    req_id.clone(),
                    Err(ToolError::ExecutionError(notification.clone())),
                ));
            }
            self.messages.push(response_message);

            // No need for description update here
            if let Some(session_file) = &self.session_file {
                session::persist_messages_with_schedule_id(
                    session_file,
                    &self.messages,
                    None,
                    self.scheduled_job_id.clone(),
                )
                .await?;
            }

            let prompt = format!(
                "The existing call to {} was interrupted. How would you like to proceed?",
                last_tool_name
            );
            self.messages.push(Message::assistant().with_text(&prompt));

            // No need for description update here
            if let Some(session_file) = &self.session_file {
                session::persist_messages_with_schedule_id(
                    session_file,
                    &self.messages,
                    None,
                    self.scheduled_job_id.clone(),
                )
                .await?;
            }

            output::render_message(&Message::assistant().with_text(&prompt), self.debug);
        } else {
            // An interruption occurred outside of a tool request-response.
            if let Some(last_msg) = self.messages.last() {
                if last_msg.role == mcp_core::role::Role::User {
                    match last_msg.content.first() {
                        Some(MessageContent::ToolResponse(_)) => {
                            // Interruption occurred after a tool had completed but not assistant reply
                            let prompt = "The tool calling loop was interrupted. How would you like to proceed?";
                            self.messages.push(Message::assistant().with_text(prompt));

                            // No need for description update here
                            if let Some(session_file) = &self.session_file {
                                session::persist_messages_with_schedule_id(
                                    session_file,
                                    &self.messages,
                                    None,
                                    self.scheduled_job_id.clone(),
                                )
                                .await?;
                            }

                            output::render_message(
                                &Message::assistant().with_text(prompt),
                                self.debug,
                            );
                        }
                        Some(_) => {
                            // A real users message
                            self.messages.pop();
                            let prompt = "Interrupted before the model replied and removed the last message.";
                            output::render_message(
                                &Message::assistant().with_text(prompt),
                                self.debug,
                            );
                        }
                        None => panic!("No content in last message"),
                    }
                }
            }
        }
        Ok(())
    }

    pub fn session_file(&self) -> Option<PathBuf> {
        self.session_file.clone()
    }

    /// Update the completion cache with fresh data
    /// This should be called before the interactive session starts
    pub async fn update_completion_cache(&mut self) -> Result<()> {
        // Get fresh data
        let prompts = self.agent.list_extension_prompts().await;

        // Update the cache with write lock
        let mut cache = self.completion_cache.write().unwrap();
        cache.prompts.clear();
        cache.prompt_info.clear();

        for (extension, prompt_list) in prompts {
            let names: Vec<String> = prompt_list.iter().map(|p| p.name.clone()).collect();
            cache.prompts.insert(extension.clone(), names);

            for prompt in prompt_list {
                cache.prompt_info.insert(
                    prompt.name.clone(),
                    output::PromptInfo {
                        name: prompt.name.clone(),
                        description: prompt.description.clone(),
                        arguments: prompt.arguments.clone(),
                        extension: Some(extension.clone()),
                    },
                );
            }
        }

        cache.last_updated = Instant::now();
        Ok(())
    }

    /// Invalidate the completion cache
    /// This should be called when extensions are added or removed
    async fn invalidate_completion_cache(&self) {
        let mut cache = self.completion_cache.write().unwrap();
        cache.prompts.clear();
        cache.prompt_info.clear();
        cache.last_updated = Instant::now();
    }

    pub fn message_history(&self) -> Vec<Message> {
        self.messages.clone()
    }

    /// Render all past messages from the session history
    pub fn render_message_history(&self) {
        if self.messages.is_empty() {
            return;
        }

        // Print session restored message
        println!(
            "\n{} {} messages loaded into context.",
            console::style("Session restored:").green().bold(),
            console::style(self.messages.len()).green()
        );

        // Render each message
        for message in &self.messages {
            output::render_message(message, self.debug);
        }

        // Add a visual separator after restored messages
        println!(
            "\n{}\n",
            console::style("──────── New Messages ────────").dim()
        );
    }

    pub fn get_metadata(&self) -> Result<session::SessionMetadata> {
        if !self.session_file.as_ref().is_some_and(|f| f.exists()) {
            return Err(anyhow::anyhow!("Session file does not exist"));
        }

        session::read_metadata(self.session_file.as_ref().unwrap())
    }

    // Get the session's total token usage
    pub fn get_total_token_usage(&self) -> Result<Option<i32>> {
        let metadata = self.get_metadata()?;
        Ok(metadata.total_tokens)
    }

    /// Display enhanced context usage with session totals
    pub async fn display_context_usage(&self) -> Result<()> {
        let provider = self.agent.provider().await?;
        let model_config = provider.get_model_config();
        let context_limit = model_config.context_limit.unwrap_or(32000);

        match self.get_metadata() {
            Ok(metadata) => {
                let total_tokens = metadata.total_tokens.unwrap_or(0) as usize;

                output::display_context_usage(total_tokens, context_limit);
            }
            Err(_) => {
                output::display_context_usage(0, context_limit);
            }
        }

        Ok(())
    }

    /// Handle prompt command execution
    async fn handle_prompt_command(&mut self, opts: input::PromptCommandOptions) -> Result<()> {
        // name is required
        if opts.name.is_empty() {
            output::render_error("Prompt name argument is required");
            return Ok(());
        }

        if opts.info {
            match self.get_prompt_info(&opts.name).await? {
                Some(info) => output::render_prompt_info(&info),
                None => output::render_error(&format!("Prompt '{}' not found", opts.name)),
            }
        } else {
            // Convert the arguments HashMap to a Value
            let arguments = serde_json::to_value(opts.arguments)
                .map_err(|e| anyhow::anyhow!("Failed to serialize arguments: {}", e))?;

            match self.get_prompt(&opts.name, arguments).await {
                Ok(messages) => {
                    let start_len = self.messages.len();
                    let mut valid = true;
                    for (i, prompt_message) in messages.into_iter().enumerate() {
                        let msg = Message::from(prompt_message);
                        // ensure we get a User - Assistant - User type pattern
                        let expected_role = if i % 2 == 0 {
                            mcp_core::Role::User
                        } else {
                            mcp_core::Role::Assistant
                        };

                        if msg.role != expected_role {
                            output::render_error(&format!(
                                "Expected {:?} message at position {}, but found {:?}",
                                expected_role, i, msg.role
                            ));
                            valid = false;
                            // get rid of everything we added to messages
                            self.messages.truncate(start_len);
                            break;
                        }

                        if msg.role == mcp_core::Role::User {
                            output::render_message(&msg, self.debug);
                        }
                        self.messages.push(msg);
                    }

                    if valid {
                        output::show_thinking();
                        self.process_agent_response(true).await?;
                        output::hide_thinking();
                    }
                }
                Err(e) => output::render_error(&e.to_string()),
            }
        }

        Ok(())
    }

    /// Save a recipe to a file
    ///
    /// # Arguments
    /// * `recipe` - The recipe to save
    /// * `filepath_str` - The path to save the recipe to
    ///
    /// # Returns
    /// * `Result<PathBuf, String>` - The path the recipe was saved to or an error message
    fn save_recipe(
        &self,
        recipe: &goose::recipe::Recipe,
        filepath_str: &str,
    ) -> anyhow::Result<PathBuf> {
        let path_buf = PathBuf::from(filepath_str);
        let mut path = path_buf.clone();

        // Update the final path if it's relative
        if path_buf.is_relative() {
            // If the path is relative, resolve it relative to the current working directory
            let cwd = std::env::current_dir().context("Failed to get current directory")?;
            path = cwd.join(&path_buf);
        }

        // Check if parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                return Err(anyhow::anyhow!(
                    "Directory '{}' does not exist",
                    parent.display()
                ));
            }
        }

        // Try creating the file
        let file = std::fs::File::create(path.as_path())
            .context(format!("Failed to create file '{}'", path.display()))?;

        // Write YAML
        serde_yaml::to_writer(file, recipe).context("Failed to save recipe")?;

        Ok(path)
    }
}

fn get_reasoner() -> Result<Arc<dyn Provider>, anyhow::Error> {
    use goose::model::ModelConfig;
    use goose::providers::create;

    let config = Config::global();

    // Try planner-specific provider first, fallback to default provider
    let provider = if let Ok(provider) = config.get_param::<String>("GOOSE_PLANNER_PROVIDER") {
        provider
    } else {
        println!("WARNING: GOOSE_PLANNER_PROVIDER not found. Using default provider...");
        config
            .get_param::<String>("GOOSE_PROVIDER")
            .expect("No provider configured. Run 'goose configure' first")
    };

    // Try planner-specific model first, fallback to default model
    let model = if let Ok(model) = config.get_param::<String>("GOOSE_PLANNER_MODEL") {
        model
    } else {
        println!("WARNING: GOOSE_PLANNER_MODEL not found. Using default model...");
        config
            .get_param::<String>("GOOSE_MODEL")
            .expect("No model configured. Run 'goose configure' first")
    };

    let model_config = ModelConfig::new(model);
    let reasoner = create(&provider, model_config)?;

    Ok(reasoner)
}
