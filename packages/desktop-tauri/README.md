# Tauricode — agent-runtime-ui (official)

**Status:** Slice 1 — **VERIFIED milestone** (2026-08-25). Real
`EcosystemSnapshot` via `ecosystem-observer`, `cargo check` PASS,
`ecosystem-observer` test suite PASS, 0 warnings, CI PASS (run
32865864898). Not the full architecture yet — see "What's still open"
below. Stopping here deliberately: the next step (extending
`EcosystemSnapshot` to Guix/contracts/tasks/evidence, or starting
`packages/app` UI integration) is a separate decision, not an automatic
continuation of this one.
**Decision:** `ECO-DECISION-2026-08-19-TAURICODE-ROLE` / `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER` / `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE` (all ACCEPTED, `/home/agents/ecosystem/decisions/`).
**Not to be confused with:** `prototype/swarm_dashboard/` — an unrelated, mock-data-driven prototype (see its own README's boundary note). This package is the real one: everything it displays comes from a live scan, never a fixture.

## Portability — three separately-evidenced claim levels

Don't let the words "portable architecture" get ahead of what's
actually proven. As of 2026-08-25:

```
NOW    core is portable        — ecosystem-observer::discover_ecosystem
                                  takes root/repositories as call
                                  parameters, never hardcodes them
                                  (source-confirmed); AgentRuntimeAdapter
                                  is runtime-agnostic, proven twice
                                  (MockAdapter + real OpenCodeAdapter
                                  through one identical generic call
                                  site, M2.9).

NEXT   observer desktop is     — this package's command layer no
       configurable              longer hardcodes the repo list either
                                  (ECOSYSTEM_REPOS env var, comma-
                                  separated, same pattern as
                                  ECOSYSTEM_ROOT). Both env vars
                                  verified with 4 unit-test scenarios
                                  (default/custom/empty/stray-commas)
                                  in an isolated scratch crate before
                                  landing here.

LATER  whole agent ecosystem   — NOT attempted. Needs: contracts/tasks/
       is portable                evidence reading (ecosystem-observer
                                  itself still says this is "out of
                                  scope"), a real fresh-repo-set
                                  end-to-end test with zero
                                  my-lisp-specific patches, and
                                  packages/app UI integration. Do not
                                  claim this level until that
                                  experiment actually runs.
```

## What this slice actually does

One Tauri command, `get_ecosystem_snapshot`, calls
`ecosystem_observer::discover_ecosystem` for real — real `git`
plumbing (`rev-parse`, `symbolic-ref`, `status`, `remote -v`) against
the six sibling repos (`my-lisp`, `fpga-lisp`, `cml`, `my-idea`,
`my-lisp-panini`, `shiva-sutras`) under `$ECOSYSTEM_ROOT` (default
`/home/agents/GitHub`), plus real local-process observation (Slice 2).
Empirically confirmed against the real repos on this machine
(2026-08-25): all six scanned `Complete`, real branches/SHAs/dirty
states/remotes returned, 2 local processes observed — not predicted.

A minimal, deliberately plain `web/index.html` calls the command and
renders the result. This is **not** `packages/app` (the shared SolidJS
UI) — that integration is a separate, larger follow-up, not attempted
in this slice, to avoid reworking an unfamiliar, substantial codebase
in one unreviewed pass. `bundle.active` is `false`: this slice proves
the real data pipeline, it does not ship an installer.

## Deliberate deviations from the ACCEPTED architecture's illustrative file tree

- `ecosystem-observer` stays at its real, existing location
  (`crates/ecosystem-observer/`, repo root) rather than being moved or
  duplicated into `packages/desktop-tauri/src-tauri/crates/`. It
  already existed as real, tested Slice 1+2 code before this package;
  the decision's own file-tree section was marked "proposal, nothing
  created" (not binding to the exact path), and moving 2500+ lines of
  working code was a bigger, riskier call than this slice needed.
  Consumed via a relative path dependency instead.
- No `tauri-specta`/generated `bindings.ts` yet — the frontend calls
  `invoke()` directly, same pattern already used by
  `prototype/swarm_dashboard/`.
- No `windows.rs`/`job_object.rs`/`window_customizer.rs`/
  `linux_display.rs`/`markdown.rs`/`server.rs`/`cli.rs` from the old
  donor Tauri layer yet — this slice is read-only display of one
  command's output, none of that surface is needed for it.

## What's still open (separate, future, explicit-request work)

- `packages/app` (SolidJS) integration + `platform.ts` bridge.
- `tauri-specta` typed bindings.
- OpenCode sidecar wiring (`packages/opencode` as `externalBin`).
- Guix state (levels 1-4), ecosystem contracts (needs the minimal
  S-expr reader), evidence, and tasks — none of Stage 1's remaining
  criteria beyond repository discovery + local runtime observation are
  implemented in `ecosystem-observer` yet.
- my-idea boundary reconciliation.
- Any installer/bundle packaging for this shell.
