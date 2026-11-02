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
pub mod workspaces;
pub mod recovery;
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

#[cfg(feature = "gui")]
pub mod session_recovery;

pub mod chat_state;

#[cfg(feature="gui")]
pub mod chat_history;

#[cfg(feature="gui")]
pub mod transfer_queue;

pub mod text_diff;

pub mod execution;
pub mod execution_history;

pub mod recipes;

pub mod ssh_import;
pub mod tunnels;

pub mod interventions;
