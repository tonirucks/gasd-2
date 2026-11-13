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
and SQLite migrations for sections and tasks. It is not connected to Tauri
commands or a CLI yet. See [the backend guide](crates/gasd-core/README.md) for
usage, persistence behavior, and adapter responsibilities.

The root Cargo workspace shares one `Cargo.lock`; its default member is the core
crate so backend checks do not require desktop system libraries:

```sh
cargo test
cargo clippy -p gasd-core --all-targets -- -D warnings
cargo fmt --all -- --check
```
