# ecosystem-scheduler

Read-only cross-repo task aggregation and routing-plan computation for the
my-lisp ecosystem (`TAURICODE-SCHEDULER-01`).

## What it does

Reads every repo's durable `tasks.my`, projects them into one global graph
(M1.1: the sources stay the authority — this is a materialized view), derives
each task's readiness by fixed-point iteration, scores ready tasks with the
ecosystem's `next-best-action` convention (`priority × (1 + unblock_impact)`),
and emits a per-agent routing plan.

## What it deliberately does NOT do

No claiming, no mesh writes, no repo mutations. Dispatch _execution_ stays
with agents via swarm-node's quorum-guarded `(claim-task ...)`; this tool
produces the routing PLAN only — merging it into the claim path would
duplicate authority that M1.1 assigns elsewhere.

## Usage

```bash
ecosystem-scheduler --github-root ~/GitHub --agent ganaka-1=rust,lisp --format json
ecosystem-scheduler --repo ~/GitHub/cml --repo ~/GitHub/my-lisp --origin cml
```

- `--github-root <DIR>` scans immediate subdirectories containing `tasks.my`.
- `--agent <ID>=<cap1,cap2>` routes per agent; without it one `_any` ranked plan.
- Exit codes: `0` ok · `2` usage error · `3` source read/parse error (names the file).

Readiness labels are honest about what they cannot resolve:

| Label              | Meaning                             |
| ------------------ | ----------------------------------- |
| `ready`            | open, all deps exist and are done   |
| `waiting-on <id>`  | queued behind an open task          |
| `missing-dep <id>` | depends on an id defined in no repo |
| `cycle`            | blocked behind a dependency cycle   |

Duplicate task ids across repos are surfaced as warnings, never silently
deduplicated (first input occurrence keeps the seat, deterministically by
sorted repo name).

## Provenance

The `.my` reader is ported from `my-lisp/crates/swarm-node/src/{sexpr,tasks_file}.rs`
so both sides read the same format the same way — a shared-format
implementation pair, not an independent definition. Zero external
dependencies (trusted local files in the ecosystem's own dialect).

## Tests

```bash
cargo test   # from this directory
```
