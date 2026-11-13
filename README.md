# gasd-2

Basic Tauri v2 desktop app with Svelte 5, SvelteKit, and TypeScript, generated
with the official `create-tauri-app` helper. SvelteKit uses static output for Tauri.

## Setup

Node.js and Rust are pinned in `.tool-versions` for asdf. Ensure the `nodejs`
and `rust` asdf plugins are installed, then run:

```sh
asdf install
npm ci
```

Install the [Tauri system prerequisites](https://tauri.app/start/prerequisites/)
for your OS as well. On Debian/Ubuntu:

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev
```

## Commands

```sh
npm run tauri dev    # Desktop app with hot reload
npm run tauri build  # Production desktop bundles
npm run dev          # Frontend only (native commands require Tauri)
npm run check        # Svelte and TypeScript checks
npm run build        # Production frontend build
```

Frontend code lives in `src/`; native Rust code and app configuration live in
`src-tauri/`. The starter includes a `greet` command demonstrating frontend-to-Rust calls.

## Shared Rust backend

`crates/gasd-core/` contains the transport-independent service/repository layer
and SQLite migrations for sections and tasks. The CLI uses it directly; Tauri
opens the shared database at startup, but task/section IPC commands and UI are
not wired up yet. See [the backend guide](crates/gasd-core/README.md) for
usage, persistence behavior, and adapter responsibilities.

The root Cargo workspace shares one `Cargo.lock`; its default members are the
core and CLI crates, so their checks do not require desktop libraries:

```sh
cargo test
cargo clippy -p gasd-core -p gasd-cli --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Command-line interface

The non-interactive CLI lives in `crates/gasd-cli/` and builds a `gasd` binary.
Install it locally (no Node.js or Tauri dependencies required):

```sh
cargo install --path crates/gasd-cli --locked
```

Or run from source with `cargo run -p gasd-cli -- <command>`.

```sh
gasd add section "Personal"             # Prints the generated section ID
gasd add task "Read a chapter" --section 1
gasd list                              # All tasks, including completed tasks
gasd list sections
gasd list tasks --section 1 --active
gasd update task 1 --done
gasd update task 1 --active             # Reopen, not toggle
gasd update task 1 --title "Read two chapters" --section 1
gasd update section 1 --name "Home"
gasd list tasks --done --json
```

Use IDs printed by the commands; the examples assume a fresh database. No default
sections are created. Task updates preserve fields not supplied. `--done` and
`--active` are mutually exclusive, and updates require at least one change.
Human-readable output includes IDs; global `--json` emits arrays of core models
for both lists and mutations. Errors go to stderr, with exit code 1 for runtime
errors and 2 for invalid command syntax (0 for success/help/version).

### Shared local storage

`gasd_core::storage` owns the database location and creates parent directories.
Both CLI and GUI use the same per-user native SQLite file, independent of the
working directory or desktop bundle ID. This is **not browser localStorage**.

| Platform | Default database |
| --- | --- |
| Linux | `$XDG_DATA_HOME/gasd/gasd.sqlite3`, or `~/.local/share/gasd/gasd.sqlite3` |
| macOS | `~/Library/Application Support/gasd/gasd.sqlite3` |
| Windows | `%LOCALAPPDATA%\gasd\gasd.sqlite3` |

Set `GASD_DATABASE` in both processes' environments to override their shared
location. The CLI also accepts global `--database <path>`, taking precedence over
the environment for that invocation. Prefer absolute override paths so different
working directories still resolve to the same file. Empty paths are rejected.
SQLite locking and the core's five-second busy timeout support separate CLI/GUI
connections. This is local storage, not a cross-device sync mechanism.

### Distribution

`cargo build --release -p gasd-cli --locked` produces `target/release/gasd`
(`gasd.exe` on Windows). It can be distributed independently from the desktop app.
Future release CI can build native binaries for each supported OS and architecture;
package-manager releases can wrap those binaries. Publishing `gasd-core` and
`gasd-cli` to crates.io would also enable `cargo install gasd-cli`.
Release automation and registry publishing are not configured yet.
