use crate::error::Result;

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct AppState {
    pub status_message: String,
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct DiffView {
    pub left: String,
    pub right: String,
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ConflictInfo {
    pub path: String,
    pub description: String,
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub enum Resolution {
    #[default]
    KeepLocal,
    AcceptRemote,
    Merge,
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct PendingAction {
    pub description: String,
}

#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct Notification {
    pub title: String,
    pub body: String,
}

pub trait UIAdapter: Send + Sync {
    fn show_status(&mut self, state: &AppState) -> Result<()>;
    fn show_diff(&mut self, diff: &DiffView) -> Result<()>;
    fn prompt_conflict_resolution(&mut self, conflict: &ConflictInfo) -> Result<Resolution>;
    fn prompt_password(&mut self) -> Result<String>;
    fn confirm_action(&mut self, action: &PendingAction) -> Result<bool>;
    fn show_notification(&mut self, msg: &Notification) -> Result<()>;
    fn show_progress(&mut self, label: &str, current: u64, total: u64) -> Result<()>;
}

pub struct MockAdapter;

impl UIAdapter for MockAdapter {
    fn show_status(&mut self, _state: &AppState) -> Result<()> {
        Ok(())
    }

    fn show_diff(&mut self, _diff: &DiffView) -> Result<()> {
        Ok(())
    }

    fn prompt_conflict_resolution(&mut self, _conflict: &ConflictInfo) -> Result<Resolution> {
        Ok(Resolution::KeepLocal)
    }

    fn prompt_password(&mut self) -> Result<String> {
        Ok(String::new())
    }

    fn confirm_action(&mut self, _action: &PendingAction) -> Result<bool> {
        Ok(true)
    }

    fn show_notification(&mut self, _msg: &Notification) -> Result<()> {
        Ok(())
    }

    fn show_progress(&mut self, _label: &str, _current: u64, _total: u64) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_adapter_show_status() {
        let mut adapter = MockAdapter;
        let state = AppState {
            status_message: "running".into(),
            ..Default::default()
        };
        assert!(adapter.show_status(&state).is_ok());
    }

    #[test]
    fn mock_adapter_show_diff() {
        let mut adapter = MockAdapter;
        let diff = DiffView {
            left: "old".into(),
            right: "new".into(),
            ..Default::default()
        };
        assert!(adapter.show_diff(&diff).is_ok());
    }

    #[test]
    fn mock_adapter_prompt_conflict_resolution() {
        let mut adapter = MockAdapter;
        let conflict = ConflictInfo {
            path: "/tmp/file".into(),
            description: "merge conflict".into(),
            ..Default::default()
        };
        let res = adapter.prompt_conflict_resolution(&conflict).unwrap();
        assert!(matches!(res, Resolution::KeepLocal));
    }

    #[test]
    fn mock_adapter_prompt_password() {
        let mut adapter = MockAdapter;
        let pwd = adapter.prompt_password().unwrap();
        assert!(pwd.is_empty());
    }

    #[test]
    fn mock_adapter_confirm_action() {
        let mut adapter = MockAdapter;
        let action = PendingAction {
            description: "delete all".into(),
            ..Default::default()
        };
        assert!(adapter.confirm_action(&action).unwrap());
    }

    #[test]
    fn mock_adapter_show_notification() {
        let mut adapter = MockAdapter;
        let notif = Notification {
            title: "Alert".into(),
            body: "Something happened".into(),
            ..Default::default()
        };
        assert!(adapter.show_notification(&notif).is_ok());
    }

    #[test]
    fn mock_adapter_show_progress() {
        let mut adapter = MockAdapter;
        assert!(adapter.show_progress("loading", 50, 100).is_ok());
    }

    #[test]
    fn mock_adapter_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockAdapter>();
    }

    #[test]
    fn mock_adapter_as_trait_object() {
        let mut adapter: Box<dyn UIAdapter> = Box::new(MockAdapter);
        assert!(adapter.show_progress("test", 0, 1).is_ok());
    }
}
