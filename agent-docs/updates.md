# Updates

How an installed Opsis checks for, downloads, verifies and installs a newer version, and how the About page shows it. Paths are relative to the repository root. How update files are signed and the feed is written is in [packaging.md](packaging.md); the settings store is in [app-state.md](app-state.md); see also [AGENTS.md](../AGENTS.md).

## Code map

| Concern                        | Where                                                                                                     |
| ------------------------------ | --------------------------------------------------------------------------------------------------------- |
| Which installer made this copy | `crates/update/src/installer.rs` → `installed_from`, `Installer`                                          |
| The feed and its keys          | `crates/update/src/feed.rs` → `Feed`, `feed_url`, `feed_key`, `is_newer`                                  |
| Check, download, verify        | `crates/update/src/fetch.rs` → `check`, `offered`, `download`, `public_key`                               |
| Install                        | `crates/update/src/install.rs` → `install`, `windows_script`, `replace_file`, `swap_folder`               |
| Build target and commit        | `crates/update/build.rs`; `opsis_update::TARGET`, `opsis_update::COMMIT`                                  |
| Update state, start check      | `crates/session/src/updates.rs` → `Updates`, `begin`, `check`, `download`, `want_restart`, `finish`       |
| Setting                        | `crates/types/src/settings.rs` → `UpdatesSection` (`updates.checkOnStart`)                                |
| About page                     | `crates/qt/cpp/update_card.cc` → `UpdateCard`; `crates/ui/src/ui.rs` → `update_view`, `restart_to_update` |
| Install at exit                | `crates/ui/src/main.rs` → `main` (after `Session::end`)                                                   |
| Public key                     | `signing/opsis.key.pub`                                                                                   |

## Rules

- **Opsis goes online for updates only when the person clicks Check for updates, or at start when they turned on "Check for updates when Opsis starts"** (`updates.checkOnStart`, off by default); a download starts only on Download update.
- **Only a copy installed from a release package checks.** The installer marker (`installed_from`) is `None` in development builds and in any program the packager didn't mark, and the About page's Updates card says so.
- **The feed is `latest.json` on the repository's latest release** (`feed_url`); a draft or prerelease isn't "latest", so a release reaches people only once it's published.
- **A version is newer only by semver order (`is_newer`)**, so a prerelease is older than its release.
- **Each package asks for its own file**: the feed key is `{os}-{arch}-{installer}` (`feed_key`, with Rust's names for the system and processor and `Installer::name`, through `this_computer`), so the NSIS and MSI installs on one computer get their own installer and never cross over.
  - A feed with no file for this computer's key says the version has no update for this computer yet (`NoFile`).
- **deb, rpm and pacman installs never download**: they show the version and its notes and open the release page, since their package manager owns the files.
- **A download is kept only if its minisign signature verifies against the public key compiled in** (`public_key`, from `signing/opsis.key.pub`), checked while it streams; otherwise the file is deleted and the person is told it wasn't signed by Opsis.
  - Losing the private key (the git-ignored `signing/opsis.key` and the repository's secret) means installed copies can never update again.
- **Downloads go to `<data folder>/updates/<version>`**, and that folder is emptied at the next start, so nothing pending survives a restart; a running download is in `AppState.tasks`, so the exit dialog lists it.
- **A downloaded update installs when Opsis closes, after the close dialog and the last save** (`finish`, called from `main` after `Session::end`): Restart to update closes the usual way and reopens the new version; an ordinary close installs it without reopening. Declining the close dialog cancels the restart.
- **Opsis does the install step itself, never in the system temp folder**:
  - AppImage: the new file is written beside the running one (`$APPIMAGE`) and renamed over it, keeping its permissions.
  - macOS: the `.app.tar.gz` is unpacked beside the bundle, the bundle renamed aside, the new one moved in and the old one removed (`swap_folder`); reopening is `open -n`.
  - Windows: a hidden PowerShell waits for Opsis to exit, runs the NSIS installer with `/P` or `msiexec /i … /passive`, and starts the program again only for Restart to update (`windows_script`).
- **The About page shows the build target and, for builds made by `bun run make`, the commit** (`OPSIS_COMMIT`, set from `GITHUB_SHA` or `git rev-parse HEAD`); other builds leave it out, and `opsis-update` builds again only when that value changes (`rerun-if-env-changed`).

## Deliberate limits

- **A failed install is only logged** (`finish`): the app is already closing, so the person sees nothing, and the next start deletes the download.
- **A macOS app in a folder the person can't write to (an `/Applications` owned by another administrator) fails to update** that way; it doesn't ask for an administrator password.
- **There is no delta update**: every update downloads the whole package.
