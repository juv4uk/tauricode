# Stage 2 — Runtime Adapter Layer: Implementation Plan

Status: IN PROGRESS. Grounds every increment in the six audits completed
2026-08-25 (fork model, dependency boundary, minimal stable core, adapter
contract, event contract). Do not re-litigate those conclusions here without
new evidence — this document executes them.

Established conclusions this plan assumes as settled:

```
Tauricode Core != OpenCode
OpenCode = optional runtime adapter
adapter contract must be derived from Tauricode needs
task-id != runtime session-id
heartbeat != domain event
attach/resume semantics are runtime-specific (capability flags, not base ops)
events must be normalized, not copied from OpenCode protocol
```

Minimal proven runtime contract (7 ops): `launch, stop, status, identity,
capabilities, workspace, events`. `send_input`, `shutdown` (separate from
`stop`), `session_started/resumed`, `capability_changed`, `heartbeat` (as a
domain event) are explicitly excluded — no proven core need as of this plan.

Minimal proven event vocabulary (8 events): `runtime_started,
runtime_stopped, task_bound, output_chunk{kind}, tool_started,
tool_completed, status_changed, error`.

## Why this increment order, not the suggested default

The suggested default order (M2.0 contract types -> M2.4 mock adapter ->
M2.5 OpenCode adapter) is kept as-is: it matches this codebase's own
precedent in `ecosystem-observer` (Slice 1 shipped a narrow, fully-tested
read-only core before Slice 2 added the self-reported layer) — prove the
contract with a mock before touching a real, harder-to-test external
process. No fact in this repo argues for reordering it.

One change from the suggested list: M2.5 (OpenCode adapter) is **out of
scope for this session's execution pass** and left as a stub increment
with its evidence requirement stated up front, not attempted. Reason:
launching and observing a real `opencode serve` subprocess end-to-end is
qualitatively riskier right now than any increment attempted today — this
session already root-caused two real WSL-hang mechanisms (unbounded
`git push`-triggered typecheck, `ollama`/`dxgkrnl` kernel WARNING) tied
to uncontrolled subprocess/resource behavior on this same machine. Proving
the contract boundary via M2.0-M2.4 (pure types, no subprocess) carries
none of that risk and delivers the architecturally load-bearing result
the audits actually asked for: "Core does not need to know which runtime
is under it." M2.5 is correctly attempted in a separate, deliberate
session, not appended unreviewed to an already long one.

## New crate: `crates/agent-runtime-contract`

Standalone crate (no workspace `Cargo.toml` exists in this repo — matches
`ecosystem-observer`/`swarm-cli`/`ecosystem-scheduler` precedent), zero
runtime dependencies beyond what a contract crate needs. Lives separately
from `ecosystem-observer` because the audits established these are
different concerns: `ecosystem-observer` is OS/git observation of
_existing_ processes; `agent-runtime-contract` is the _adapter_ boundary
for runtimes Tauricode itself launches and controls. Conflating them would
re-introduce exactly the coupling the audits spent effort ruling out.

---

## M2.0 — contract types only

- **goal:** define the 7-operation contract's data types (not the trait
  itself yet) — `RuntimeHandle` (opaque), `Identity`, `Capabilities`,
  `Workspace`, `Status` — with zero behavior, zero I/O.
- **files affected:** new `crates/agent-runtime-contract/Cargo.toml`,
  `crates/agent-runtime-contract/src/lib.rs`, `src/types.rs`.
- **public contract change:** none yet (new crate, nothing depends on it).
- **implementation steps:** 1) create crate skeleton; 2) define
  `RuntimeHandle(String)` (opaque wrapper, no runtime-specific fields —
  audit finding: "runtime handle must be opaque"); 3) reuse the _shape_
  already proven in `ecosystem-observer::SelfReportedIdentity` for
  `Identity`/`Capabilities` fields (`model, role, repository, instance,
task, declared_capabilities`) rather than inventing a new shape — the
  audit found this shape already generic; 4) define `Status` as the
  existing `IdentityStatus{Fresh,Stale,Orphaned,NotFound}` enum, imported
  by value/re-derived, not silently diverged; 5) define `Workspace{cwd,
repo_association}` matching `OsObservedFacts`' proven fields.
- **tests:** unit tests only — construction, `Debug`/`Clone`/equality
  where relevant. No behavior to test yet.
- **evidence required:** `cargo build -p agent-runtime-contract` and
  `cargo test -p agent-runtime-contract` both green.
- **rollback boundary:** delete the crate directory; nothing else
  references it yet.
- **done criteria:** types compile, tests green, no clippy warnings
  (`cargo clippy -- -D warnings`, matching this repo's own gate
  convention seen in `my-lisp`'s CI).

## M2.1 — generic runtime handle + lifecycle state

- **goal:** add the lifecycle state machine (`LifecycleState` — a small
  enum tracking `Launching -> Running -> Stopped`, kept separate from
  `Status`/`IdentityStatus` per the audit's explicit warning not to
  conflate "process lifecycle" with "identity freshness" — they are
  different axes, same mistake root policy already flags for
  `IdentityStatus` itself).
- **files affected:** `crates/agent-runtime-contract/src/lifecycle.rs`.
- **public contract change:** adds `LifecycleState` enum + a
  `RuntimeHandle -> LifecycleState` in-memory map type (`HandleRegistry`),
  still no adapter trait, no real process.
- **implementation steps:** 1) define `LifecycleState`; 2) define
  `HandleRegistry` as a plain in-memory `HashMap<RuntimeHandle,
LifecycleState>` with `register/transition/get` methods; 3) enforce
  legal transitions only (`Launching -> Running`, `Running -> Stopped`,
  reject others) — return a typed error, never silently allow an invalid
  transition (matches root policy's "never hide unknown/failure").
- **tests:** legal transition sequence succeeds; illegal transition
  (e.g. `Stopped -> Running`) is rejected with a typed error; unknown
  handle lookup returns `None`/typed error, not a panic.
- **evidence required:** `cargo test -p agent-runtime-contract` green,
  including the illegal-transition negative test.
- **rollback boundary:** revert `lifecycle.rs`; M2.0 types stand alone.
- **done criteria:** all lifecycle tests green; no adapter or I/O code
  introduced.

## M2.2 — normalized event types

- **goal:** the 8-event vocabulary as typed data, with `output_chunk`
  carrying a `kind: Text | Reasoning | ToolCall | ToolResult` tag (the
  audit's specific finding: one untyped `output_chunk` is insufficient).
- **files affected:** `crates/agent-runtime-contract/src/events.rs`.
- **public contract change:** adds `Event` enum (8 variants) and
  `OutputKind` enum (4 variants), both `Serialize`/`Deserialize` (events
  cross an adapter boundary, likely eventually to a dashboard over IPC —
  same JSON-for-untrusted/external-facing-data precedent as
  `ecosystem-observer`'s `serde_json` dependency).
- **implementation steps:** 1) define `OutputKind`; 2) define `Event`
  with exactly the 8 audited variants, each carrying only the fields the
  audit justified (`task_bound{handle_id, task_id, origin}`,
  `tool_started/completed{handle_id, task_id?, tool_name: String
(opaque), status}` — no `tool_failed` variant, folded into
  `tool_completed.status`, per the audit's explicit anti-proliferation
  finding); 3) add serde round-trip tests.
- **tests:** serialize/deserialize round-trip for every variant;
  explicit test asserting `Event` has exactly 8 variants (a compile-time
  exhaustive `match` in a test function — if someone adds a 9th variant
  without justification, this test forces them to touch this file and
  see why 8 was deliberate).
- **evidence required:** `cargo test -p agent-runtime-contract` green,
  round-trip tests included.
- **rollback boundary:** revert `events.rs`.
- **done criteria:** 8 variants, round-trip tests green, no
  runtime-specific field (no `session_id`, no OpenCode `Part` type
  leaking in).

## M2.3 — adapter trait/interface

- **goal:** the actual `AgentRuntimeAdapter` trait, built from M2.0-M2.2
  types only.
- **files affected:** `crates/agent-runtime-contract/src/adapter.rs`,
  `lib.rs` (re-exports).
- **public contract change:** the trait itself —
  `launch(cwd, task: Option<TaskId>) -> Result<RuntimeHandle, AdapterError>`,
  `stop(&RuntimeHandle) -> Result<(), AdapterError>`,
  `status(&RuntimeHandle) -> Result<Status, AdapterError>`,
  `identity(&RuntimeHandle) -> Result<Identity, AdapterError>`,
  `capabilities(&RuntimeHandle) -> Result<Capabilities, AdapterError>`,
  `workspace(&RuntimeHandle) -> Result<Workspace, AdapterError>`,
  `events(&RuntimeHandle) -> EventStream` (an associated type / boxed
  iterator-of-events shape, kept abstract — no async runtime dependency
  decision forced at this layer). Plus a separate `capability flags`
  trait or const fn (`supports_live_attach() -> bool`, `supports_resume()
-> bool`) per the audit's explicit instruction not to fake a common
  attach/resume API.
- **implementation steps:** 1) write the trait exactly as audited, no
  extra methods; 2) write `AdapterError` as a small enum (`LaunchFailed,
NotFound, Unsupported, Io(String)`); 3) document on the trait itself,
  in one line per method, which audit finding justifies it (traceability
  back to the audits, not just to this plan).
- **tests:** none yet possible (no implementation) beyond "trait compiles
  and is object-safe if that's a goal" — verify object-safety only if a
  consumer will need `Box<dyn AgentRuntimeAdapter>` (check M2.8's
  scheduler/dashboard consumption need before deciding; do not add
  object-safety constraints speculatively).
- **evidence required:** `cargo build -p agent-runtime-contract` green
  with the trait defined and zero implementors yet.
- **rollback boundary:** revert `adapter.rs`.
- **done criteria:** trait compiles; every method traceable to a specific
  audited finding in a doc comment; no method present that the audits
  marked "not proven" (`send_input`, separate `shutdown`, `attach` as a
  base method).

## M2.4 — mock adapter for tests

- **goal:** the first (and, for this session, only) real implementor —
  proves the trait is actually implementable and that Core can drive it
  without knowing it's fake.
- **files affected:** `crates/agent-runtime-contract/src/mock.rs` (or a
  `tests/` integration test module — decide based on whether other
  crates will want to reuse the mock; default to `src/mock.rs` behind a
  `#[cfg(any(test, feature = "mock"))]` gate so it can be reused later
  without becoming default production surface).
- **public contract change:** adds `MockAdapter`, no change to the trait.
- **implementation steps:** 1) `MockAdapter` holds an in-memory
  `HandleRegistry` (from M2.1) plus a scripted event queue per handle; 2) implement all 7 trait methods purely in-memory, no subprocess, no
  I/O; 3) `launch` synthesizes a `RuntimeHandle`, transitions
  `Launching -> Running`, and can be configured (test-only) to emit a
  scripted `Vec<Event>` from `events()`.
- **tests:** full lifecycle test — `launch` -> `status` is `Running` ->
  `events()` yields the scripted sequence in order -> `stop` -> `status`
  is reflected as stopped -> calling any op on a stopped/unknown handle
  returns the correct typed error, not a panic. This is the "smallest
  vertical slice that can be verified end-to-end" the plan's own
  ordering principle asked for.
- **evidence required:** `cargo test -p agent-runtime-contract` green,
  full lifecycle test included, run at least once with
  `cargo test -- --nocapture` to inspect actual event ordering by eye
  (not just assert-pass) before calling this done.
- **rollback boundary:** revert `mock.rs`; trait (M2.3) stands alone,
  unimplemented.
- **done criteria:** **"Core does not need to know which runtime is
  under it"** is now a checked fact, not a claim — a test written
  against `Box<dyn AgentRuntimeAdapter>` (or generic `impl` bound) that
  only ever sees `MockAdapter` proves the trait boundary holds with zero
  reference to OpenCode/Claude anywhere in this crate.

## M2.5 — OpenCode adapter minimal launch/stop/status — DONE

Preflight before starting (per this section's own stated requirement):
57min+ stable uptime, load average 0.05-0.08, 6.3GB RAM available,
`ollama.service` confirmed inactive/disabled (the kernel-WARNING factor
this same session root-caused earlier). Implemented in
`src/opencode_adapter.rs`, gated behind `feature = "opencode"`. Real,
isolated `opencode serve` subprocess (never the shared
`opencode-shared.service`); `stop()` uses a bounded `try_wait()` poll,
never a blocking `wait()`; liveness checked via `/proc/<pid>` only, zero
HTTP round-trips anywhere. `identity`/`capabilities`/`events` return
honest stubs (M2.5's own stated scope is launch/stop/status only).
21/21 tests green, 0.05s total runtime, zero orphaned processes verified
after. One real bug caught before committing: a broken placeholder test
in M2.0 (`unimplemented!()` inside a test body) — fixed by adding a real
`serde_json` dev-dependency and a real round-trip assertion, not by
deleting the test.

## M2.6 — OpenCode event normalization — DONE

`src/opencode_log_normalizer.rs`: pure text parser over OpenCode's own
structured log format (`~/.local/share/opencode/log/opencode.log`), not
HTTP — keeps M2.5's zero-network-call property. Deliberately incomplete:
only `ToolStarted` (from `evaluated permission=...` lines) and `Error`
(from `stream error` lines) have direct evidence from this session's own
captured log lines to normalize from. Every other message form actually
observed (`llm runtime selected`, `resolved path`, `stream`, `loop`,
`process`, `cancel`) is explicitly dropped, not force-mapped — inventing
a parse rule for a value never seen would violate `Unknown > invented`.
`cancel` specifically surfaces a real, disclosed gap: session
cancellation is not `RuntimeStopped`, and no core event represents it
yet. Test fixtures are verbatim structure from this session's own
`opencode.log`, not synthesized. One real bug caught by a failing test,
not by inspection: an over-escaped match-arm string literal that would
have made the `Error` mapping silently unreachable.

Known, disclosed gap left for a later increment: this normalizer is not
yet wired into `OpenCodeAdapter::events()` (still the M2.5 stub) — doing
so requires correlating a specific spawned instance's `run=` id against
the shared log file, a nontrivial problem not solved this session, not
papered over either.

## M2.7 — task binding/provenance — DONE

`launch()`'s `task` parameter was accepted since M2.5 but silently
discarded (bound to `_task`). Now stored on `Instance` and surfaced
through `identity().task` — the one `Identity` field this adapter can
honestly fill in from what it was actually given; `model`/`role`/
`instance` stay `None` (no introspection wired, never invented).
`TaskBound`'s `origin` field stays untracked — no caller this session had
a real value for it. Two new tests (task present / task absent), 29/29
total.

## M2.8 — dashboard/scheduler consumption — RESCOPED TWICE, DONE (2nd rescope)

**Second rescope (2026-08-25, later same day):** the target recorded
below (wiring `AgentRuntimeAdapter` behind `get_swarm_topology`/
`query_derivation_trace`/`stream_phoneme_vector`) was itself found
wrong before being attempted: those three commands are swarm-mesh-
topology/Pāṇinian-derivation/phoneme-vector domain, `AgentRuntimeAdapter`
is process-lifecycle domain (launch/stop/status/identity/capabilities/
workspace) — no natural connection point, confirmed by reading both
sides' actual signatures rather than assumed. Owner decision (asked
directly, not guessed): reformulate M2.8 as its own panel instead of
forcing the connection.

**Done as:** `prototype/swarm_dashboard/src-tauri/src/commands/agent_runtime.rs`
— a new, separate Tauri state (`AgentRuntimePanelState`) and three IPC
commands (`list_agent_runtimes`, `launch_agent_runtime`,
`stop_agent_runtime`) backed by `agent-runtime-contract::MockAdapter`
(the crate's "mock" feature — not "opencode": launching a real
subprocess from a GUI button is a separate, bigger decision than
exposing the status shape). Registered alongside the existing swarm
commands in one `tauri::generate_handler!` call in `lib.rs`
(`register_swarm_commands()` from M2.10 left untouched/unused — no
reason to disturb an already-CI-verified-green code path for this).
Type-correctness verified against the real, unmodified
`AgentRuntimeAdapter` trait and `agent-runtime-contract` itself
`cargo check`s clean with `--features mock`; the full Tauri build
(needs pkg-config/libglib2.0-dev, unavailable in this session's
sandbox) is CI's job, same verification-boundary note as M2.8's data
sibling. No frontend wiring yet — this is the backend IPC surface
only, same honest scoping as `full_fixtures()`.

---

### First rescope (superseded by the above, kept for the record)

Originally deferred (see prior paragraph, kept below for the record):
concretely meant modifying `ecosystem-scheduler`, an existing, separate,
already-shipped crate — crossing into another crate's ownership without
the owner's input on how it should consume this contract.

**Circumstances changed after that deferral, same session (2026-08-25,
owner working directly, not through an agent):** the owner himself
landed `3eeabe449` (`feat: add Tauricode Windows Tauri build`) — a real,
buildable `prototype/swarm_dashboard/src-tauri/` (`Cargo.toml`,
`tauri.conf.json`, `main.rs`/`lib.rs`, currently a bare
`tauri::Builder::default()`), plus `dae21bb36` closing the exact
`publish.yml` `publish`-job upstream-gate gap this same investigation
had found earlier but left unfixed (crossing into CI/release territory
was correctly not this crate's call either), and two `beta.ts`
robustness fixes for the fork's missing remote `beta` branch.

This does not retroactively make M2.8-as-`ecosystem-scheduler`
attempted — that specific target is still open, for the same reason as
before. But it opens a **more concrete, better-scoped M2.8 target that
didn't exist when this plan was written**: the already-designed
`prototype/swarm_dashboard/tauri_ipc.rs` (real `#[tauri::command]`
functions: `get_swarm_topology`, `query_derivation_trace`,
`stream_phoneme_vector`) has never been registered into the new
`main.rs`/`lib.rs` — the scaffold and the IPC design exist side by side,
unconnected. Wiring `agent-runtime-contract`'s `AgentRuntimeAdapter`
behind those commands (so the dashboard can actually launch/observe a
real adapter through the real desktop shell, not just fixtures) is now
a concrete, testable, in-crate-adjacent task — not a speculative
cross-crate commitment. Still not attempted as of this update; recorded
here as the corrected next target, not silently substituted for the
original one.

## M2.9 — second adapter proof — DONE

`tests/second_adapter_proof.rs` (crate integration test — public-API-only
visibility, a real black-box check). NOT a Claude adapter: spawning a
second, recursive AI agent process is a materially different risk
category from spawning `opencode serve` (autonomous behavior, potential
runaway cost/loops), and wasn't a call to make solo. Used the plan's own
stated alternative ("Claude or minimal dummy second runtime") via the
already-real, already-proven `OpenCodeAdapter` instead of inventing a
synthetic dummy — a stronger proof than a dummy would have been. 4 tests:
each adapter individually satisfies the contract; both pass through one
identical `&mut dyn AgentRuntimeAdapter` call site that never names which
concrete type it's driving; `supports_live_attach`/`supports_resume`
genuinely differ between the two real implementations (evidenced from
this session's own `opencode.log`, not asserted blind).

---

## This session's execution scope — final status

M2.0 through M2.9 attempted; M2.0-M2.7 and M2.9 done, M2.8 deliberately
left open (see above) — not silently dropped. "Core does not need to
know which runtime is under it" is now proven twice: once against a pure
mock (M2.4) and once against a real, independently-implemented subprocess
adapter through the identical generic call site (M2.9). All work
committed incrementally (one commit per increment) and pushed after each
increment completed, per the owner's own instruction mid-session.

## M2.10 — wire tauri_ipc.rs into the real shell — DONE, CI-VERIFIED GREEN

Step 2 (move the three `#[tauri::command]` functions into
`src-tauri/src/commands/swarm_dashboard.rs`, register via
`tauri::generate_handler!`) done — commit `eb52d0419`. Steps 1/3/4
(`agent-runtime-contract` wiring via `MockAdapter`/`OpenCodeAdapter`)
deliberately NOT done: that commit's own message corrects this plan's
earlier overstatement — `get_swarm_topology`/`query_derivation_trace`/
`stream_phoneme_vector` are swarm-mesh-topology and Pāṇinian-derivation
domain, not agent-runtime-lifecycle domain; forcing `agent-runtime-contract`
behind them would have been an artificial connection, not a real one.
`SwarmDashboardState::placeholder()` (one real node, empty
traces/phonemes — proves both the happy path and the "not found" error
path) stands in its place.

**Real, CI-verified end-to-end result, not a claim:** the Windows build
was broken in 4 more ways once actually compiled against a real
toolchain (this WSL environment has no `pkg-config`/`glib`, so none of
this was locally checkable — every one of these was found from a real
`gh run view --repo juv4uk/tauricode --log-failed`, one commit +
`workflow_dispatch` re-run at a time, never predicted ahead of
evidence):

1. `bun install --frozen-lockfile=false` — invalid flag syntax (`fix(ci)`, `c64689d9f`)
2. `frontendDist: ".."` captured `src-tauri`/`node_modules` — isolated a `web/` folder (`a524fc6d9`)
3. `icons/icon.ico` missing (tauri-build's Windows Resource requirement) — generated a real 6-resolution ICO via Pillow (`a2af6ac64`)
4. `register_swarm_commands<R: tauri::Runtime>()` generic vs. `stream_phoneme_vector`'s concrete `AppHandle` — real `E0277` rustc error, fixed by dropping the unneeded genericity (`f1332fea2`)
5. `bundle.icon` unset — WiX couldn't find an icon separately from the Resource-file one — wired the existing `icon.ico` into the config key the bundler actually reads (`284884884`)

**Run [`32855223160`](https://github.com/juv4uk/tauricode/actions/runs/32855223160): ✓ green, 5m35s, artifact `tauricode-windows` (MSI + NSIS) uploaded.** First real, working Tauri build for tauricode.

`ecosystem-scheduler` consumption remains a separate, still-open, still
cross-crate decision — not resolved by this, not silently dropped either.
