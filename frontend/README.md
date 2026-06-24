# SkimmIQ Solver Injection Frontend

This folder contains the Vite/Three.js frontend for the SkimmIQ solver
injection package.

The important page is:

```text
skimmiq.html
```

The SkimmIQ page calls the native solver API through:

```text
src/skimmiq-worker.js
src/skimmiq-api.js
public/api/solve.php
```

See the repository root `README.md` and `docs/DEPLOYMENT.md` for full build and
deployment instructions.

## Development

```bash
npm install
npm run dev
```

Open:

```text
http://127.0.0.1:5173/skimmiq.html
```

## Build

```bash
npm run build
```

The production build is written to `dist/`.
