use crate::config::Config;
use crate::app::page::ContextPage;

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
    EnrollStopSuccess,
    DeleteComplete,
    DeleteFailed(String),
}
