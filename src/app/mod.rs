// SPDX-License-Identifier: MPL-2.0

use crate::accounts_dbus::{AccountsProxy, UserProxy};
use crate::config::Config;
use crate::fl;
use crate::fprint_dbus::DeviceProxy;
use cosmic::app::context_drawer;
use cosmic::cosmic_config::{self, CosmicConfigEntry};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::widget::pick_list;
use cosmic::iced::{Alignment, Length, Subscription};
use cosmic::prelude::*;
use cosmic::widget::{self, icon, menu, nav_bar, text};
use cosmic::{cosmic_theme, theme};
use futures_util::SinkExt;
use std::collections::HashMap;

pub mod page;
pub mod message;
pub mod fprint;

use page::{ContextPage, Page};
use message::{Message, UserOption};
use fprint::{delete_fingerprint_dbus, enroll_fingerprint_process, find_device};

const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const APP_ICON: &[u8] = include_bytes!("../../resources/icons/hicolor/scalable/apps/icon.svg");

const STATUS_TEXT_SIZE: u16 = 16;
const PROGRESS_BAR_HEIGHT: u16 = 10;
const MAIN_SPACING: u16 = 20;
const MAIN_PADDING: u16 = 20;

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
    // List of users (username, realname)
    users: Vec<UserOption>,
    // Selected user
    selected_user: Option<UserOption>,
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
                .text(page.localized_name())
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
            users: Vec::new(),
            selected_user: std::env::var("USER").ok().map(|u| UserOption {
                username: u.clone(),
                realname: String::new(),
            }),
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

        let mut column = widget::column().push(
            text::title1(fl!("fprint"))
                .apply(widget::container)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Center),
        );

        if !self.users.is_empty() {
            column = column.push(
                pick_list(
                    self.users.as_slice(),
                    self.selected_user.clone(),
                    Message::UserSelected,
                )
                .width(Length::Fixed(200.0))
                .apply(widget::container)
                .width(Length::Fill)
                .align_x(Horizontal::Center),
            );
        }

        column = column
            .push(
                widget::svg(widget::svg::Handle::from_path(std::path::PathBuf::from(
                    "resources/icons/hicolor/scalable/apps/fprint.svg",
                )))
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .push(
                widget::text(&self.status)
                    .size(STATUS_TEXT_SIZE)
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
                .height(PROGRESS_BAR_HEIGHT),
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
                    .padding(MAIN_PADDING),
            )
            .align_x(Horizontal::Center)
            .spacing(MAIN_SPACING)
            .padding(MAIN_PADDING)
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
                    for why in update.errors {
                        tracing::error!(?why, "app config error");
                    }

                    Message::UpdateConfig(update.config)
                }),
        ];

        // Add enrollment subscription if enrolling
        if let (Some(finger_name), Some(device_path), Some(connection), Some(user)) = (
            &self.enrolling_finger,
            &self.device_path,
            &self.connection,
            &self.selected_user,
        ) {
            let finger_name = finger_name.clone();
            let device_path = device_path.clone();
            let connection = connection.clone();
            let username = user.username.clone();

            subscriptions.push(Subscription::run_with_id(
                std::any::TypeId::of::<EnrollmentSubscription>(),
                cosmic::iced::stream::channel(100, move |mut output| async move {
                    // Implement enrollment stream here
                    match enroll_fingerprint_process(
                        connection,
                        device_path,
                        finger_name,
                        username,
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

                let conn_clone = conn.clone();
                let find_device_task = Task::perform(
                    async move {
                        match find_device(&conn_clone).await {
                            Ok(path) => Message::DeviceFound(Some(path)),
                            Err(e) => Message::OperationError(format!("Failed to find device: {}", e)),
                        }
                    },
                    cosmic::Action::App,
                );

                let conn_clone = conn.clone();
                let fetch_users_task = Task::perform(
                    async move {
                        let mut users = Vec::new();
                        // Try to get users from AccountsService
                        if let Ok(accounts) = AccountsProxy::new(&conn_clone).await {
                            if let Ok(user_paths) = accounts.list_cached_users().await {
                                for path in user_paths {
                                    if let Ok(user_proxy) = UserProxy::builder(&conn_clone)
                                        .path(path)
                                        .expect("path should be valid")
                                        .build()
                                        .await
                                    {
                                        if let (Ok(name), Ok(real_name)) =
                                            (user_proxy.user_name().await, user_proxy.real_name().await)
                                        {
                                            users.push(UserOption {
                                                username: name,
                                                realname: real_name,
                                            });
                                        }
                                    }
                                }
                            }
                        }

                        // Fallback to current user if list is empty
                        if users.is_empty() {
                            if let Ok(user) = std::env::var("USER") {
                                users.push(UserOption {
                                    username: user.clone(),
                                    realname: String::new(),
                                });
                            }
                        }

                        Message::UsersFound(users)
                    },
                    cosmic::Action::App,
                );

                return Task::batch(vec![find_device_task, fetch_users_task]);
            }

            Message::UsersFound(users) => {
                self.users = users;
                // Ensure selected_user is valid
                if let Some(selected) = &self.selected_user {
                    if !self.users.iter().any(|u| u.username == selected.username) {
                        if !self.users.is_empty() {
                            self.selected_user = Some(self.users[0].clone());
                        }
                    } else if let Some(updated_user) =
                        self.users.iter().find(|u| u.username == selected.username)
                    {
                        // Update realname if found
                        self.selected_user = Some(updated_user.clone());
                    }
                } else if !self.users.is_empty() {
                    self.selected_user = Some(self.users[0].clone());
                }
            }

            Message::UserSelected(user) => {
                self.selected_user = Some(user);
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
                self.status = Self::map_error(&err);
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
                    "enroll-too-fast" => fl!("enroll-too-fast"),
                    "enroll-duplicate" => fl!("enroll-duplicate"),
                    "enroll-cancelled" => fl!("enroll-cancelled"),
                    _ => status.clone(),
                };
                self.status = status_msg;

                if done {
                    self.busy = false;
                    self.enrolling_finger = None;
                }
            }

            Message::EnrollStop => {
                if let (Some(path), Some(conn)) =
                    (self.device_path.clone(), self.connection.clone())
                {
                    return Task::perform(
                        async move {
                            let device = DeviceProxy::builder(&conn).path(path)?.build().await?;
                            let _ = device.enroll_stop().await;
                            device.release().await?;
                            Ok::<(), zbus::Error>(())
                        },
                        |res| match res {
                            Ok(_) => cosmic::Action::App(Message::EnrollStatus(
                                "enroll-cancelled".to_string(),
                                true,
                            )),
                            Err(e) => cosmic::Action::App(Message::OperationError(e.to_string())),
                        },
                    );
                }
            }

            Message::DeleteComplete => {
                self.status = fl!("deleted");
                self.busy = false;
            }

            Message::Delete => {
                if let Some(page) = self.nav.data::<Page>(self.nav.active())
                    && let (Some(path), Some(conn), Some(user)) = (
                        self.device_path.clone(),
                        self.connection.clone(),
                        self.selected_user.clone(),
                    )
                {
                    self.status = format!("Deleting fingerprint {}", page.as_finger_id());
                    self.busy = true;
                    let finger_name = page.as_finger_id().to_string();
                    return Task::perform(
                        async move {
                            match delete_fingerprint_dbus(&conn, path, finger_name, user.username).await
                            {
                                Ok(_) => Message::DeleteComplete,
                                Err(e) => Message::OperationError(e.to_string()),
                            }
                        },
                        cosmic::Action::App,
                    );
                }
            }

            Message::Register => {
                if let Some(page) = self.nav.data::<Page>(self.nav.active())
                    && self.device_path.is_some()
                    && self.selected_user.is_some()
                {
                    self.busy = true;
                    self.status = "Starting enrollment...".to_string();
                    self.enrolling_finger = Some(page.as_finger_id().to_string());
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

    fn map_error(err: &str) -> String {
        if err.contains("net.reactivated.Fprint.Error.PermissionDenied") {
            fl!("error-permission-denied")
        } else if err.contains("net.reactivated.Fprint.Error.AlreadyInUse") {
            fl!("error-already-in-use")
        } else if err.contains("net.reactivated.Fprint.Error.Internal") {
            fl!("error-internal")
        } else if err.contains("net.reactivated.Fprint.Error.NoEnrolledPrints") {
            fl!("error-no-enrolled-prints")
        } else if err.contains("net.reactivated.Fprint.Error.ClaimDevice") {
            fl!("error-claim-device")
        } else if err.contains("net.reactivated.Fprint.Error.PrintsNotDeleted") {
            fl!("error-prints-not-deleted")
        } else if err.contains("net.reactivated.Fprint.Error.Timeout") {
            fl!("error-timeout")
        } else if err.contains("net.reactivated.Fprint.Error.DeviceNotFound")
            || err.contains("Failed to find device")
        {
            fl!("error-device-not-found")
        } else {
            err.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_error() {
        assert_eq!(
            AppModel::map_error("net.reactivated.Fprint.Error.PermissionDenied"),
            "Permission denied."
        );
        assert_eq!(
            AppModel::map_error("Error: net.reactivated.Fprint.Error.PermissionDenied: foo"),
            "Permission denied."
        );
        assert_eq!(
            AppModel::map_error("Some random error"),
            "Some random error"
        );
        assert_eq!(
            AppModel::map_error("net.reactivated.Fprint.Error.AlreadyInUse"),
            "Device is already in use by another application."
        );
        assert_eq!(
            AppModel::map_error("Failed to find device: something"),
            "Fingerprint device not found."
        );
    }
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
