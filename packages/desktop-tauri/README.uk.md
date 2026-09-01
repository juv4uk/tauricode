# Tauricode Desktop Tauri — українська версія

Це офіційний read-only desktop shell Tauricode на Tauri 2. Він є оригінальною
роботою WSM/Tauricode у перевіреному scope `packages/desktop-tauri/`; OpenCode-
derived частини репозиторію зберігають окремий upstream MIT notice. Сам Tauri,
Rust dependencies, generated lock data та tool-generated assets не
оголошуються винятковою власністю WSM.

## Архітектурна межа

```text
public ecosystem-observer
        ↓ canonical read-only snapshot
Tauricode Tauri command
        ↓
plain webview UI
```

Tauricode показує snapshot, але не володіє його семантикою. Observer не змінює
репозиторії, не запускає агентів і не виносить доменні вироки.

Поточний UI показує:

- Git-стан репозиторіїв із явними `Complete / Partial / Failed`;
- локальні процеси та окремий self-reported identity status;
- topics із явно налаштованого `guard-reference.wsm` і наявність `guard-ask`;
- присутність старих Guard paths без твердження, що вони активні;
- реально живі `swarm-node` процеси без твердження про delivery чи mesh
  convergence.

Ключові розрізнення:

```text
path present != component active
process live != message delivered
socket write != peer accepted
peer accepted != mesh converged
observation != judgement
```

## Конфігурація

- `ECOSYSTEM_ROOT` — батьківський каталог репозиторіїв;
- `ECOSYSTEM_REPOS` — comma-separated список репозиторіїв;
- `ECOSYSTEM_GUARD_REFERENCE` — явний шлях до `guard-reference.wsm`;
- `ECOSYSTEM_COORDINATION_ROOT` — coordination repo для спостереження legacy
  Guard paths.

Observer pinned на точний public Git commit, тому рухома branch не може
мовчки змінити snapshot contract.

## Що ще не доведено

Guix state, tasks, evidence та повне wiring contract facts ще не входять у
snapshot. Локальний Tauri test gate потребує системних GLib/WebKitGTK
development packages; відсутність `glib-2.0.pc` є environment blocker, а не
доказом помилки Rust-коду. Остаточний build gate виконує CI з Tauri
dependencies.
