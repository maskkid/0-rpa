//! Cancellation token for cooperative task cancellation.

use std::sync::Arc;
use tokio::sync::watch;

/// A token that can be used to signal cancellation of a running task.
#[derive(Clone)]
pub struct CancellationToken {
    sender: Arc<watch::Sender<bool>>,
    receiver: watch::Receiver<bool>,
}

impl CancellationToken {
    /// Create a new cancellation token.
    pub fn new() -> Self {
        let (sender, receiver) = watch::channel(false);
        Self {
            sender: Arc::new(sender),
            receiver,
        }
    }

    /// Signal cancellation. All holders of this token will be notified.
    pub fn cancel(&self) {
        let _ = self.sender.send(true);
    }

    /// Check if cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        *self.receiver.borrow()
    }

    /// Get a cloned receiver to check cancellation status.
    pub fn receiver(&self) -> watch::Receiver<bool> {
        self.receiver.clone()
    }

    /// Wait until cancellation is signalled.
    pub async fn cancelled(&mut self) {
        while !*self.receiver.borrow_and_update() {
            if self.receiver.changed().await.is_err() {
                break;
            }
        }
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_token_new() {
        let token = CancellationToken::new();
        assert!(!token.is_cancelled());
    }

    #[test]
    fn cancellation_token_cancel() {
        let token = CancellationToken::new();
        token.cancel();
        assert!(token.is_cancelled());
    }

    #[test]
    fn cancellation_token_clone() {
        let token = CancellationToken::new();
        let cloned = token.clone();
        token.cancel();
        assert!(cloned.is_cancelled());
    }
}