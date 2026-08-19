<p align="center">
  <a href="README.md">English</a> |
  <a href="README.uk.md">Українська</a> |
  <a href="README.de.md">Deutsch</a>
</p>

# Tauricode

**Спочатку побач. Потім доведи. Дій останнім.**

Tauri-застосунок — agent workstation для екосистеми my-lisp.

Tauricode — незалежний проєкт, зосереджений на відтворюваних,
спостережуваних і контрольованих середовищах виконання AI-агентів.

Він поєднує:

- Tauri 2 + Rust desktop-бекенд
- SolidJS фронтенд
- Guix відтворювані середовища
- WSL/Linux виконання агентів
- спостереження за репозиторіями / контрактами / задачами / evidence
- підключувані agent runtime

## Що вже існує

- `ecosystem-observer` (Rust-крейт, `crates/ecosystem-observer/`):
  read-only знімки стану Git-репозиторіїв — гілка, HEAD, dirty-стан,
  remotes; семантика спостереження `Complete` / `Partial` / `Failed`;
  hardening ідентифікації репозиторію (worktree / bare / submodule
  коректно розрізняються, не успадковують стан батьківського репо
  мовчки); захист таймаутом на кожен probe проти зависання git-процесу.
  22 тести проти реальних git-фікстур.

Це вся поточна реалізація. Усе нижче цього рядка — цільова
архітектура, не реалізована поведінка.

## Що ми будуємо

- Desktop-шар на Tauri 2 (`packages/desktop-tauri/`, ще не створено),
  розробляється паралельно — не замінюючи — наявний Electron-desktop,
  успадкований від OpenCode, доки не буде доведено достатній feature
  parity.
- Керування agent runtime (запуск, життєвий цикл, дозволи) —
  заплановано, не реалізовано.
- Спостереження за contracts/tasks/evidence/Guix поза git-станом —
  заплановано, не реалізовано.

## Архітектура

Tauricode володіє архітектурою workstation і control plane.

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

Agent runtime — це адаптери:

    Tauricode
        ├── OpenCode adapter
        ├── Claude adapter
        └── майбутні runtime

## Принципи проєктування

### Спочатку спостерігати, потім діяти

Tauricode спершу встановлює, що насправді істинне про середовище,
перш ніж дозволити агенту діяти на його основі.

Невідомий стан має лишатись видимим як невідомий.

### Evidence понад припущення

Стан репозиторіїв, контракти, задачі й результати виконання мають
бути простежувані до конкретних джерел і відтворюваних середовищ.

### Відтворюване виконання

Guix — цільовий шар середовища для агентів екосистеми.

Цільова абстракція:

    агент + репозиторій + Git revision + Guix-середовище + задача + evidence

### Межі authority

Tauricode не є authority для:

- семантики мови my-lisp
- семантики компілятора cml
- ISA fpga-lisp
- paninian ontology
- shiva canon

Він спостерігає й оркеструє ці домени; їхні власні репозиторії й
контракти лишаються авторитетними.

## Відношення до my-idea

Головна мета my-idea — system observation / analysis / evidence
interpretation. Головна мета Tauricode — agent execution / environment
lifecycle / task control.

Обидва можуть читати ті самі первинні дані (tasks, evidence, contracts)
— цей перетин очікуваний і прийнятний. Що не повинно перетинатись —
primary purpose: my-idea не стає другим control plane, а Tauricode не
стає другим шаром інтерпретації.

## Відношення до OpenCode

Tauricode походить з кодової бази OpenCode, але зараз розробляється як
незалежний проєкт.

OpenCode розглядається як:

- постачальник agent runtime
- референс API/протоколу
- джерело окремих ідей реалізації
- донор сумісних upstream-компонентів

OpenCode не є архітектурним authority для Tauricode.

Проєкт наразі зберігає частини SolidJS-застосунку й runtime-коду
OpenCode, поки розробляється власна Tauri/Rust-архітектура Tauricode.

Tauricode не афілійований з командою OpenCode і не підтримується нею.

## Дорожня карта розвитку

Stage 1 — Observer *(у процесі — repository/git-зріз реалізовано;
contracts, tasks, evidence і Guix-спостереження — ще ні)*
- стан репозиторію
- контракти й дрейф
- задачі
- evidence
- стан Guix
- спостереження локального runtime

Stage 2 — Launcher *(заплановано)*
- запуск агентів
- вхід у відтворювані Guix-середовища
- запуск runtime-адаптерів

Stage 3 — Controller *(заплановано)*
- контрольований життєвий цикл задач
- життєвий цикл агентів
- явні дозволи й межі authority

Stage 4 — Reproducible Agent Workstation *(заплановано)*
- середовище + агент + задача + evidence як єдиний відтворюваний workflow

## Поточна реалізація

    crates/
      ecosystem-observer/

Майбутнє:

    packages/
      desktop-tauri/

## Архітектурні рішення

Ці рішення щодо дизайну й скоупу зафіксовані не лише в prose, а як
записи в `/home/agents/ecosystem/decisions/`:

- `ECO-DECISION-2026-08-19-TAURICODE-ROLE` — роль, межі authority,
  поетапний шлях (observer → launcher → controller → reproducible
  agent workstation)
- `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER` — acceptance
  criteria Stage 1
- `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE` —
  розміщення Tauri-шару, `ecosystem-observer` як Rust-крейт, OpenCode
  як sidecar/adapter

Якщо цей README колись розійдеться з decision-документом — рішення
має пріоритет.

## Ліцензія й атрибуція

Tauricode містить та/або похідний від частин коду OpenCode.

Оригінальний проєкт OpenCode: [anomalyco/opencode](https://github.com/anomalyco/opencode)

Копірайт і ліцензійні примітки з upstream-коду мають зберігатись там,
де це вимагається. Див. `LICENSE` та історію репозиторію.
