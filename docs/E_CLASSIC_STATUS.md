# E-classic Solver Status

`E-classic` is the hardest SkimmIQ layout/difficulty for a stateless solver.

The important distinction:

- Ariadne/history reversal is excellent when the app generated the scramble and
  still knows the move history.
- This solver is the harder fallback: it receives only the current color state.

## What The Current Solver Does

The web integration uses the native Rust solver through `api/solve.php`.

Default profile selection:

- `E-classic` -> `quality`
- other `E` difficulties -> `balanced`
- other layouts -> `fast`

Default timeout:

```text
300000 ms = 5 minutes
```

The UI reports `Timed out` when the native solver exceeds that budget.

## Practical Interpretation

For `E-classic`, a timeout does not mean the state is invalid. It means the
current stateless solver did not find a solution within the configured budget.

This is expected for some high-entropy `E-classic` states. During research, the
color-only stateless approach reached a practical ceiling: it is useful, but it
cannot reliably match Ariadne/history reversal on hard `E-classic` scrambles.

## Product Recommendation

For a production SkimmIQ app:

1. Use Ariadne/history reversal when the scramble history is known.
2. Use this stateless solver as a fallback/demo/research solver.
3. Treat `E-classic` timeout as a valid solver outcome, not as a UI or backend
   crash.
