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
npm test             # Frontend store tests
npm run build        # Production frontend build
```

Frontend code lives in `src/`; native Rust code and app configuration live in
`src-tauri/`. Run the desktop app for native data access; the frontend-only server
shows a recoverable connection error rather than silently using demo data.

## Shared Rust backend

`crates/gasd-core/` contains the transport-independent service/repository layer
and SQLite migrations for sections and tasks. Both the CLI and Tauri use this
service and the same database. See [the backend guide](crates/gasd-core/README.md) for
usage, persistence behavior, and adapter responsibilities.

### Desktop data flow

- `src-tauri/src/commands/` groups native commands by domain (`sections_*`,
  `tasks_*`), with `list_get` for loading and structured `{ code, message }` errors.
  SQLite operations run on Tauri's blocking pool behind a shared service mutex.
- `src/lib/data/client.ts` is the typed IPC boundary; `types.ts` mirrors core's
  serialized models. Keep these models aligned when changing the native API.
- `src/lib/data/list-store.ts` uses Svelte's built-in stores (no runtime store
  dependency). It owns loading, write serialization, refresh, and error recovery.
  Components own only presentation state and transient drag previews.
- The UI shows each section as a Kanban swimlane with Open, In progress, Waiting,
  and Done today columns. Tasks can be dragged within columns or between columns
  and sections, changing status/section and order atomically. Section creation,
  rename, deletion, task creation/edit/deletion, and completion/reopening remain
  available. Section deletion confirms that
  completed tasks are also removed. Empty databases are not seeded with demo data.
- Drag previews never reach storage. The destination's final drop calls core's
  atomic `place_task`, validating full destination membership before committing.
  Column order is scoped to section and status. Done today uses `statusChangedAt`
  and local calendar-day boundaries; older completed tasks remain stored and
  available through the CLI. Hidden tasks are included in drag membership, and
  the display refreshes its day boundary while open and on window focus.
  Stale orders caused by CLI edits fail safely and reload current data.
- Data reloads after writes and when the window regains focus (except during a
  drag); external CLI edits are not streamed live while the app stays focused.

The root Cargo workspace shares one `Cargo.lock`; its default members are the
core and CLI crates, so their checks do not require desktop libraries:

```sh
cargo test
cargo clippy -p gasd-core -p gasd-cli --all-targets -- -D warnings
cargo fmt --all -- --check
```

## Jira Cloud

The desktop app supports named, saved JQL queries and one optional Jira ticket
per local task. Jira access is read-only: linking does not change Jira or sync
task status. Open **Settings** in the app header to configure your connection:

1. Enter your Jira site (for example, `https://your-team.atlassian.net`).
2. Enter your Atlassian account email and API token.
3. **Save connection**, then **Test saved connection**.

Site and email are stored in the shared SQLite database's `app_settings` table.
The token is stored separately in your OS credential store (macOS Keychain,
Windows Credential Manager, or a Linux Secret Service provider such as GNOME
Keyring or KeePassXC). The UI only receives whether a token exists, never its
saved value. A locked or unavailable credential store produces an error; there
is no plaintext fallback. Credential-store work runs on the native blocking pool.

Leave the token field blank to keep the saved token for the same site and email,
or enter a replacement. Changes apply immediately, without restarting. Changing
the site or account requires a token for that connection; credentials are scoped
to the site and email. **Disconnect** removes the active connection and its token,
but keeps saved queries and task links. The earlier `GASD_JIRA_*` environment
variables are no longer used. On Linux, ensure your Secret Service provider is
running and its keyring is unlocked before saving a token. OS credentials are
separate from the database: database backups and `gasd clean` do not copy or delete
credential-store entries.

Use an **unscoped Jira Cloud API token** created at
<https://id.atlassian.com/manage-profile/security/api-tokens> and your Atlassian
account email. Scoped tokens using the Atlassian API gateway are not supported
in this minimal version. The site must be an HTTPS `*.atlassian.net` origin,
without a path. Requests time out after 30 seconds and never follow redirects
with credentials.

Use the header menu to switch between **Board**, **Jira import**, and **Settings**.
On the **Jira import** page, create a named query, then **Run query**.
Results show ticket key, summary, and status, with **Load more tickets** for
pagination. Select a task beside a result to link it; replacing an existing link
asks for confirmation. Alternatively, use **Link Jira ticket** on a task and
enter a key such as `TEAM-123`. Tickets are checked against Jira before linking.
Click a linked key to open Jira, or **Unlink** to remove the local association
(no connection needed). Deleting a query does not remove task links. Searches
run only when requested; results are not cached or synchronized in the background.
CLI task edits preserve links, and JSON task output includes `jira` when linked;
Jira query management and linking are desktop-only for now.

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
gasd list tasks --section 1 --status open
gasd update task 1 --status inProgress
gasd update task 1 --status waiting
gasd update task 1 --status done
gasd update task 1 --status open        # Reopen, not toggle
gasd update task 1 --title "Read two chapters" --section 1
gasd update section 1 --name "Home"
gasd list tasks --status done --json
```

Use IDs printed by the commands; the examples assume a fresh database. No default
sections are created. New tasks start with status `open`. Task updates preserve
fields not supplied and require at least one change. `--status` accepts exactly
`open`, `inProgress`, `waiting`, or `done` for updates and list filtering, and can
be combined with title/section edits atomically. The old `--done` and `--active`
flags are not supported. Human-readable output includes IDs and the actual status;
global `--json` emits arrays of core models, including `status` and
`statusChangedAt` (UTC Unix milliseconds), without a separate `done` field,
for both lists and mutations. Errors go to stderr, with exit code 1 for runtime
errors and 2 for invalid command syntax (0 for success/help/version).

### Resetting a development database

Close the desktop app and any other database users, then run:

```sh
gasd clean --yes                       # Permanently remove the shared database file
gasd --database ./test.sqlite3 clean --yes
```

This deletes the selected file without opening it, so it also works for outdated
or invalid databases. It does not recreate the file immediately; the next normal
command or desktop startup creates a fresh database and reapplies migrations.
An absent file is a successful no-op. `--yes` is required to confirm data loss.
`--json` returns `[{ "database": "...", "removed": true }]` (`false` if absent).

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
