# ecosystem/ scaffold (Swarm Contract v0.1, TAURICODE-SWARM-CONTRACT-01)

Per `repo.my`'s `(imports language-contract swarm-contract)`: tauricode
consumes both as versioned contracts, not hypotheses —
`my-lisp/language-contract.my` (currently major 2, read from the file,
never from prose) governs every `.my` file this repo emits or parses
(`crates/swarm-cli/src/tasks_file.rs`, `crates/ecosystem-scheduler`,
both ported from `my-lisp/crates/swarm-node` for format parity), and
the Swarm Contract v0.1 (`my-lisp/docs/swarm-mesh-v2.md`) governs the
coordination-plane behavior of those adapters: read-only observation
by default, mutating task ops transported with full response
read-back while quorum-guarded authority stays in swarm-node.

No `imports/*.my` claim files are populated here — same principle as
fpga-lisp's scaffold: an empty placeholder would be worse than an
explanation of why it's empty. The durable records of contract
conformance live in this repo's own test suites (19 scheduler tests,
10+ adapter/e2e tests) and in evidence committed alongside the crates.
