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
*existing* processes; `agent-runtime-contract` is the *adapter* boundary
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
  audit finding: "runtime handle must be opaque"); 3) reuse the *shape*
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
  `HandleRegistry` (from M2.1) plus a scripted event queue per handle;
  2) implement all 7 trait methods purely in-memory, no subprocess, no
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

## M2.5 — OpenCode adapter minimal launch/stop/status

- **goal (not attempted this session — see "Why this increment order"
  above):** implement `AgentRuntimeAdapter` for a real `opencode serve`
  subprocess: `launch` spawns it, `status` polls liveness, `stop` kills
  it cleanly.
- **evidence required before starting, next session:** a fresh
  `AGENT-RESOURCE-POLICY.md` preflight check (current machine load/RAM),
  and explicit confirmation that WSL has been stable (no unexplained
  freeze) for a reasonable window beforehand — this increment is exactly
  the class of subprocess-lifecycle work this session found real risk
  in.
- **rollback boundary:** an entirely new file
  (`crates/agent-runtime-contract/src/opencode_adapter.rs` or a separate
  crate) — does not touch M2.0-M2.4 if reverted.
- **done criteria (deferred):** not defined further until attempted with
  fresh evidence; defining it now would be speculative design ahead of
  the actual subprocess behavior.

## M2.6-M2.9 — deferred

`OpenCode event normalization`, `task binding/provenance`, `dashboard/
scheduler consumption`, `second adapter proof` all depend on M2.5 landing
first (there is nothing to normalize events *from* without a real
adapter, and "second adapter proof" is only meaningful once the first
real one exists to compare against). Listing done-criteria for these now
would be invented, not evidenced — deferred per this plan's own
`Unknown > invented` rule.

---

## This session's execution scope

M2.0 through M2.4 — the complete, self-contained proof that "Core does
not need to know which runtime is under it," using zero subprocesses and
therefore zero WSL-stability risk. M2.5 onward explicitly deferred with
reasons stated above, not silently dropped.
