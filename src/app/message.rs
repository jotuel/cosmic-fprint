use crate::config::Config;
use crate::app::page::ContextPage;
use std::sync::Arc;

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    LaunchUrl(String),
    Delete,
    Register,
    ConnectionReady(zbus::Connection),
    DeviceFound(Option<zbus::zvariant::OwnedObjectPath>),
    OperationError(String),
    EnrollStart(i32),
    EnrollStatus(String, bool),
    EnrollStop,
    DeleteComplete,
    EnrolledFingers(Vec<String>),
    UsersFound(Vec<UserOption>),
    UserSelected(UserOption),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOption {
    pub username: Arc<String>,
    pub realname: Arc<String>,
}

impl std::fmt::Display for UserOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.realname.is_empty() {
            write!(f, "{}", self.username)
        } else {
            write!(f, "{} ({})", self.realname, self.username)
        }
    }
}
