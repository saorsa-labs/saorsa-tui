//! Export command for saving sessions to HTML.

use fae_agent::{FaeAgentError, SessionId, SessionStorage, export_to_html};
use std::path::Path;

/// Export command handler.
pub struct ExportCommand;

impl ExportCommand {
    /// Export a session to HTML.
    pub fn execute(session_id: &SessionId, output_path: &Path) -> Result<(), FaeAgentError> {
        let storage = SessionStorage::new()?;
        export_to_html(&storage, session_id, output_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_command_exists() {
        let _ = ExportCommand;
    }
}
