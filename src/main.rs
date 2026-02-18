// SPDX-License-Identifier: MPL-2.0

mod app;
mod config;
mod fprint_dbus;
mod accounts_dbus;
mod i18n;

extern crate tracing;
extern crate zbus;

const WINDOW_MIN_WIDTH: f32 = 400.0;
const WINDOW_MIN_HEIGHT: f32 = 380.0;

fn main() -> cosmic::iced::Result {
    // Get the system's preferred languages.
    let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();

    // Enable localizations to be applied.
    i18n::init(&requested_languages);

    // Settings for configuring the application window and iced runtime.
    let settings = cosmic::app::Settings::default().size_limits(
        cosmic::iced::Limits::NONE
            .min_width(WINDOW_MIN_WIDTH)
            .min_height(WINDOW_MIN_HEIGHT),
    );

    // Starts the application's event loop with `()` as the application's flags.
    cosmic::app::run::<app::AppModel>(settings, ())
}
