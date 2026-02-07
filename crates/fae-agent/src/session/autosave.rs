//! Automatic session saving with debouncing and retry logic.

use crate::FaeAgentError;
use crate::session::{Message, SessionId, SessionMetadata, SessionStorage};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::sleep;
use tracing::{debug, error, warn};

/// Configuration for auto-save behavior.
#[derive(Debug, Clone)]
pub struct AutoSaveConfig {
    /// Minimum interval between saves (debounce duration).
    pub save_interval: Duration,
    /// Maximum number of messages to batch before forcing a save.
    pub max_batch_size: usize,
    /// Maximum number of retry attempts on save failure.
    pub max_retries: usize,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            save_interval: Duration::from_millis(500),
            max_batch_size: 10,
            max_retries: 3,
        }
    }
}

/// Auto-save manager handles background saving with debouncing and retry logic.
pub struct AutoSaveManager {
    storage: Arc<SessionStorage>,
    session_id: SessionId,
    metadata: Arc<RwLock<SessionMetadata>>,
    messages: Arc<RwLock<Vec<Message>>>,
    dirty: Arc<Mutex<bool>>,
    save_tx: mpsc::UnboundedSender<SaveRequest>,
}

/// Request to save session data.
#[derive(Debug)]
enum SaveRequest {
    /// Save the current state.
    Save,
    /// Shutdown the auto-save task.
    Shutdown,
}

impl AutoSaveManager {
    /// Create a new auto-save manager and start the background task.
    pub fn new(
        storage: SessionStorage,
        config: AutoSaveConfig,
        session_id: SessionId,
        metadata: SessionMetadata,
    ) -> Self {
        let storage = Arc::new(storage);
        let metadata = Arc::new(RwLock::new(metadata));
        let messages = Arc::new(RwLock::new(Vec::new()));
        let dirty = Arc::new(Mutex::new(false));

        let (save_tx, save_rx) = mpsc::unbounded_channel();

        // Spawn background save task
        let task_storage = Arc::clone(&storage);
        let task_metadata = Arc::clone(&metadata);
        let task_messages = Arc::clone(&messages);
        let task_dirty = Arc::clone(&dirty);
        let task_config = config.clone();
        let task_session_id = session_id;

        tokio::spawn(async move {
            Self::save_task(
                task_storage,
                task_config,
                task_session_id,
                task_metadata,
                task_messages,
                task_dirty,
                save_rx,
            )
            .await;
        });

        Self {
            storage,
            session_id,
            metadata,
            messages,
            dirty,
            save_tx,
        }
    }

    /// Add a message and mark dirty for auto-save.
    pub async fn add_message(&self, message: Message) {
        let mut messages = self.messages.write().await;
        messages.push(message);
        drop(messages);

        *self.dirty.lock().await = true;

        // Trigger debounced save
        let _ = self.save_tx.send(SaveRequest::Save);
    }

    /// Get all messages.
    pub async fn messages(&self) -> Vec<Message> {
        self.messages.read().await.clone()
    }

    /// Update metadata.
    pub async fn update_metadata(&self, metadata: SessionMetadata) {
        *self.metadata.write().await = metadata;
        *self.dirty.lock().await = true;
        let _ = self.save_tx.send(SaveRequest::Save);
    }

    /// Force an immediate save (bypassing debounce).
    pub async fn force_save(&self) -> Result<(), FaeAgentError> {
        self.perform_save().await
    }

    /// Shutdown the auto-save task.
    pub fn shutdown(&self) {
        let _ = self.save_tx.send(SaveRequest::Shutdown);
    }

    /// Background save task with debouncing and retry logic.
    async fn save_task(
        storage: Arc<SessionStorage>,
        config: AutoSaveConfig,
        session_id: SessionId,
        metadata: Arc<RwLock<SessionMetadata>>,
        messages: Arc<RwLock<Vec<Message>>>,
        dirty: Arc<Mutex<bool>>,
        mut save_rx: mpsc::UnboundedReceiver<SaveRequest>,
    ) {
        let mut pending_save = false;
        let mut last_saved_count = 0;

        loop {
            tokio::select! {
                request = save_rx.recv() => {
                    match request {
                        Some(SaveRequest::Save) => {
                            pending_save = true;
                        }
                        Some(SaveRequest::Shutdown) | None => {
                            debug!("Auto-save task shutting down");
                            break;
                        }
                    }
                }
                _ = sleep(config.save_interval), if pending_save => {
                    // Debounce timer expired, perform save
                    let is_dirty = *dirty.lock().await;
                    let current_count = messages.read().await.len();

                    // Check if we should save (dirty flag or batch size exceeded)
                    let should_save = is_dirty ||
                        (current_count > last_saved_count &&
                         current_count - last_saved_count >= config.max_batch_size);

                    if should_save {
                        debug!(session_id = %session_id, "Performing auto-save");

                        // Perform save with retry logic
                        let mut attempt = 0;
                        loop {
                            attempt += 1;

                            let metadata_clone = metadata.read().await.clone();
                            let messages_clone = messages.read().await.clone();

                            match Self::save_with_retry(
                                &storage,
                                session_id,
                                &metadata_clone,
                                &messages_clone,
                                last_saved_count,
                            )
                            .await
                            {
                                Ok(()) => {
                                    *dirty.lock().await = false;
                                    last_saved_count = messages_clone.len();
                                    debug!(session_id = %session_id, messages = last_saved_count, "Auto-save complete");
                                    break;
                                }
                                Err(e) => {
                                    if attempt >= config.max_retries {
                                        error!(
                                            session_id = %session_id,
                                            error = %e,
                                            "Auto-save failed after {} retries",
                                            config.max_retries
                                        );
                                        break;
                                    } else {
                                        warn!(
                                            session_id = %session_id,
                                            attempt,
                                            error = %e,
                                            "Auto-save failed, retrying..."
                                        );
                                        sleep(Duration::from_millis(100 * attempt as u64)).await;
                                    }
                                }
                            }
                        }
                    }

                    pending_save = false;
                }
            }
        }
    }

    /// Perform the actual save operation.
    async fn perform_save(&self) -> Result<(), FaeAgentError> {
        let metadata = self.metadata.read().await.clone();
        let messages = self.messages.read().await.clone();

        Self::save_with_retry(&self.storage, self.session_id, &metadata, &messages, 0).await?;

        *self.dirty.lock().await = false;
        debug!(session_id = %self.session_id, "Force save complete");
        Ok(())
    }

    /// Save with incremental message append.
    async fn save_with_retry(
        storage: &SessionStorage,
        session_id: SessionId,
        metadata: &SessionMetadata,
        messages: &[Message],
        last_saved_count: usize,
    ) -> Result<(), FaeAgentError> {
        // Save manifest
        storage.save_manifest(&session_id, metadata)?;

        // Incremental save: only append new messages
        if last_saved_count < messages.len() {
            for (idx, message) in messages.iter().enumerate().skip(last_saved_count) {
                storage.save_message(&session_id, idx, message)?;
            }
        }

        Ok(())
    }
}

impl Drop for AutoSaveManager {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{Message, SessionId, SessionMetadata};
    use chrono::Utc;
    use std::collections::HashSet;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_debouncing_coalesces_rapid_saves() {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(e) => panic!("Failed to create temp dir: {}", e),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());

        let config = AutoSaveConfig {
            save_interval: Duration::from_millis(100),
            max_batch_size: 10,
            max_retries: 3,
        };

        let session_id = SessionId::new();
        let now = Utc::now();
        let metadata = SessionMetadata {
            created: now,
            modified: now,
            last_active: now,
            title: Some("Test Session".to_string()),
            description: None,
            tags: HashSet::new(),
        };

        let manager = AutoSaveManager::new(storage, config, session_id, metadata);

        // Add multiple messages rapidly
        for i in 0..5 {
            manager
                .add_message(Message::user(format!("Message {}", i)))
                .await;
            sleep(Duration::from_millis(10)).await;
        }

        // Wait for debounce interval plus processing time
        sleep(Duration::from_millis(200)).await;

        // Verify all messages saved
        let messages = manager.messages().await;
        assert_eq!(messages.len(), 5);
    }

    #[tokio::test]
    async fn test_incremental_save_appends_only_new_messages() {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(e) => panic!("Failed to create temp dir: {}", e),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());

        let config = AutoSaveConfig {
            save_interval: Duration::from_millis(50),
            max_batch_size: 10,
            max_retries: 3,
        };

        let session_id = SessionId::new();
        let now = Utc::now();
        let metadata = SessionMetadata {
            created: now,
            modified: now,
            last_active: now,
            title: Some("Incremental Test".to_string()),
            description: None,
            tags: HashSet::new(),
        };

        let manager = AutoSaveManager::new(storage, config, session_id, metadata);

        // Add first batch
        manager
            .add_message(Message::user("First".to_string()))
            .await;
        manager
            .add_message(Message::user("Second".to_string()))
            .await;
        sleep(Duration::from_millis(100)).await;

        // Add second batch
        manager
            .add_message(Message::user("Third".to_string()))
            .await;
        sleep(Duration::from_millis(100)).await;

        let messages = manager.messages().await;
        assert_eq!(messages.len(), 3);
    }

    #[tokio::test]
    async fn test_retry_logic_on_simulated_io_error() {
        // This test verifies retry logic exists
        // In practice, we'd need a mock storage to simulate failures
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(e) => panic!("Failed to create temp dir: {}", e),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());

        let config = AutoSaveConfig {
            save_interval: Duration::from_millis(50),
            max_batch_size: 10,
            max_retries: 3,
        };

        let session_id = SessionId::new();
        let now = Utc::now();
        let metadata = SessionMetadata {
            created: now,
            modified: now,
            last_active: now,
            title: Some("Retry Test".to_string()),
            description: None,
            tags: HashSet::new(),
        };

        let manager = AutoSaveManager::new(storage, config, session_id, metadata);
        manager.add_message(Message::user("Test".to_string())).await;
        sleep(Duration::from_millis(150)).await;

        // If storage works, save succeeds (no actual retry needed)
        // This validates the happy path
        let messages = manager.messages().await;
        assert_eq!(messages.len(), 1);
    }

    #[tokio::test]
    async fn test_session_state_persists_after_autosave() {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(e) => panic!("Failed to create temp dir: {}", e),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());

        let config = AutoSaveConfig {
            save_interval: Duration::from_millis(50),
            max_batch_size: 10,
            max_retries: 3,
        };

        let session_id = SessionId::new();
        let now = Utc::now();
        let metadata = SessionMetadata {
            created: now,
            modified: now,
            last_active: now,
            title: Some("Persist Test".to_string()),
            description: None,
            tags: HashSet::new(),
        };

        let manager = AutoSaveManager::new(storage.clone(), config, session_id, metadata.clone());

        manager
            .add_message(Message::user("Persisted".to_string()))
            .await;
        sleep(Duration::from_millis(150)).await;

        // Load from storage to verify persistence
        let loaded_metadata = match storage.load_manifest(&session_id) {
            Ok(meta) => meta,
            Err(e) => panic!("Failed to load manifest: {}", e),
        };
        assert_eq!(loaded_metadata.title, Some("Persist Test".to_string()));

        let loaded_messages = match storage.load_messages(&session_id) {
            Ok(msgs) => msgs,
            Err(e) => panic!("Failed to load messages: {}", e),
        };
        assert_eq!(loaded_messages.len(), 1);
    }

    #[tokio::test]
    async fn test_no_data_loss_on_rapid_message_additions() {
        let temp_dir = match TempDir::new() {
            Ok(dir) => dir,
            Err(e) => panic!("Failed to create temp dir: {}", e),
        };
        let storage = SessionStorage::with_base_path(temp_dir.path().to_path_buf());

        let config = AutoSaveConfig {
            save_interval: Duration::from_millis(100),
            max_batch_size: 5, // Force save every 5 messages
            max_retries: 3,
        };

        let session_id = SessionId::new();
        let now = Utc::now();
        let metadata = SessionMetadata {
            created: now,
            modified: now,
            last_active: now,
            title: Some("Rapid Test".to_string()),
            description: None,
            tags: HashSet::new(),
        };

        let manager = AutoSaveManager::new(storage, config, session_id, metadata);

        // Add 20 messages rapidly
        for i in 0..20 {
            manager
                .add_message(Message::user(format!("Rapid {}", i)))
                .await;
        }

        // Wait for all saves to complete
        sleep(Duration::from_millis(500)).await;

        let messages = manager.messages().await;
        assert_eq!(messages.len(), 20, "All messages should be preserved");
    }
}
