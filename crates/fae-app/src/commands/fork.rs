//! /fork command implementation for session branching.

use fae_agent::{FaeAgentError, SessionId, SessionStorage, fork_session};

/// The /fork command for creating session branches.
pub struct ForkCommand;

impl ForkCommand {
    /// Execute the fork command.
    ///
    /// Arguments:
    /// - `current_session_id`: The session to fork from
    /// - `title`: Optional title for the forked session
    pub fn execute(
        current_session_id: &SessionId,
        title: Option<String>,
    ) -> Result<SessionId, FaeAgentError> {
        let storage = SessionStorage::new()?;

        // Fork at the current point (all messages)
        let new_id = fork_session(&storage, current_session_id, None, title)?;

        Ok(new_id)
    }

    /// Fork at a specific message index.
    pub fn execute_at_index(
        current_session_id: &SessionId,
        fork_index: usize,
        title: Option<String>,
    ) -> Result<SessionId, FaeAgentError> {
        let storage = SessionStorage::new()?;
        let new_id = fork_session(&storage, current_session_id, Some(fork_index), title)?;
        Ok(new_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_command_exists() {
        // Verify ForkCommand structure exists
        let _ = ForkCommand;
    }
}
