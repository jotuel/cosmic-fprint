// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::app::page::ContextPage;
use std::sync::Arc;
use crate::app::error::AppError;

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
    OperationError(AppError),
    EnrollStart(Option<u32>),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_user_option_display_with_realname() {
        let user_option = UserOption {
            username: Arc::new("jdoe".to_string()),
            realname: Arc::new("John Doe".to_string()),
        };
        assert_eq!(user_option.to_string(), "John Doe (jdoe)");
    }

    #[test]
    fn test_user_option_display_without_realname() {
        let user_option = UserOption {
            username: Arc::new("jdoe".to_string()),
            realname: Arc::new("".to_string()),
        };
        assert_eq!(user_option.to_string(), "jdoe");
    }

    #[test]
    fn test_user_option_display_with_whitespace_realname() {
        let user_option = UserOption {
            username: Arc::new("jdoe".to_string()),
            realname: Arc::new("   ".to_string()),
        };
        assert_eq!(user_option.to_string(), "    (jdoe)");
    }

    #[test]
    fn test_user_option_display_empty_username() {
        let user_option = UserOption {
            username: Arc::new("".to_string()),
            realname: Arc::new("John Doe".to_string()),
        };
        assert_eq!(user_option.to_string(), "John Doe ()");
    }

    #[test]
    fn test_user_option_display_both_empty() {
        let user_option = UserOption {
            username: Arc::new("".to_string()),
            realname: Arc::new("".to_string()),
        };
        assert_eq!(user_option.to_string(), "");
    }
}
