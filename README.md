# Fprint built for COSMIC™ DE

Program for GUI based fingerprint management.

## Prerequisites

You're using Linux or freedesktop compatible system with a supported fingerprint scanner. Tested with COSMIC™ DE, Pop!_OS, Framework 13 laptop with a Goodix MOC Fingerprint Sensor. 

## Usage

Choose which finger to register or delete by a tab. Change user from a menu (default is current session.)</br> Click the action you want to take. Prompts you for your password. Follow instruction.</br> If you don't have correct rights or incorrect password your attempt will be dismissed.

[google_screen_recording_2026-01-28T11-02_01.789Z.webm](https://github.com/user-attachments/assets/f12d923b-a290-4e45-94bd-aa39f6ed6782)

## Todos

- [x] Improve feedback given to user. Currently prints what daemon returns.
- [x] Add a user dropdown to make it possible for admin to register for other users.
- [ ] Get project into cosmic-utils.
- [x] Package & distribute, most likely as a flatpak, but maybe as a deb package also.
- [ ] Sherlock the application by adding all functionality directly into cosmic-settings.


## Installation

A [justfile](./justfile) is included by default for the [casey/just][just] command runner.

- `just` builds the application with the default `just build-release` recipe
- `just run` builds and runs the application
- `just install` installs the project into the system
- `just vendor` creates a vendored tarball
- `just build-vendored` compiles with vendored dependencies from that tarball
- `just check` runs clippy on the project to check for linter warnings
- `just check-json` can be used by IDEs that support LSP

## Translators

[Fluent][fluent] is used for localization of the software. Fluent's translation files are found in the [i18n directory](./i18n). New translations may copy the [English (en) localization](./i18n/en) of the project, rename `en` to the desired [ISO 639-1 language code][iso-codes], and then translations can be provided for each [message identifier][fluent-guide]. If no translation is necessary, the message may be omitted.

## Packaging

If packaging for a Linux distribution, vendor dependencies locally with the `vendor` rule, and build with the vendored sources using the `build-vendored` rule. When installing files, use the `rootdir` and `prefix` variables to change installation paths.

```sh
just vendor
just build-vendored
just rootdir=debian/cosmic-ext-fprint prefix=/usr install
```

It is recommended to build a source tarball with the vendored dependencies, which can typically be done by running `just vendor` on the host system before it enters the build environment.

### Flatpak

A Flatpak manifest is included in the `flatpak` directory. To build and install the Flatpak:

```sh
flatpak-builder --user --install --force-clean build flatpak/fi.joonastuomi.CosmicFprint.yml
```

Note: The included manifest uses network access during build to fetch dependencies. For strict offline builds, you should generate a `cargo-sources.json` file using `flatpak-builder-tools`.

## Developers

Developers should install [rustup][rustup] and configure their editor to use [rust-analyzer][rust-analyzer]. To improve compilation times, disable LTO in the release profile, install the [mold][mold] linker, and configure [sccache][sccache] for use with Rust. The [mold][mold] linker will only improve link times if LTO is disabled.

[fluent]: https://projectfluent.org/
[fluent-guide]: https://projectfluent.org/fluent/guide/hello.html
[iso-codes]: https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes
[just]: https://github.com/casey/just
[rustup]: https://rustup.rs/
[rust-analyzer]: https://rust-analyzer.github.io/
[mold]: https://github.com/rui314/mold
[sccache]: https://github.com/mozilla/sccache
