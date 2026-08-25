<p align="center">
  <a href="README.md">English</a> |
  <a href="README.uk.md">Українська</a> |
  <a href="README.de.md">Deutsch</a>
</p>

# Tauricode

**Erst sehen. Dann beweisen. Zuletzt handeln.**

Tauri-basierte Agent-Workstation für das my-lisp-Ökosystem.

Tauricode ist ein eigenständiges Projekt für reproduzierbare,
beobachtbare und kontrollierbare Ausführungsumgebungen für
KI-Agenten.

Es kombiniert:

- Tauri 2 + Rust Desktop-Backend
- SolidJS Frontend
- Guix reproduzierbare Umgebungen
- WSL/Linux Agent-Ausführung
- Beobachtung von Repositories / Contracts / Tasks / Evidence
- austauschbare Agent-Runtimes

## Was heute existiert

- `ecosystem-observer` (Rust-Crate, `crates/ecosystem-observer/`):
  schreibgeschützte Git-Repository-Snapshots — Branch, HEAD,
  Dirty-Status, Remotes; Beobachtungssemantik `Complete` / `Partial` /
  `Failed`; Härtung der Repository-Identifikation (Worktree / Bare /
  Submodule werden korrekt unterschieden, nicht stillschweigend vom
  übergeordneten Repo übernommen); Timeout-Schutz pro Probe gegen
  hängende git-Prozesse. 22 Tests gegen echte git-Fixtures.

Das ist die gesamte aktuelle Implementierung. Alles unterhalb dieser
Zeile ist Zielarchitektur, kein ausgeliefertes Verhalten.

## Was wir gerade bauen

- Eine Tauri-2-Desktop-Shell (`packages/desktop-tauri/`, noch nicht
  angelegt), die parallel zum — nicht anstelle des — bestehenden,
  von OpenCode geerbten Electron-Desktops entwickelt wird, bis eine
  ausreichende Feature-Parität nachgewiesen ist.
- Agent-Runtime-Steuerung (Start, Lebenszyklus, Berechtigungen) —
  geplant, nicht implementiert.
- Beobachtung von Contracts/Tasks/Evidence/Guix über den Git-Status
  hinaus — geplant, nicht implementiert.

## Architektur

Tauricode besitzt die Architektur der Workstation und der Control
Plane.

    SolidJS UI
        ↓
    Tauri 2
        ↓
    Rust Backend
        ↓
    ecosystem-observer
        ├── Git
        ├── contracts
        ├── tasks
        ├── evidence
        ├── Guix
        └── runtime state

Agent-Runtimes sind Adapter:

    Tauricode
        ├── OpenCode-Adapter
        ├── Claude-Adapter
        └── zukünftige Runtimes

## Designprinzipien

### Erst beobachten, dann handeln

Tauricode stellt zuerst fest, was über die Umgebung tatsächlich
zutrifft, bevor einem Agenten erlaubt wird, darauf basierend zu
handeln.

Unbekannter Zustand muss als unbekannt sichtbar bleiben.

### Evidence statt Annahmen

Repository-Zustand, Contracts, Tasks und Ausführungsergebnisse
müssen auf konkrete Quellen und reproduzierbare Umgebungen
zurückführbar sein.

### Reproduzierbare Ausführung

Guix ist die vorgesehene Umgebungsschicht für Ökosystem-Agenten.

Die Ziel-Abstraktion lautet:

    Agent + Repository + Git-Revision + Guix-Umgebung + Task + Evidence

### Authority-Grenzen

Tauricode ist keine Authority für:

- my-lisp-Sprachsemantik
- cml-Compiler-Semantik
- fpga-lisp-ISA
- Paninianische Ontologie
- Shiva-Kanon

Es beobachtet und orchestriert diese Domänen; ihre eigenen
Repositories und Contracts bleiben maßgeblich.

## Verhältnis zu my-idea

Der primäre Zweck von my-idea ist Systembeobachtung / Analyse /
Evidence-Interpretation. Der primäre Zweck von Tauricode ist
Agent-Ausführung / Umgebungs-Lebenszyklus / Task-Steuerung.

Beide können dieselben zugrunde liegenden Daten lesen (Tasks,
Evidence, Contracts) — diese Überschneidung ist erwartet und
zulässig. Was sich nicht überschneiden darf, ist der primäre Zweck:
my-idea wird nicht zu einer zweiten Control Plane, und Tauricode wird
nicht zu einer zweiten Interpretationsschicht.

## Verhältnis zu OpenCode

Tauricode entstand aus der OpenCode-Codebasis, wird aber jetzt als
eigenständiges Projekt weiterentwickelt.

OpenCode wird behandelt als:

- Anbieter einer Agent-Runtime
- API-/Protokoll-Referenz
- Quelle einzelner Implementierungsideen
- Spender kompatibler Upstream-Komponenten

OpenCode ist nicht die architektonische Authority für Tauricode.

Das Projekt behält derzeit Teile der SolidJS-Anwendung und des
Runtime-Codes von OpenCode bei, während die eigene Tauri/Rust-
Architektur von Tauricode entwickelt wird.

Tauricode ist nicht mit dem OpenCode-Team verbunden und wird nicht
von ihm gepflegt.

## Entwicklungs-Roadmap

Stage 1 — Observer _(in Arbeit — Repository/Git-Slice ausgeliefert;
Contracts, Tasks, Evidence und Guix-Beobachtung noch nicht)_

- Repository-Zustand
- Contracts und Drift
- Tasks
- Evidence
- Guix-Zustand
- lokale Runtime-Beobachtung

Stage 2 — Launcher _(geplant)_

- Agenten starten
- reproduzierbare Guix-Umgebungen betreten
- Runtime-Adapter starten

Stage 3 — Controller _(geplant)_

- kontrollierter Task-Lebenszyklus
- Agenten-Lebenszyklus
- explizite Berechtigungen und Authority-Grenzen

Stage 4 — Reproducible Agent Workstation _(geplant)_

- Umgebung + Agent + Task + Evidence als ein reproduzierbarer
  Workflow

## Aktuelle Implementierung

    crates/
      ecosystem-observer/

Zukünftig:

    packages/
      desktop-tauri/

## Architekturentscheidungen

Diese Design- und Scope-Entscheidungen sind nicht nur als Prosa
beschrieben, sondern als Datensätze unter
`/home/agents/ecosystem/decisions/` festgehalten:

- `ECO-DECISION-2026-08-19-TAURICODE-ROLE` — Rolle,
  Authority-Grenzen, gestufter Weg (Observer → Launcher →
  Controller → Reproducible Agent Workstation)
- `ECO-DECISION-2026-08-19-TAURICODE-STAGE1-OBSERVER` —
  Abnahmekriterien für Stage 1
- `ECO-DECISION-2026-08-19-TAURICODE-TAURI-ARCHITECTURE` —
  Platzierung der Tauri-Shell, `ecosystem-observer` als Rust-Crate,
  OpenCode als Sidecar/Adapter

Falls dieses README jemals von einem Entscheidungsdokument abweicht,
gilt das Entscheidungsdokument.

## Lizenz und Attribution

Tauricode enthält und/oder leitet sich aus Teilen des OpenCode-Codes
ab.

Ursprüngliches OpenCode-Projekt: [anomalyco/opencode](https://github.com/anomalyco/opencode)

Copyright- und Lizenzhinweise aus Upstream-Code müssen dort erhalten
bleiben, wo dies erforderlich ist. Siehe `LICENSE` und die
Repository-Historie für Details.
