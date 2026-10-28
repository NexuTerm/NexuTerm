#[cfg(feature="gui")]
pub mod terminal_mcp;
#[cfg(feature="gui")]
pub mod agent_profile;
#[cfg(feature="gui")]
mod agent_runtime;
#[cfg(feature = "gui")]
pub mod dictation;
#[cfg(feature = "gui")]
pub mod codex;
pub mod ai;
pub mod ai_config;
pub mod config;
pub mod connection;
pub mod desktop;
pub mod monitor;
pub mod rdp;
pub mod runtime;
pub mod ssh;
pub mod transfer;
pub mod vnc;

pub mod remote_edit;

pub mod ai_usage;

#[cfg(feature = "gui")]
pub mod approval_grants;

pub mod recovery;
pub mod workspaces;
