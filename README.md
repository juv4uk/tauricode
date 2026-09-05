<p align="center">
  <a href="README.md">English</a> |
  <a href="README.uk.md">Українська</a> |
  <a href="README.de.md">Deutsch</a>
</p>

# Tauricode

**See first. Prove next. Act last.**

Tauri-based agent workstation for the my-lisp ecosystem.

Tauricode is an independent project focused on reproducible,
observable and controllable AI-agent execution environments.

It combines:

- Tauri 2 + Rust desktop backend
- SolidJS frontend
- Guix reproducible environments
- WSL/Linux agent execution
- repository / contract / task / evidence observation
- pluggable agent runtimes

## What exists today

- `ecosystem-observer` (Rust crate, `crates/ecosystem-observer/`):
  read-only Git repository snapshots — branch, HEAD, dirty state,
  remotes; `Complete` / `Partial` / `Failed` observation semantics;
  repository-identity hardening (worktree / bare / submodule edge
  cases correctly distinguished, not silently inherited from a parent
  repo); per-probe timeout protection against a wedged git process.
  22 tests against real git fixtures.

That is the entire current implementation. Everything below this
line is target architecture, not shipped behavior.

## What we are building

- A Tauri 2 desktop shell (`packages/desktop-tauri/`, not created
  yet), developed alongside — not replacing — the existing
  Electron desktop inherited from OpenCode, until sufficient feature
  parity is demonstrated.
- Agent-runtime control (launch, lifecycle, permissions) — planned,
  not implemented.
- Contracts/tasks/evidence/Guix observation beyond git state —
  planned, not implemented.

## Architecture

Tauricode owns the workstation and control-plane architecture.

    SolidJS UI
        ↓
    Tauri 2
        ↓
    Rust backend
        ↓
    ecosystem-observer
        ├── Git
        ├── contracts
        ├── tasks
        ├── evidence
        ├── Guix
        └── runtime state

Agent runtimes are adapters:

    Tauricode
        ├── OpenCode adapter
        ├── Claude adapter
        └── future runtimes

## Design principles

### Observe before acting

Tauricode first establishes what is actually true about the
environment before allowing an agent to act on it.

Unknown state must remain visible as unknown.

### Evidence over assumptions

Repository state, contracts, tasks and execution results should be
traceable to concrete sources and reproducible environments.

### Reproducible execution

Guix is the intended environment layer for ecosystem agents.

The target abstraction is:

    agent + repository + Git revision + Guix environment + task + evidence

### Authority boundaries

Tauricode is not an authority for:

- my-lisp language semantics
- cml compiler semantics
- fpga-lisp ISA
- Paninian ontology
- Shiva canon

It observes and orchestrates these domains; their own repositories
and contracts remain authoritative.

## Relationship to my-idea

Per owner-ratified my-idea ADR-003 (2026-08-30), my-idea is a small,
practical IDE for WSM and Tauri projects — its loop is Open project →
Edit → Build or Run → Stop → Read output. System observation, swarm
dashboard, ecosystem knowledge graph and agent control-plane features
belong to `tauricode`, not my-idea; any residual Observatory code in
my-idea is historical and behind the product surface only.

Both may read the same underlying data (tasks, evidence, contracts) —
that overlap is expected and acceptable. What must not overlap is
primary purpose: my-idea does not become a second control plane, and
Tauricode does not become a second IDE/interpretation layer.

## Relationship to OpenCode

Tauricode originated from the OpenCode codebase, but it is now being
developed as an independent project.

OpenCode is treated as:

- an agent-runtime provider
- an API/protocol reference
- a source of selected implementation ideas
- a donor for compatible upstream components

OpenCode is not the architectural authority for Tauricode.

The project currently retains portions of OpenCode's SolidJS
application and runtime code while the Tauricode-owned Tauri/Rust
architecture is developed.

Tauricode is not affiliated with or maintained by the OpenCode team.

## Development roadmap

Stage 1 — Observer _(in progress — repository/git slice shipped;
contracts, tasks, evidence, and Guix observation not yet)_

- repository state
- contracts and drift
- tasks
- evidence
- Guix state
- local runtime observation

Stage 2 — Launcher _(planned)_

- start agents
- enter reproducible Guix environments
- launch runtime adapters

Stage 3 — Controller _(planned)_

- controlled task lifecycle
- agent lifecycle
- explicit permissions and authority boundaries

Stage 4 — Reproducible Agent Workstation _(planned)_

- environment + agent + task + evidence as one reproducible workflow

## Current implementation

    crates/
      ecosystem-observer/

Future:

    packages/
      desktop-tauri/

## Architecture decisions

These design and scope decisions are recorded, not just described in
prose, at `/home/agents/ecosystem/decisions/`:

- `ECO-DECISION-2026-08-19-TAURICODE-ROLE` — role, authority
  boundaries, staged path (observer → launcher → controller →
  reproducible agent workstation)
- `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER` — Stage 1
  acceptance criteria
- `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE` — Tauri
  shell placement, `ecosystem-observer` as a Rust crate, OpenCode as
  a sidecar/adapter

If this README and a decision document ever disagree, the decision
document wins.

## License and attribution

Tauricode is a mixed-provenance distribution. It contains and/or derives
portions of code from OpenCode, while original WSM/Tauricode components are
separately identified and licensed.

Original OpenCode project: [anomalyco/opencode](https://github.com/anomalyco/opencode)

Copyright and license notices from upstream code must be preserved
where required. `LICENSE` contains the upstream OpenCode MIT notice.
`LICENSE-WSM` contains Waldemar Sydiy M's MIT notice for original WSM work.
`NOTICE` records the verified scope and the rule for mixed/unaudited files.

## Ліцензія та атрибуція

Tauricode має змішане походження. Частина коду успадкована або похідна від
OpenCode, а оригінальні компоненти WSM/Tauricode позначаються й ліцензуються
окремо.

- `LICENSE` — upstream MIT notice OpenCode; він не видаляється з похідних
  частин.
- `LICENSE-WSM` — MIT notice Waldemar Sydiy M для оригінальної роботи WSM.
- `NOTICE` — перевірений scope, provenance boundary і правило для змішаних або
  ще не перевірених файлів.

Локальний commit не робить upstream-файл повністю власним. Власна ліцензія
застосовується лише до оригінального внеску; сторонні компоненти зберігають
свої copyright та умови.
