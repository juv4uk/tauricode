;; guix shell -m manifest.scm --pure
;; Toolchain for tauricode's Rust crates (swarm-cli, ecosystem-scheduler,
;; ecosystem-observer) plus git/TLS for cargo. The TypeScript side is
;; deliberately NOT here: guix does not package bun, and node would be a
;; silent substitute for a different runtime. Instead bun is pinned by
;; `.bun-version` (1.3.14 = package.json's packageManager field) and
;; verified by scripts/env-check.sh — see that script for the layering
;; contract. `--pure` is the point: it catches accidental dependencies on
;; whatever the ambient profile happens to provide (same lesson as
;; cml/manifest.scm's CML-MANIFEST-COMPLETE note).
(specifications->manifest
 '("rust"
   "rust:cargo"
   "git"
   "nss-certs"
   "gcc-toolchain"
   "make"
   ;; --pure strips the ambient PATH: the shell itself and coreutils must
   ;; be declared, not inherited (env-check.sh runs under this shell).
   "bash"
   "coreutils"))
