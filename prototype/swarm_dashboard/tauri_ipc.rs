// ============================================================================
// Tauricode Desktop Workbench - Tauri v2 IPC Command Definitions
// Location: src-tauri/src/commands/swarm_dashboard.rs
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};

/// Status of an active Swarm Node in the mesh
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeStatus {
    ONLINE,
    DEGRADED,
    OFFLINE,
    SYNCING,
}

/// Metadata and real-time telemetry for a Swarm Node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmNode {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub role: String,
    pub repo: String,
    pub layer: String,
    pub status: NodeStatus,
    pub latency_ms: f64,
    pub completed_tasks: u32,
    pub total_tasks: u32,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub cpu_usage_pct: f32,
    pub memory_mb: f32,
    pub uptime_seconds: u64,
    pub last_heartbeat: String,
    pub version: String,
    pub active_peers: Vec<String>,
}

/// Full Swarm Mesh Topology Snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwarmMeshTopology {
    pub cluster_id: String,
    pub cluster_name: String,
    pub active_nodes_count: usize,
    pub total_tasks_completed: u32,
    pub mesh_health_pct: f32,
    pub timestamp: String,
    pub nodes: Vec<SwarmNode>,
    pub connections: Vec<MeshConnection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConnection {
    pub source: String,
    pub target: String,
    pub protocol: String,
    pub bandwidth_kbps: u32,
    pub latency_ms: f64,
    pub active: bool,
}

/// Pāṇinian Rule metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SutraRule {
    pub sutra_id: String,
    pub text_deva: String,
    pub text_slp1: String,
    pub classification: String,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paribhasha_principle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_sutras: Option<Vec<String>>,
}

/// Morphological / Phonological term inside a derivation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationTerm {
    pub id: String,
    pub kind: String,
    pub source_form: String,
    pub surface_form: String,
    pub designations: Vec<String>,
}

/// Immutable state in the Derivation Proof Graph (DAG)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationState {
    pub id: String,
    pub hash: String,
    pub step_index: usize,
    pub schema: String,
    pub terms: Vec<DerivationTerm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_rule: Option<SutraRule>,
    pub mutation_type: Option<String>,
    pub proof_verified: bool,
}

/// Complete Derivation Trace payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationTrace {
    pub ir_version: String,
    pub derivation_id: String,
    pub target_word: String,
    pub description: String,
    pub status: String,
    pub final_surface_form: String,
    pub root: String,
    pub rules: Vec<SutraRule>,
    pub states: Vec<DerivationState>,
    pub cryptographic_proof: ProofVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerification {
    pub root_hash: String,
    pub terminal_hash: String,
    pub algorithm: String,
    pub verified: bool,
}

/// Articulatory phonetic vector (PVC-16 & 64-bit Pratyāhāra)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhonemeVectorData {
    pub phoneme: String,
    pub slp1: String,
    pub deva: String,
    pub upc8: u8,
    pub upc8_hex: String,
    pub pvc16_raw: u16,
    pub pvc16_hex: String,
    pub is_vowel: bool,
    pub sthana_name: String,
    pub prayatna_name: String,
    pub is_palatalized: bool,
    pub pratyahara_bit_index: u8,
    pub pratyahara_mask_u64: String,
    pub pratyaharas_contained: Vec<String>,
    pub is_ukrainian: bool,
    pub ipa: String,
}

/// Swarm Dashboard Managed State inside Tauri App
pub struct SwarmDashboardState {
    pub topology: Mutex<SwarmMeshTopology>,
    pub traces: Mutex<HashMap<String, DerivationTrace>>,
    pub phonemes: Mutex<HashMap<String, PhonemeVectorData>>,
}

// ----------------------------------------------------------------------------
// Tauri v2 Command Implementations
// ----------------------------------------------------------------------------

/// IPC Command 1: Get the current Swarm Topology across ports 9101-9107
#[tauri::command]
pub async fn get_swarm_topology(
    state: State<'_, SwarmDashboardState>,
) -> Result<SwarmMeshTopology, String> {
    let topology = state.topology.lock().map_err(|e| e.to_string())?;
    Ok(topology.clone())
}

/// IPC Command 2: Query a Pāṇinian Derivation Trace Proof Graph by ID
#[tauri::command]
pub async fn query_derivation_trace(
    derivation_id: String,
    state: State<'_, SwarmDashboardState>,
) -> Result<DerivationTrace, String> {
    let traces = state.traces.lock().map_err(|e| e.to_string())?;
    traces
        .get(&derivation_id)
        .cloned()
        .ok_or_else(|| format!("Derivation trace not found for ID: {}", derivation_id))
}

/// IPC Command 3: Stream and inspect phonetic vector representations
#[tauri::command]
pub async fn stream_phoneme_vector(
    phoneme_or_code: String,
    app: AppHandle,
    state: State<'_, SwarmDashboardState>,
) -> Result<PhonemeVectorData, String> {
    let phonemes = state.phonemes.lock().map_err(|e| e.to_string())?;
    let data = phonemes
        .get(&phoneme_or_code)
        .cloned()
        .ok_or_else(|| format!("Phoneme data not found for: {}", phoneme_or_code))?;

    // Emit live event to all listening Tauri frontends
    let _ = app.emit("telemetry://phoneme-vector-stream", &data);

    Ok(data)
}

/// Helper function to register commands with Tauri v2 Builder
pub fn register_swarm_commands<R: tauri::Runtime>() -> impl Fn(tauri::ipc::Invoke<R>) -> bool {
    tauri::generate_handler![
        get_swarm_topology,
        query_derivation_trace,
        stream_phoneme_vector
    ]
}
