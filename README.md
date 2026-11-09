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
