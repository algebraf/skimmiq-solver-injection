# Deployment

This page documents the server-native deployment used by the SkimmIQ solver
injection.

## Requirements

Runtime server:

- static file hosting for HTML, JS, CSS, and assets
- PHP with `exec()` enabled
- permission to execute a native Linux binary from PHP
- enough CPU time for long `E-classic` searches

Build machine:

- Node.js and npm
- Rust and Cargo

The server does **not** need Rust installed if you copy a prebuilt
`ashtree_native_bench` binary compiled for the server's OS/CPU.

## Build

From repository root:

```bash
cd native/ashtree_native_bench
cargo build --release

cd ../../frontend
npm install
npm run build
```

Copy the native binary into the frontend build:

```bash
mkdir -p dist/bin
cp ../native/ashtree_native_bench/target/release/ashtree_native_bench \
  dist/bin/ashtree_native_bench
chmod 755 dist/bin/ashtree_native_bench
```

Publish `frontend/dist/` to your web server.

## Expected Runtime Layout

```text
skimmiq-solver/
|-- skimmiq.html
|-- index.html
|-- assets/
|-- api/
|   `-- solve.php
`-- bin/
    `-- ashtree_native_bench
```

`api/solve.php` uses the directory above `api/` as the solver root:

```php
$solverDir = realpath(dirname(__DIR__)) ?: dirname(__DIR__);
$solverBin = $solverDir . '/bin/ashtree_native_bench';
```

That makes the deployment independent of a hard-coded path such as
`/var/www/html/skimmiq-solver`.

## API Request

The frontend posts:

```json
{
  "state": {
    "layoutId": "E",
    "difficultyId": "classic",
    "colors": [0, 1, 2]
  },
  "profile": "quality",
  "timeoutMs": 300000
}
```

The `colors` array must contain the full state for the chosen layout.

## API Responses

Solved:

```json
{
  "status": "solved",
  "found": true,
  "moves": [{ "tapeId": "x0", "direction": 1 }],
  "text": "x0+"
}
```

Not found within solver search:

```json
{
  "status": "not_found",
  "found": false,
  "moves": []
}
```

Timed out:

```json
{
  "status": "timeout",
  "found": false,
  "reason": "timeout",
  "moves": []
}
```

## Shared Hosting Warning

Typical shared hosting often does not allow PHP to execute custom native
binaries. In that environment, host the frontend as static files and run the
solver API on a VPS or another machine that allows native processes.
