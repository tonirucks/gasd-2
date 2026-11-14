# gasd-core

Shared Rust backend for the desktop app and CLI. No Tauri, runtime,
command handlers, demo data, or global state are required. The optional `storage`
helpers select a shared per-user database path; explicit paths and in-memory
repositories remain available.

## Structure

- `sections/` and `tasks/`: each domain owns its models, service methods,
  and concrete persistence operations in `repository.rs`.
- `service`: shared application entry point.
- `error`: shared `Error` and `Result` types.
- `validation`: private input validation helpers.
- `sqlite`: shared connection setup, versioned migrations, and persistence helpers.
- `storage`: shared native database location and parent-directory creation.
- `model`: convenience re-exports of the domain models.

`Service` handles input validation and delegates persistence to `SqliteRepository`.
The repository's methods are grouped by domain, without traits or generic service
parameters. Models, `Service`, and `SqliteRepository` are exported at crate root.

Both domains use the same repository connection; splitting the code
does not split transactions or change cross-domain integrity and cascade rules.

## Persistence choices

Keep `rusqlite` and explicit SQL for this small, local SQLite backend. Additional
fields and tables do not by themselves require an ORM or query builder. Add small
helpers when repetition emerges rather than introducing a generic CRUD layer;
ordering and atomic list operations remain application-specific either way.

Database-specific error conversions live in the shared `sqlite` module; shared
errors expose a boxed storage error while preserving its source for diagnostics.

For now, retain the small migration runner: it acquires an immediate transaction
before reading the schema version, protecting simultaneous desktop/CLI startup.
Revisit a focused migration library as the schema grows, provided it preserves
that locking behavior and supports existing `user_version` databases.

## Usage

```rust
use gasd_core::{Service, SqliteRepository, TaskStatus, TaskUpdate};

# fn main() -> gasd_core::Result<()> {
let mut app = Service::new(SqliteRepository::in_memory()?);
// Production adapters use SqliteRepository::open(explicit_database_path).
let section = app.create_section("Personal")?;
let task = app.create_task(section.id, "Read a chapter")?;
app.set_task_status(task.id, TaskStatus::InProgress)?;
app.update_task(task.id, TaskUpdate {
    title: task.title,
    section_id: section.id,
    status: TaskStatus::Done,
})?;
# Ok(())
# }
```

## Behavior and adapter responsibilities

- IDs are database-generated and not reused. `position` is zero-based but may
  contain gaps after deletion/moves; callers must not assume contiguous values.
- Sections append to the section list. Tasks are ordered independently within
  each section/status column, including completed tasks.
- Tasks have a `TaskStatus`: `Open`, `InProgress`, `Waiting`, or `Done`, serialized
  as `open`, `inProgress`, `waiting`, and `done`. New tasks start open. Use
  `set_task_status` to change status without changing title or section. All
  status changes append to the destination column; repeating the same status
  preserves position.
- `Task.status_changed_at` (JSON `statusChangedAt`) is a UTC Unix timestamp in
  milliseconds. It starts at task creation and changes only on actual status
  transitions, including changes through `update_task` or
  `place_task`. Repeating a status, editing, moving, or reordering without changing
  status preserves it. Tracking uses the database clock; transitions within the
  same millisecond can share a timestamp.
- `TaskUpdate` replaces title, section, and status atomically. There is no separate
  completion flag. `TaskList { section_id, status }` identifies the complete
  destination column for `place_task` and `reorder_tasks`. Placement sets both
  section and status atomically with destination ordering.
- Moving or completing/reopening appends to the destination list; edits within a list
  preserve position. Reordering takes the complete list of IDs exactly once.
- Reordering is atomic; missing, foreign, duplicate, or stale membership is
  rejected without writing. This does not detect concurrent order-only changes;
  the last valid reorder wins. Use `place_task` to move and reorder atomically.
- The desktop shows only tasks completed during the user's current local day in
  each done column. Core and CLI retain all completed tasks. Filtered UI orders
  must include hidden tasks when calling core's complete-column ordering API.
- **Deleting a section also deletes all its tasks, including completed ones.**
  A future adapter should confirm destructive actions as appropriate.
- SQLite enforces foreign keys, cascading deletes, and basic field constraints.
  Every multi-statement mutation and migration uses an immediate transaction;
  each connection waits up to five seconds for locks before returning a storage
  error. Desktop and CLI can open separate connections to the same local file.
- Opening a database applies migrations transactionally using `user_version`.
  The initial schema (version 1) includes task status and status timestamps.
  During this early development stage, databases created with earlier schema
  definitions must be reset; there is no upgrade path for them.
  Unknown newer schema versions are rejected. Add sequential migrations when
  evolving the schema; do not edit an already-released migration.
- Adapters select a shared database path, create its parent directory, and map
  errors to IPC responses or CLI exit codes. The core's `storage` module
  provides shared path selection and opening for both adapters; the desktop opens
  that database at startup. No default sections are seeded.
- Calls are synchronous. A future async Tauri adapter should use a blocking worker
  and serialize access to its service/connection rather than blocking the UI thread.
  Prefer explicit desired status over a read-then-toggle operation.

## Checks

From the repository root (no desktop system dependencies needed):

```sh
cargo test -p gasd-core
cargo clippy -p gasd-core --all-targets -- -D warnings
cargo fmt --all -- --check
```
