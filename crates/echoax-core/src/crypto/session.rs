use crate::error::{EchoAccessError, Result};
use crate::ui::adapter::UIAdapter;

pub struct SessionManager {
    master_key: Option<[u8; 32]>,
    salt: [u8; 16],
    locked: bool,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            master_key: None,
            salt: super::kdf::generate_salt(),
            locked: true,
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked
    }

    pub fn unlock(&mut self, password: &str) -> Result<()> {
        let key = super::kdf::derive_key(password.as_bytes(), &self.salt)?;
        self.master_key = Some(key);
        self.locked = false;
        Ok(())
    }

    pub fn lock(&mut self) {
        if let Some(ref mut key) = self.master_key {
            key.fill(0);
        }
        self.master_key = None;
        self.locked = true;
    }

    pub fn ensure_unlocked(&mut self, ui: &mut dyn UIAdapter) -> Result<()> {
        if self.is_locked() {
            let password = ui.prompt_password()?;
            self.unlock(&password)?;
        }
        Ok(())
    }

    pub fn get_key(&self) -> Result<&[u8; 32]> {
        self.master_key
            .as_ref()
            .ok_or_else(|| EchoAccessError::Crypto("Session is locked".into()))
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::adapter::MockAdapter;

    #[test]
    fn new_session_is_locked() {
        let session = SessionManager::new();
        assert!(session.is_locked());
    }

    #[test]
    fn unlock_then_locked_false() {
        let mut session = SessionManager::new();
        session.unlock("test-password").unwrap();
        assert!(!session.is_locked());
    }

    #[test]
    fn lock_clears_key() {
        let mut session = SessionManager::new();
        session.unlock("test-password").unwrap();
        assert!(session.get_key().is_ok());
        session.lock();
        assert!(session.is_locked());
        assert!(session.get_key().is_err());
    }

    #[test]
    fn ensure_unlocked_uses_adapter() {
        let mut session = SessionManager::new();
        let mut adapter = MockAdapter;
        session.ensure_unlocked(&mut adapter).unwrap();
        assert!(!session.is_locked());
    }

    #[test]
    fn get_key_when_locked_returns_error() {
        let session = SessionManager::new();
        assert!(session.get_key().is_err());
    }
}
