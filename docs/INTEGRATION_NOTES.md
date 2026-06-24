# Integration Notes

These are the main files to merge into an existing `rubik-kociemba-3d` style
page.

## Frontend

Key files:

- `frontend/src/skimmiq-worker.js`
- `frontend/src/skimmiq-api.js`
- `frontend/src/skimmiq-page.js`
- `frontend/public/api/solve.php`

Flow:

1. The SkimmIQ page builds the current color state with `puzzle.toJSON()`.
2. `skimmiq-worker.js` sends that state to the native API wrapper.
3. `skimmiq-api.js` posts JSON to `api/solve.php`.
4. `solve.php` runs the native Rust binary.
5. The worker returns `solved`, `not_found`, or `timeout` to the page.

## Native Solver

Key files:

- `native/ashtree_native_bench/Cargo.toml`
- `native/ashtree_native_bench/src/main.rs`

The web API calls:

```bash
ashtree_native_bench solve-state \
  --layout E \
  --difficulty classic \
  --profile quality \
  --colors 0,1,2,...
```

## Deployment Contract

The deployed web directory must contain:

```text
api/solve.php
bin/ashtree_native_bench
assets/
skimmiq.html
```

Do not rely on `/var/www/html`. The PHP endpoint is path-relative by design.

## No Ariadne Fallback

The injected solver is intentionally stateless. It does not reverse the
scramble sequence and does not use hidden scramble history.
