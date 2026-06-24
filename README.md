# SkimmIQ Solver Injection

This repository contains an experimental SkimmIQ solver package prepared for
integration with the fan-made `ashtree74/rubik-kociemba-3d` web playground.

It packages two parts:

- `frontend/` - the Vite/Three.js SkimmIQ page and worker integration.
- `native/ashtree_native_bench/` - the Rust native solver used by the PHP API.

The current web integration is intentionally a **pure stateless solver** path:
it receives only the current color state and searches for a legal move sequence
to any solved color arrangement. It does not use the scramble history and it
does not fall back to Ariadne/history reversal.

## Why This Exists

The original SkimmIQ solver idea treated the puzzle as a state graph:
each color arrangement is a node, and each legal tape shift is an edge.

This package extends that direction with the native solver we developed during
SkimmIQ E-classic research. It is meant to be easy to inspect, run locally, and
drop into a hosted copy of the SkimmIQ page.

## Important Status

The solver is useful across many SkimmIQ layouts, but `E-classic` remains the
hardest stateless case.

For `E-classic`, the normal product flow should still prefer Ariadne/history
reversal whenever the app has the generated scramble history. This public
package is about the harder fallback case: solving from color state only.

In the web demo, the native backend timeout is currently set to 5 minutes.
When the solver does not finish in time, the API returns:

```json
{ "status": "timeout", "found": false }
```

See `docs/E_CLASSIC_STATUS.md` for more context.

## Repository Layout

```text
.
|-- frontend/
|   |-- skimmiq.html
|   |-- src/
|   |-- public/api/solve.php
|   |-- package.json
|   `-- vite.config.js
|-- native/ashtree_native_bench/
|   |-- Cargo.toml
|   `-- src/main.rs
|-- docs/
|   |-- DEPLOYMENT.md
|   |-- E_CLASSIC_STATUS.md
|   `-- INTEGRATION_NOTES.md
|-- NOTICE.md
`-- LICENSE.md
```

## Quick Start

Build the native solver:

```bash
cd native/ashtree_native_bench
cargo build --release
```

Build the frontend:

```bash
cd frontend
npm install
npm run build
```

Deploy the frontend and native binary together:

```bash
mkdir -p frontend/dist/bin
cp ../native/ashtree_native_bench/target/release/ashtree_native_bench \
  frontend/dist/bin/ashtree_native_bench
```

Then publish `frontend/dist/` under a web path such as:

```text
/skimmiq-solver/
```

The PHP endpoint expects this runtime layout:

```text
skimmiq-solver/
|-- skimmiq.html
|-- assets/
|-- api/solve.php
`-- bin/ashtree_native_bench
```

## Local Development

Frontend dev server:

```bash
cd frontend
npm run dev
```

Native solver smoke test:

```bash
cd native/ashtree_native_bench
cargo run --release -- solve-state \
  --layout E \
  --difficulty classic \
  --profile fast \
  --colors 0,0,0,0,0,0,0,0,0,1,1,1,1,1,1,1,1,1,2,2,2,2,2,2,2,2,2,3,3,3,3,3,3,3,3,3,4,4,4,4,4,4,4,4,4,5,5,5,5,5,5,5,5,5
```

## Notes For The Original Author

The main files to inspect first are:

- `frontend/src/skimmiq-worker.js`
- `frontend/src/skimmiq-api.js`
- `frontend/public/api/solve.php`
- `native/ashtree_native_bench/src/main.rs`

The integration point is simple: the browser worker sends the current SkimmIQ
color state to `api/solve.php`; PHP runs the native Rust solver; the browser
receives the solved move sequence and animates it.

## Attribution

This work builds on the fan-made puzzle playground and solver direction shared
by `ashtree74/rubik-kociemba-3d`, with permission for this SkimmIQ solver
collaboration. See `NOTICE.md`.
