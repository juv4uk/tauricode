; repo.my — Swarm Contract v0.1 scope declaration for tauricode.
; See my-lisp/docs/swarm-mesh-v2.md for the full spec. Format confirmed
; by example against fpga-lisp/repo.my and shiva-sutras/repo.my.
;
; A declaration of scope, not an authorization grant — authorities/
; non-authorities state what this repo is and is not the source of
; truth for, so other repos' agents don't have to re-derive it.

(repository
  (id tauricode)
  (role agent-workstation)
  (exports swarm-cli ecosystem-scheduler opencode-fork cli protocol sdk)
  (imports language-contract swarm-contract)
  (capabilities tauricode typescript bun rust gui control-plane adapters docs testing)
  (authorities agent-workstation-ui coordination-plane-adapters opencode-fork-codebase)
  (non-authorities language-semantics sanskrit-canon shiva-source-provenance
    paninian-ontology fpga-isa cml-compilation swarm-protocol-semantics))
