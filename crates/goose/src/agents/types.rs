use crate::session;
use mcp_core::{Content, Tool, ToolResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// Type alias for the tool result channel receiver
pub type ToolResultReceiver = Arc<Mutex<mpsc::Receiver<(String, ToolResult<Vec<Content>>)>>>;

/// A frontend tool that will be executed by the frontend rather than an extension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendTool {
    pub name: String,
    pub tool: Tool,
}

/// Session configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Unique identifier for the session
    pub id: session::Identifier,
    /// Working directory for the session
    pub working_dir: PathBuf,
    /// ID of the schedule that triggered this session, if any
    pub schedule_id: Option<String>,
    /// Execution mode for scheduled jobs: "foreground" or "background"
    pub execution_mode: Option<String>,
    /// Maximum number of turns (iterations) allowed without user input
    pub max_turns: Option<u32>,
}
