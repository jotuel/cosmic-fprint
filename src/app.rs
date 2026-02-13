// SPDX-License-Identifier: MPL-2.0

use crate::config::Config;
use crate::fl;
use crate::fprint_dbus::{DeviceProxy, ManagerProxy};
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar, text};
use cosmic::{cosmic_theme, theme};
use futures_util::sink::Sink;
use futures_util::{SinkExt, StreamExt};
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
    // Shared DBus connection
    connection: Option<zbus::Connection>,
    // Whether an operation is in progress
    busy: bool,
    // Finger currently being enrolled (None if not enrolling)
    enrolling_finger: Option<String>,
    // Enrollment progress
    enroll_progress: i32,
    enroll_total_stages: i32,
}

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
    EnrollComplete,
    EnrollStop,
    EnrollStopSuccess,
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

        for page in Page::all() {
            nav.insert()
                .text(fl!("page-id", name = page.display_name()))
                .data::<Page>(*page)
                .icon(icon::from_name("applications-utilities-symbolic"));
        }

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
                    Err((errors, config)) => {
                        for why in errors {
                            tracing::error!(%why, "error loading app config");
                        }

                        config
                    }
                })
                .unwrap_or_default(),
            status: "Connecting to system bus...".to_string(),
            device_path: None,
            connection: None,
            busy: true,
            enrolling_finger: None,
            enroll_progress: 0,
            enroll_total_stages: 0,
        };

        // Create a startup command that sets the window title.
        let command = app.update_title();

        // Start async task to connect to DBus
        let connect_task = Task::perform(
            async move {
                match zbus::Connection::system().await {
                    Ok(conn) => Message::ConnectionReady(conn),
                    Err(e) => Message::OperationError(format!("Failed to connect to DBus: {}", e)),
                }
            },
            cosmic::Action::App,
        );

        (app, command.chain(connect_task))
    }

    /// Elements to pack at the start of the header bar.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let menu_bar = menu::bar(vec![menu::Tree::with_children(
            Element::from(menu::root(fl!("view"))),
            menu::items(
                &self.key_binds,
                vec![menu::Item::Button(fl!("about"), None, MenuAction::About)],
            ),
        )]);

        vec![menu_bar.into()]
    }

    /// Enables the COSMIC application to create a nav bar with this model.
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav)
    }

    /// Display a context drawer if the context page is requested.
    fn context_drawer(&self) -> Option<context_drawer::ContextDrawer<'_, Self::Message>> {
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
    fn view(&self) -> Element<'_, Self::Message> {
        let buttons_enabled =
            !self.busy && self.device_path.is_some() && self.enrolling_finger.is_none();

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

        let mut cancel_btn = widget::button::text(fl!("cancel"));
        if self.enrolling_finger.is_some() {
            cancel_btn = cancel_btn.on_press(Message::EnrollStop);
        }

        let mut column = widget::column()
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
                    .align_x(Horizontal::Center),
            );

        if self.enrolling_finger.is_some() && self.enroll_total_stages > 0 {
            column = column.push(
                widget::progress_bar(
                    0.0..=(self.enroll_total_stages as f32),
                    self.enroll_progress as f32,
                )
                .height(10),
            );
        }

        let mut row = widget::row()
            .push(register_btn)
            .push(delete_btn);

        if self.enrolling_finger.is_some() {
            row = row.push(cancel_btn);
        }

        column
            .push(
                row.apply(widget::container)
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
                cosmic::iced::stream::channel(4, move |_channel| async move {
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
        if let (Some(finger_name), Some(device_path), Some(connection)) =
            (&self.enrolling_finger, &self.device_path, &self.connection)
        {
            let finger_name = finger_name.clone();
            let device_path = device_path.clone();
            let connection = connection.clone();

            subscriptions.push(Subscription::run_with_id(
                std::any::TypeId::of::<EnrollmentSubscription>(),
                cosmic::iced::stream::channel(100, move |mut output| async move {
                    // Implement enrollment stream here
                    match enroll_fingerprint_process(
                        connection,
                        device_path,
                        finger_name,
                        &mut output,
                    )
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            let _ = output.send(Message::OperationError(e.to_string())).await;
                        }
                    }
                    futures_util::future::pending().await
                }),
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
            Message::ConnectionReady(conn) => {
                self.connection = Some(conn.clone());
                self.status = "Searching for fingerprint reader...".to_string();

                return Task::perform(
                    async move {
                        match find_device(&conn).await {
                            Ok(path) => Message::DeviceFound(Some(path)),
                            Err(e) => {
                                Message::OperationError(format!("Failed to find device: {}", e))
                            }
                        }
                    },
                    cosmic::Action::App,
                );
            }

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

            Message::EnrollStart(total) => {
                self.enroll_total_stages = total;
                self.enroll_progress = 0;
                self.status = fl!("enroll-starting");
            }

            Message::EnrollStatus(status, done) => {
                let status_msg = match status.as_str() {
                    "enroll-stage-passed" => {
                        self.enroll_progress += 1;
                        fl!("enroll-stage-passed")
                    }
                    "enroll-retry-scan" => fl!("enroll-retry-scan"),
                    "enroll-swipe-too-short" => fl!("enroll-swipe-too-short"),
                    "enroll-finger-not-centered" => fl!("enroll-finger-not-centered"),
                    "enroll-remove-and-retry" => fl!("enroll-remove-and-retry"),
                    "enroll-unknown-error" => fl!("enroll-unknown-error"),
                    "enroll-completed" => fl!("enroll-completed"),
                    "enroll-failed" => fl!("enroll-failed"),
                    "enroll-disconnected" => fl!("enroll-disconnected"),
                    "enroll-data-full" => fl!("enroll-data-full"),
                    _ => status.clone(),
                };
                self.status = status_msg;

                if done {
                    self.status = fl!("enroll-completed");
                    self.busy = false;
                    self.enrolling_finger = None;
                }
            }

            Message::EnrollComplete => {
                self.status = fl!("enroll-completed");
                self.busy = false;
                self.enrolling_finger = None;
            }

            Message::EnrollStop => {
                if let (Some(path), Some(conn)) =
                    (self.device_path.clone(), self.connection.clone())
                {
                    return Task::perform(
                        async move {
                            let device = DeviceProxy::builder(&conn).path(path)?.build().await?;
                            device.enroll_stop().await?;
                            Ok::<(), zbus::Error>(())
                        },
                        |res| match res {
                            Ok(_) => cosmic::Action::App(Message::EnrollStopSuccess),
                            Err(e) => cosmic::Action::App(Message::OperationError(e.to_string())),
                        },
                    );
                }
            }

            Message::EnrollStopSuccess => {
                self.busy = false;
                self.enrolling_finger = None;
                self.status = fl!("enroll-completed");

                if let (Some(path), Some(conn)) =
                    (self.device_path.clone(), self.connection.clone())
                {
                    return Task::perform(
                        async move {
                            let device = DeviceProxy::builder(&conn).path(path)?.build().await?;
                            device.release().await?;
                            Ok::<(), zbus::Error>(())
                        },
                        |res| match res {
                            Ok(_) => cosmic::Action::App(Message::EnrollComplete),
                            Err(e) => cosmic::Action::App(Message::OperationError(e.to_string())),
                        },
                    );
                }
            }

            Message::DeleteComplete => {
                self.status = "Fingerprint was deleted.".to_string();
                self.busy = false;
            }

            Message::Delete => {
                if let Some(page) = self.nav.data::<Page>(self.nav.active()) {
                    if let (Some(path), Some(conn)) =
                        (self.device_path.clone(), self.connection.clone())
                    {
                        self.busy = true;
                        self.status = "Deleting fingerprints...".to_string();
                        let finger_name = page.as_finger_id().to_string();
                        return Task::perform(
                            async move {
                                match delete_fingerprint_dbus(&conn, path, finger_name).await {
                                    Ok(_) => Message::DeleteComplete,
                                    Err(e) => Message::OperationError(e.to_string()),
                                }
                            },
                            |m| cosmic::Action::App(m),
                        );
                    }
                }
            }

            Message::Register => {
                if let Some(page) = self.nav.data::<Page>(self.nav.active()) {
                    if let Some(_) = &self.device_path {
                        self.busy = true;
                        self.status = "Starting enrollment...".to_string();
                        self.enrolling_finger = Some(page.as_finger_id().to_string());
                    }
                }
            }

            Message::OpenRepositoryUrl => {
                let _ = open::that_detached(REPOSITORY);
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
    pub fn about(&self) -> Element<'_, Message> {
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

async fn find_device(
    connection: &zbus::Connection,
) -> zbus::Result<zbus::zvariant::OwnedObjectPath> {
    let manager = ManagerProxy::new(&connection).await?;
    let device = manager.get_default_device().await?;
    Ok(device)
}

async fn delete_fingerprint_dbus(
    connection: &zbus::Connection,
    path: zbus::zvariant::OwnedObjectPath,
    finger: String,
) -> zbus::Result<()> {
    let device = DeviceProxy::builder(connection).path(path)?.build().await?;

    device.claim("").await?;
    device.delete_enrolled_finger(&finger).await?;
    device.release().await?;
    Ok(())
}

async fn enroll_fingerprint_process<S>(
    connection: zbus::Connection,
    path: zbus::zvariant::OwnedObjectPath,
    finger_name: String,
    output: &mut S,
) -> zbus::Result<()>
where
    S: Sink<Message> + Unpin + Send,
    S::Error: std::fmt::Debug + Send,
{
    let device = DeviceProxy::builder(&connection)
        .path(path)?
        .build()
        .await?;

    // Claim device
    match device.claim("").await {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    let total_stages = device.num_enroll_stages().await.unwrap_or(-1);
    let _ = output.send(Message::EnrollStart(total_stages)).await;

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
                let _ = output
                    .send(Message::EnrollStatus(result.clone(), done))
                    .await;

                if done {
                    break;
                }
            }
            Err(_) => {
                let _ = output
                    .send(Message::OperationError(
                        "Failed to parse signal".to_string(),
                    ))
                    .await;
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
    pub fn all() -> &'static [Self] {
        &[
            Self::RightThumb,
            Self::RightIndex,
            Self::RightMiddle,
            Self::RightRing,
            Self::RightPinky,
            Self::LeftThumb,
            Self::LeftIndex,
            Self::LeftMiddle,
            Self::LeftRing,
            Self::LeftPinky,
        ]
    }

    fn display_name(&self) -> &'static str {
        match self {
            Self::RightThumb => "Right Thumb",
            Self::RightIndex => "Right Index",
            Self::RightMiddle => "Right Middle",
            Self::RightRing => "Right Ring",
            Self::RightPinky => "Right Pinky",
            Self::LeftThumb => "Left Thumb",
            Self::LeftIndex => "Left Index",
            Self::LeftMiddle => "Left Middle",
            Self::LeftRing => "Left Ring",
            Self::LeftPinky => "Left Pinky",
        }
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_all() {
        let pages = Page::all();
        assert_eq!(pages.len(), 10);
        assert_eq!(pages[0], Page::RightThumb);
        assert_eq!(pages[1], Page::RightIndex);
        assert_eq!(pages[2], Page::RightMiddle);
        assert_eq!(pages[3], Page::RightRing);
        assert_eq!(pages[4], Page::RightPinky);
        assert_eq!(pages[5], Page::LeftThumb);
        assert_eq!(pages[6], Page::LeftIndex);
        assert_eq!(pages[7], Page::LeftMiddle);
        assert_eq!(pages[8], Page::LeftRing);
        assert_eq!(pages[9], Page::LeftPinky);
    }

    #[test]
    fn test_page_display_name() {
        assert_eq!(Page::RightThumb.display_name(), "Right Thumb");
        assert_eq!(Page::RightIndex.display_name(), "Right Index");
        assert_eq!(Page::RightMiddle.display_name(), "Right Middle");
        assert_eq!(Page::RightRing.display_name(), "Right Ring");
        assert_eq!(Page::RightPinky.display_name(), "Right Pinky");
        assert_eq!(Page::LeftThumb.display_name(), "Left Thumb");
        assert_eq!(Page::LeftIndex.display_name(), "Left Index");
        assert_eq!(Page::LeftMiddle.display_name(), "Left Middle");
        assert_eq!(Page::LeftRing.display_name(), "Left Ring");
        assert_eq!(Page::LeftPinky.display_name(), "Left Pinky");
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
