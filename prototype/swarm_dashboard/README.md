# Tauricode Swarm Mesh Dashboard & Telemetry Workbench

**Author:** TauriCode Desktop Workbench & Telemetry Agent  
**Date:** 2026-08-21  
**Status:** `ACTIVE PROTOTYPE / P5 GATE DELIVERABLE`  
**Epistemic Layer:** Layer 6 (Developer Workbench, Desktop UI & Real-Time Telemetry)  
**Location:** `prototype/swarm_dashboard/`

---

## 0. Boundary note (added 2026-08-25)

This prototype is **experimental UI/prototyping work**. It is **not**
the authoritative implementation of
`ECO-DECISION-2026-08-19-TAURICODE-ROLE` (the ecosystem's ACCEPTED
`agent-runtime-ui` role for tauricode), **not** an `EcosystemSnapshot`
authority, and **not** a replacement for the `ecosystem-observer` crate
(`crates/ecosystem-observer/`, already implemented as an isolated core —
`snapshot.rs`/`git_read.rs`/`discover.rs`/`process_observe.rs` — the
real Stage 1 read-only observer logic).

Everything this prototype displays (the 6-node mesh, task counts,
derivation traces, phoneme table) is fixture/mock data
(`fixtures.ts` / `SwarmDashboardState::full_fixtures()`), never a real
read of any repository's git state, Guix manifest, contract, or
`tasks.my`. Its build/release (including the `t0.0.2` Windows/Linux
artifacts) was authorized as its own scoped task, separately from — and
without pre-empting — the ACCEPTED architecture's own future
`packages/desktop-tauri/` integration. Promoting any piece of this
prototype into that integration is a separate decision, not automatic.

---

## 1. Overview & Strategic Purpose

The **Tauricode Swarm Mesh Dashboard** provides a high-performance desktop telemetry workbench built for **Tauri v2**. It bridges the multi-agent P2P swarm mesh operating across ports **:9101 through :9107**, providing real-time node monitoring, live Pāṇinian Derivation Directed Acyclic Graph (DAG) streaming, and interactive phonetic articulatory inspection (PVC-16 and 64-bit Pratyāhāra ALU).

---

## 2. Active Swarm Mesh Topology (:9101 - :9107)

The mesh interconnects 6 autonomous specialized nodes:

| Node ID                | TCP/IPC Port | Specialization / Role                                 | Epistemic Layer                    | Tasks Completed |
| ---------------------- | ------------ | ----------------------------------------------------- | ---------------------------------- | --------------- |
| **`my-lisp-1`**        | **`:9101`**  | Core Lisp VM / Semantic Oracle / Knowledge Store      | Layer 6 (VM) / Layer 5 (Proof)     | **82**          |
| **`fpga-lisp-1`**      | **`:9102`**  | Hardware Synthesizer / ALU & Accelerator Co-processor | Layer 6 (FPGA RTL)                 | **38**          |
| **`cml-1`**            | **`:9103`**  | CML Compiler Architecture & Lowering Middle-End       | Layer 6 (Compiler / Hardware)      | **44**          |
| **`my-idea-1`**        | **`:9104`**  | IDE & Visual Tooling Workbench / System Observatory   | Layer 6 (Workbench UI)             | **41**          |
| **`my-lisp-panini-1`** | **`:9106`**  | Panini Grammar Machine / Derivation IR Engine         | Layer 2 (Grammar) / Layer 5 (DAG)  | **46**          |
| **`shiva-sutras-1`**   | **`:9107`**  | Phonetic Engine / Śiva Sūtras, UPC-8 & PVC-16 Canon   | Layer 1 (Canon) / Layer 6 (Codecs) | **47**          |

**Total Mesh Completed Tasks:** **298 completed tasks** (exceeding the 271+ task target).

---

## 3. Tauri v2 IPC Command & Event Interface

The Rust backend (`tauri_ipc.rs`) registers 3 core asynchronous commands and event emitters:

```rust
// In src-tauri/src/commands/swarm_dashboard.rs

#[tauri::command]
pub async fn get_swarm_topology(
    state: State<'_, SwarmDashboardState>,
) -> Result<SwarmMeshTopology, String>;

#[tauri::command]
pub async fn query_derivation_trace(
    derivation_id: String,
    state: State<'_, SwarmDashboardState>,
) -> Result<DerivationTrace, String>;

#[tauri::command]
pub async fn stream_phoneme_vector(
    phoneme_or_code: String,
    app: AppHandle,
    state: State<'_, SwarmDashboardState>,
) -> Result<PhonemeVectorData, String>;
```

### TypeScript Client Bridge (`tauri_ipc.ts`)

The client bridge provides type-safe access with automatic detection of Tauri Webview environment vs. browser fallback:

```typescript
import { getSwarmTopology, queryDerivationTrace, streamPhonemeVector } from "./tauri_ipc"

const topology = await getSwarmTopology()
const trace = await queryDerivationTrace("bhavati")
const phoneme = await streamPhonemeVector("a")
```

---

## 4. UI Component Architecture

1. **`SwarmMeshGraph.tsx`:** Interactive canvas rendering the 6 nodes in a hexagonal ring with animated packet travel and connection bandwidth indicators.
2. **`NodeHealthMatrix.tsx`:** Real-time health cards displaying heartbeat latencies, CPU/memory usage, active peers, and advertised capabilities.
3. **`TaskCompletionTelemetry.tsx`:** Analytics dashboard tracking 298+ completed tasks, capability specialization breakdown, and recent audit logs.
4. **`DerivationDagStreamer.tsx`:** Live Pāṇinian proof graph visualizer with step playback, AST morphology diffing, Sūtra citation, and SHA-256 state proof verification.
5. **`PhoneticInspector.tsx`:** 16-bit PVC-16 articulatory register explorer, Sūtra 1.1.9 Savarṇa homogeneity comparator (1 clock cycle), and Ukrainian softened consonant modifier inspection.

---

## 5. Running the Prototype

### Standalone Browser Mode (Zero Dependencies)

Open `index.html` directly in any modern browser:

```bash
# Using Python built-in server or opening directly:
python3 -m http.server 8080
# Open http://localhost:8080/index.html
```

### Running Automated Test Suite

```bash
node test_swarm_dashboard.mjs
```

---

## 6. Epistemic Integrity & Compliance

- **ECA-007 Compliance:** All derivation transitions are sealed with SHA-256 content digests (`state:sha256:...`).
- **Paribhāṣā Conflict Resolution:** Demonstrates deterministic rule precedence (e.g., _Apavāda > Utsarga_ where Sūtra 2.4.75 _Ślu_ blocks 3.1.68 _Śap_ in `dadāti`).
- **Phonetic Fidelity:** Articulatory parameters adhere to the ratified **ADR-002** standard and **PVC-16** specification.
