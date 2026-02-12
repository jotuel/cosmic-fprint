// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use crate::fprint_dbus::{ManagerProxy, DeviceProxy};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar, text};
use cosmic::{cosmic_theme, theme};
use futures_util::{SinkExt, StreamExt};
use futures_util::sink::Sink;
use std::collections::HashMap;

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../resources/icons/hicolor/scalable/apps/icon.svg");

/// The application model stores app-specific state used to describe its interface and
/// drive its logic.
pub struct AppModel {
    /// Application state which is managed by the COSMIC runtime.
    core: cosmic::Core,
    /// Display a context drawer with the designated page if defined.
    context_page: ContextPage,
    /// Contains items assigned to the nav bar panel.
    nav: nav_bar::Model,
    /// Key bindings for the application's menu bar.
    key_binds: HashMap<menu::KeyBind, MenuAction>,
    // Configuration data that persists between application runs.
    config: Config,
    // Status text for the UI
    status: String,
    // Currently selected device path
    device_path: Option<zbus::zvariant::OwnedObjectPath>,
    // Whether an operation is in progress
    busy: bool,
    // Finger currently being enrolled (None if not enrolling)
    enrolling_finger: Option<String>,
}

/// Messages emitted by the application and its widgets.
#[derive(Debug, Clone)]
pub enum Message {
    OpenRepositoryUrl,
    SubscriptionChannel,
    ToggleContextPage(ContextPage),
    UpdateConfig(Config),
    LaunchUrl(String),
    Delete,
    Register,
    Feedback(String),
    // Async Operation Messages
    DeviceFound(Option<zbus::zvariant::OwnedObjectPath>),
    OperationError(String),
    EnrollStatus(String, bool),
    EnrollComplete,
    EnrollStop, // Used to manually stop or when done
    DeleteComplete,
}

/// Create a COSMIC application from the app model
impl cosmic::Application for AppModel {
    /// The async executor that will be used to run your application's commands.
    type Executor = cosmic::executor::Default;

    /// Data that your application receives to its init method.
    type Flags = ();

    /// Messages which the application and its widgets will emit.
    type Message = Message;

    /// Unique identifier in RDNN (reverse domain name notation) format.
    const APP_ID: &'static str = "fi.joonastuomi.CosmicFprint";

    fn core(&self) -> &cosmic::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::Core {
        &mut self.core
    }

    /// Initializes the application with any given flags and startup commands.
    fn init(
        core: cosmic::Core,
        _flags: Self::Flags,
    ) -> (Self, Task<cosmic::Action<Self::Message>>) {
        // Create a nav bar for every fingerprint
        let mut nav = nav_bar::Model::default();

        nav.insert()
            .text(fl!("page-id", name = "Right Thumb"))
            .data::<Page>(Page::RightThumb)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Right Index"))
            .data::<Page>(Page::RightIndex)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Right Middle"))
            .data::<Page>(Page::RightMiddle)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Right Ring"))
            .data::<Page>(Page::RightRing)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Right Pinky"))
            .data::<Page>(Page::RightPinky)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Left Thumb"))
            .data::<Page>(Page::LeftThumb)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Left Index"))
            .data::<Page>(Page::LeftIndex)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Left Middle"))
            .data::<Page>(Page::LeftMiddle)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Left Ring"))
            .data::<Page>(Page::LeftRing)
            .icon(icon::from_name("applications-utilities-symbolic"));

        nav.insert()
            .text(fl!("page-id", name = "Left Pinky"))
            .data::<Page>(Page::LeftPinky)
            .icon(icon::from_name("applications-utilities-symbolic"));

        // Construct the app model with the runtime's core.
        let mut app = AppModel {
            core,
            context_page: ContextPage::default(),
            nav,
            key_binds: HashMap::new(),
            // Optional configuration file for an application.
            config: cosmic_config::Config::new(Self::APP_ID, Config::VERSION)
                .map(|context| match Config::get_entry(&context) {
                    Ok(config) => config,
                    Err((_errors, config)) => {
                        // for why in errors {
                        //     tracing::error!(%why, "error loading app config");
                        // }

                        config
                    }
                })
                .unwrap_or_default(),
            status: "Searching for fingerprint reader...".to_string(),
            device_path: None,
            busy: true,
            enrolling_finger: None,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        // Start async task to find device
        let find_device_task = Task::perform(async move {
            match find_device().await {
                Ok(path) => Message::DeviceFound(Some(path)),
                Err(e) => Message::OperationError(format!("Failed to find device: {}", e)),
            }
        }, |m| cosmic::Action::App(m));

        (app, command.chain(find_device_task))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        // Commented out due to trait bound issues in libcosmic update
        /*
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            menu::root(fl!("view")),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
        */
        vec![]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        Some(match self.context_page {
            ContextPage::About => context_drawer::context_drawer(
                self.about(),
                Message::ToggleContextPage(ContextPage::About),
            )
            .title(fl!("about")),
        })
    }

    /// Describes the interface based on the current state of the application model.
    ///
    /// Application events will be processed through the view. Any messages emitted by
    /// events received by widgets will be passed to the update method.
    fn view(&self) -> Element<Self::Message> {
        let buttons_enabled = !self.busy && self.device_path.is_some() && self.enrolling_finger.is_none();

        let register_btn = widget::button::text(fl!("register"));
        let delete_btn = widget::button::text(fl!("delete"));

        let register_btn = if buttons_enabled {
            register_btn.on_press(Message::Register)
        } else {
            register_btn
        };

        let delete_btn = if buttons_enabled {
            delete_btn.on_press(Message::Delete)
        } else {
            delete_btn
        };

        widget::column()
            .push(
                text::title1(fl!("fprint"))
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center),
            )
            .push(
                widget::svg(widget::svg::Handle::from_path(std::path::PathBuf::from(
                    "resources/icons/hicolor/scalable/apps/fprint.svg",
                )))
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .push(
                widget::text(&self.status)
                    .size(16)
                    .apply(widget::container)
                    .width(Length::Fill)
                    .align_x(Horizontal::Center)
            )
            .push(
                widget::row()
                    .push(register_btn)
                    .push(delete_btn)
                    .apply(widget::container)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .align_x(Horizontal::Center)
                    .align_y(Vertical::Center)
                    .padding(20),
            )
            .align_x(Horizontal::Center)
            .spacing(20)
            .padding(20)
            .into()
    }

    /// Register subscriptions for this application.
    ///
    /// Subscriptions are long-running async tasks running in the background which
    /// emit messages to the application through a channel. They are started at the
    /// beginning of the application, and persist through its lifetime.
    fn subscription(&self) -> Subscription<Self::Message> {
        struct MySubscription;
        struct EnrollmentSubscription;

        let mut subscriptions = vec![
            // Create a subscription which emits updates through a channel.
            Subscription::run_with_id(
                std::any::TypeId::of::<MySubscription>(),
                cosmic::iced::stream::channel(4, move |mut channel| async move {
                    _ = channel.send(Message::SubscriptionChannel).await;

                    futures_util::future::pending().await
                }),
            ),
            // Watch for application configuration changes.
            self.core()
                .watch_config::<Config>(Self::APP_ID)
                .map(|update| {
                    // for why in update.errors {
                    //     tracing::error!(?why, "app config error");
                    // }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Add enrollment subscription if enrolling
        if let (Some(finger_name), Some(device_path)) = (&self.enrolling_finger, &self.device_path) {
             let finger_name = finger_name.clone();
             let device_path = device_path.clone();

             subscriptions.push(Subscription::run_with_id(
                std::any::TypeId::of::<EnrollmentSubscription>(),
                cosmic::iced::stream::channel(100, move |mut output| async move {
                    // Implement enrollment stream here
                    match enroll_fingerprint_process(device_path, finger_name, &mut output).await {
                         Ok(_) => {},
                         Err(e) => {
                             let _ = output.send(Message::OperationError(e.to_string())).await;
                         }
                    }
                    futures_util::future::pending().await
                })
             ));
        }

        Subscription::batch(subscriptions)
    }

    /// Handles messages emitted by the application and its widgets.
    ///
    /// Tasks may be returned for asynchronous execution of code in the background
    /// on the application's async runtime.
    fn update(&mut self, message: Self::Message) -> Task<cosmic::Action<Self::Message>> {
        match message {
            Message::DeviceFound(path) => {
                self.device_path = path;
                if self.device_path.is_some() {
                    self.status = "Device found. Ready.".to_string();
                    self.busy = false;
                } else {
                    self.status = "No fingerprint reader found.".to_string();
                    self.busy = true;
                }
            }
            Message::OperationError(err) => {
                self.status = format!("Error: {}", err);
                self.busy = false;
                self.enrolling_finger = None;
            }
            Message::EnrollStatus(status, done) => {
                self.status = format!("Enrolling: {}", status);
                if done {
                    self.status = "Enrollment completed successfully.".to_string();
                    self.busy = false;
                    self.enrolling_finger = None;
                }
            }
            Message::EnrollComplete => {
                 self.status = "Enrollment completed.".to_string();
                 self.busy = false;
                 self.enrolling_finger = None;
            }
            Message::EnrollStop => {
                 self.busy = false;
                 self.enrolling_finger = None;
            }
            Message::DeleteComplete => {
                self.status = "Fingerprints deleted.".to_string();
                self.busy = false;
            }
            Message::Delete => {
                if let Some(page) = self.nav.data::<Page>(self.nav.active()) {
                    if let Some(path) = self.device_path.clone() {
                         self.busy = true;
                         self.status = "Deleting fingerprints...".to_string();
                         let finger_name = page.as_finger_id().to_string();
                         return Task::perform(async move {
                             match delete_fingerprint_dbus(path, finger_name).await {
                                 Ok(_) => Message::DeleteComplete,
                                 Err(e) => Message::OperationError(e.to_string()),
                             }
                         }, |m| cosmic::Action::App(m));
                    }
                }
            }
            Message::Register => {
                if let Some(page) = self.nav.data::<Page>(self.nav.active()) {
                    if let Some(_) = &self.device_path {
                         self.busy = true;
                         self.status = "Starting enrollment...".to_string();
                         self.enrolling_finger = Some(page.as_finger_id().to_string());
                         // The subscription will pick this up automatically
                    }
                }
            }
            Message::OpenRepositoryUrl => {
                let _ = open::that_detached(REPOSITORY);
            }

            Message::SubscriptionChannel => {
                // For example purposes only.
            }

            Message::ToggleContextPage(context_page) => {
                if self.context_page == context_page {
                    // Close the context drawer if the toggled context page is the same.
                    self.core.window.show_context = !self.core.window.show_context;
                } else {
                    // Open the context drawer to display the requested context page.
                    self.context_page = context_page;
                    self.core.window.show_context = true;
                }
            }

            Message::Feedback(feedback) => {
                self.status = feedback;
            }

            Message::UpdateConfig(config) => {
                self.config = config;
            }

            Message::LaunchUrl(url) => match open::that_detached(&url) {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("failed to open {url:?}: {err}");
                }
            },
        }
        Task::none()
    }

    /// Called when a nav item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<cosmic::Action<Self::Message>> {
        // Activate the page in the model.
        self.nav.activate(id);

        self.update_title()
    }
}

impl AppModel {
    /// The about page for this app.
    pub fn about(&self) -> Element<Message> {
        let cosmic_theme::Spacing { space_xxs, .. } = theme::active().cosmic().spacing;

        let icon = widget::svg(widget::svg::Handle::from_memory(APP_ICON));

        let title = text::title3(fl!("app-title"));

        let hash = env!("VERGEN_GIT_SHA");
        let short_hash: String = hash.chars().take(7).collect();
        let date = env!("VERGEN_GIT_COMMIT_DATE");

        let link = widget::button::link(REPOSITORY)
            .on_press(Message::OpenRepositoryUrl)
            .padding(0);

        widget::column()
            .push(icon)
            .push(title)
            .push(link)
            .push(
                widget::button::link(fl!(
                    "git-description",
                    hash = short_hash.as_str(),
                    date = date
                ))
                .on_press(Message::LaunchUrl(format!("{REPOSITORY}/commits/{hash}")))
                .padding(0),
            )
            .align_x(Alignment::Center)
            .spacing(space_xxs)
            .into()
    }

    /// Updates the header and window titles.
    pub fn update_title(&mut self) -> Task<cosmic::Action<Message>> {
        let mut window_title = fl!("app-title");

        if let Some(page) = self.nav.text(self.nav.active()) {
            window_title.push_str(" — ");
            window_title.push_str(page);
        }

        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}


async fn find_device() -> zbus::Result<zbus::zvariant::OwnedObjectPath> {
    let connection = zbus::Connection::system().await?;
    let manager = ManagerProxy::new(&connection).await?;
    let device = manager.get_default_device().await?;
    Ok(device)
}

async fn delete_fingerprint_dbus(path: zbus::zvariant::OwnedObjectPath, _finger: String) -> zbus::Result<()> {
    let connection = zbus::Connection::system().await?;
    let device = DeviceProxy::builder(&connection).path(path)?.build().await?;

    device.claim("").await?;
    device.delete_enrolled_fingers("").await?;
    device.release().await?;
    Ok(())
}

async fn enroll_fingerprint_process<S>(
    path: zbus::zvariant::OwnedObjectPath,
    finger_name: String,
    output: &mut S
) -> zbus::Result<()>
where S: Sink<Message> + Unpin + Send,
      S::Error: std::fmt::Debug + Send
{
    let connection = zbus::Connection::system().await?;
    let device = DeviceProxy::builder(&connection).path(path)?.build().await?;

    // Claim device
    match device.claim("").await {
        Ok(_) => {},
        Err(e) => return Err(e),
    };

    // Start enrollment
    if let Err(e) = device.enroll_start(&finger_name).await {
        let _ = device.release().await;
        return Err(e);
    }

    // Listen for signals
    let mut stream = device.receive_enroll_status().await?;

    while let Some(signal) = stream.next().await {
        let args = signal.args();
        match args {
            Ok(args) => {
                 let result: String = args.result;
                 let done: bool = args.done;

                 // Map result string to user friendly message if needed, or pass through
                 let _ = output.send(Message::EnrollStatus(result.clone(), done)).await;

                 if done {
                     break;
                 }
            },
            Err(_) => {
                let _ = output.send(Message::OperationError("Failed to parse signal".to_string())).await;
                break;
            }
        }
    }

    // Release device
    let _ = device.release().await;

    let _ = output.send(Message::EnrollComplete).await;

    Ok(())
}

/// The page to display in the application.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    RightThumb,
    #[default]
    RightIndex,
    RightMiddle,
    RightRing,
    RightPinky,
    LeftThumb,
    LeftIndex,
    LeftMiddle,
    LeftRing,
    LeftPinky,
}

impl Page {
    fn as_finger_id(&self) -> &'static str {
        match self {
            Page::RightThumb => "right-thumb",
            Page::RightIndex => "right-index-finger",
            Page::RightMiddle => "right-middle-finger",
            Page::RightRing => "right-ring-finger",
            Page::RightPinky => "right-little-finger",
            Page::LeftThumb => "left-thumb",
            Page::LeftIndex => "left-index-finger",
            Page::LeftMiddle => "left-middle-finger",
            Page::LeftRing => "left-ring-finger",
            Page::LeftPinky => "left-little-finger",
        }
    }
}

/// The context page to display in the context drawer.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ContextPage {
    #[default]
    About,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MenuAction {
    About,
}

impl menu::action::MenuAction for MenuAction {
    type Message = Message;

    fn message(&self) -> Self::Message {
        match self {
            MenuAction::About => Message::ToggleContextPage(ContextPage::About),
        }
    }
}

