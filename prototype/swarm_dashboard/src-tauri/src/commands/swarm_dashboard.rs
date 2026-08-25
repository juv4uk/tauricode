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

impl SwarmDashboardState {
    /// Minimal placeholder seed data, deliberately NOT a full port of the
    /// browser-mode fixtures (`../../fixtures.ts`, ~789 lines describing
    /// all 6 real mesh nodes + Pāṇinian traces + phoneme table). This
    /// exists only to prove the IPC wiring compiles and returns a real,
    /// non-empty response through a real Tauri window — porting every
    /// fixture value is a separate, larger increment, not silently
    /// done here. One node, no traces, no phonemes: `get_swarm_topology`
    /// has something real to return; `query_derivation_trace` and
    /// `stream_phoneme_vector` correctly return their "not found" error
    /// path against an empty map, which is itself a real, useful check
    /// that the error path in these commands actually works end to end.
    pub fn placeholder() -> Self {
        let node = SwarmNode {
            id: "node:my-lisp-1".to_string(),
            name: "my-lisp-1".to_string(),
            port: 9101,
            role: "Core Lisp VM / Semantic Oracle / Knowledge Store".to_string(),
            repo: "my-lisp".to_string(),
            layer: "Layer 6 (VM & Runtime) / Layer 5 (Proof Engine)".to_string(),
            status: NodeStatus::ONLINE,
            latency_ms: 1.2,
            completed_tasks: 82,
            total_tasks: 82,
            capabilities: vec!["eval".to_string(), "lisp".to_string(), "vm".to_string()],
            endpoint: "tcp://127.0.0.1:9101".to_string(),
            cpu_usage_pct: 14.5,
            memory_mb: 128.4,
            uptime_seconds: 864200,
            last_heartbeat: "2026-08-21T09:29:58+03:00".to_string(),
            version: "v0.9.4-p5".to_string(),
            active_peers: vec![],
        };

        let topology = SwarmMeshTopology {
            cluster_id: "swarm:mylisp-mesh-p5-alpha".to_string(),
            cluster_name: "My-Lisp Autonomous Ecosystem Mesh".to_string(),
            active_nodes_count: 1,
            total_tasks_completed: 82,
            mesh_health_pct: 100.0,
            timestamp: "2026-08-21T09:30:00+03:00".to_string(),
            nodes: vec![node],
            connections: vec![],
        };

        Self {
            topology: Mutex::new(topology),
            traces: Mutex::new(HashMap::new()),
            phonemes: Mutex::new(HashMap::new()),
        }
    }

    /// Full port of the browser-mode fixtures (`../../fixtures.ts`): all 6
    /// real mesh nodes + their connections, both canonical Pāṇinian
    /// derivation traces (bhavati, dadati), and the 5-entry phoneme
    /// registry (a, i, u, k, t_palatal). Data-only transcription, camelCase
    /// TS fields mapped to the existing snake_case Rust struct fields
    /// above — no struct changes. Two TS shapes have no Rust counterpart
    /// and are silently dropped rather than invented here: `TaskStats`
    /// (`MOCK_TASK_STATS`, no corresponding field anywhere in
    /// `SwarmDashboardState`) and, per phoneme entry, everything nested
    /// under `pvc16` beyond what `PhonemeVectorData` already models
    /// (`sthana`/`prayatna`'s individual boolean flags, `svara`, and
    /// `modifier.isExtension` all exist only in the TS source).
    /// `DerivationState.diff` and `DerivationTerm.it_tags`/`anubandhas`
    /// are dropped the same way (no Rust field to hold them).
    pub fn full_fixtures() -> Self {
        let nodes = vec![
            SwarmNode {
                id: "node:my-lisp-1".to_string(),
                name: "my-lisp-1".to_string(),
                port: 9101,
                role: "Core Lisp VM / Semantic Oracle / Knowledge Store".to_string(),
                repo: "my-lisp".to_string(),
                layer: "Layer 6 (VM & Runtime) / Layer 5 (Proof Engine)".to_string(),
                status: NodeStatus::ONLINE,
                latency_ms: 1.2,
                completed_tasks: 82,
                total_tasks: 82,
                capabilities: vec![
                    "eval", "lisp", "vm", "ast", "proof", "oracle", "alist-parser",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                endpoint: "tcp://127.0.0.1:9101".to_string(),
                cpu_usage_pct: 14.5,
                memory_mb: 128.4,
                uptime_seconds: 864200,
                last_heartbeat: "2026-08-21T09:29:58+03:00".to_string(),
                version: "v0.9.4-p5".to_string(),
                active_peers: vec![
                    "fpga-lisp-1",
                    "cml-1",
                    "my-idea-1",
                    "my-lisp-panini-1",
                    "shiva-sutras-1",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            SwarmNode {
                id: "node:fpga-lisp-1".to_string(),
                name: "fpga-lisp-1".to_string(),
                port: 9102,
                role: "Hardware Synthesizer / ALU & Accelerator Co-processor".to_string(),
                repo: "fpga-lisp".to_string(),
                layer: "Layer 6 (Hardware Synthesis & RTL)".to_string(),
                status: NodeStatus::ONLINE,
                latency_ms: 0.8,
                completed_tasks: 38,
                total_tasks: 38,
                capabilities: vec![
                    "verilog",
                    "fpga",
                    "alu",
                    "synthesis",
                    "hardware",
                    "pvc16-alu",
                    "pratyahara-rom",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                endpoint: "tcp://127.0.0.1:9102".to_string(),
                cpu_usage_pct: 8.2,
                memory_mb: 64.1,
                uptime_seconds: 864200,
                last_heartbeat: "2026-08-21T09:29:59+03:00".to_string(),
                version: "v0.4.1-tang25k".to_string(),
                active_peers: vec!["my-lisp-1", "cml-1", "shiva-sutras-1"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            SwarmNode {
                id: "node:cml-1".to_string(),
                name: "cml-1".to_string(),
                port: 9103,
                role: "CML Compiler Architecture & Lowering Middle-End".to_string(),
                repo: "cml".to_string(),
                layer: "Layer 6 (Compiler & Hardware Co-Design)".to_string(),
                status: NodeStatus::ONLINE,
                latency_ms: 1.5,
                completed_tasks: 44,
                total_tasks: 44,
                capabilities: vec![
                    "compiler",
                    "rust",
                    "lowering",
                    "proof",
                    "phonetic",
                    "constant-folding",
                    "c99-emitter",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                endpoint: "tcp://127.0.0.1:9103".to_string(),
                cpu_usage_pct: 21.0,
                memory_mb: 195.8,
                uptime_seconds: 864200,
                last_heartbeat: "2026-08-21T09:29:57+03:00".to_string(),
                version: "v0.8.2-lowering".to_string(),
                active_peers: vec![
                    "my-lisp-1",
                    "fpga-lisp-1",
                    "my-lisp-panini-1",
                    "shiva-sutras-1",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
            SwarmNode {
                id: "node:my-idea-1".to_string(),
                name: "my-idea-1".to_string(),
                port: 9104,
                role: "IDE & Visual Tooling Agent / System Observatory".to_string(),
                repo: "my-idea".to_string(),
                layer: "Layer 6 (Workbench & Developer Tooling)".to_string(),
                status: NodeStatus::ONLINE,
                latency_ms: 1.8,
                completed_tasks: 41,
                total_tasks: 41,
                capabilities: vec![
                    "ide",
                    "visualizer",
                    "clojurescript",
                    "tauri",
                    "codemirror",
                    "dag-inspector",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                endpoint: "tcp://127.0.0.1:9104".to_string(),
                cpu_usage_pct: 18.7,
                memory_mb: 245.2,
                uptime_seconds: 864200,
                last_heartbeat: "2026-08-21T09:29:58+03:00".to_string(),
                version: "v0.7.0-observatory".to_string(),
                active_peers: vec!["my-lisp-1", "my-lisp-panini-1", "shiva-sutras-1"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            SwarmNode {
                id: "node:my-lisp-panini-1".to_string(),
                name: "my-lisp-panini-1".to_string(),
                port: 9106,
                role: "Panini Grammar Machine / Derivation IR Engine".to_string(),
                repo: "my-lisp-panini".to_string(),
                layer: "Layer 2 (Grammar Mechanics) / Layer 5 (Derivation Proofs)".to_string(),
                status: NodeStatus::ONLINE,
                latency_ms: 2.1,
                completed_tasks: 46,
                total_tasks: 46,
                capabilities: vec![
                    "grammar",
                    "panini",
                    "derivation",
                    "proof-graph",
                    "ir",
                    "paribhasha",
                    "ganapatha",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                endpoint: "tcp://127.0.0.1:9106".to_string(),
                cpu_usage_pct: 28.4,
                memory_mb: 312.0,
                uptime_seconds: 864200,
                last_heartbeat: "2026-08-21T09:29:59+03:00".to_string(),
                version: "v0.5.2-ir-v0.1".to_string(),
                active_peers: vec!["my-lisp-1", "cml-1", "my-idea-1", "shiva-sutras-1"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
            SwarmNode {
                id: "node:shiva-sutras-1".to_string(),
                name: "shiva-sutras-1".to_string(),
                port: 9107,
                role: "Phonetic Engine / Śiva Sūtras & UPC-8 / PVC-16 Canon".to_string(),
                repo: "shiva-sutras".to_string(),
                layer: "Layer 1 (Canonical Text) / Layer 6 (Phonetic Codecs)".to_string(),
                status: NodeStatus::ONLINE,
                latency_ms: 0.9,
                completed_tasks: 47,
                total_tasks: 47,
                capabilities: vec![
                    "phonetics",
                    "shiva",
                    "upc8",
                    "canon",
                    "pratyahara",
                    "slavic-phonetics",
                    "tokenizers",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
                endpoint: "tcp://127.0.0.1:9107".to_string(),
                cpu_usage_pct: 11.2,
                memory_mb: 88.5,
                uptime_seconds: 864200,
                last_heartbeat: "2026-08-21T09:29:58+03:00".to_string(),
                version: "v1.0.0-adr002".to_string(),
                active_peers: vec![
                    "my-lisp-1",
                    "fpga-lisp-1",
                    "cml-1",
                    "my-idea-1",
                    "my-lisp-panini-1",
                ]
                .into_iter()
                .map(String::from)
                .collect(),
            },
        ];

        let connections = vec![
            MeshConnection {
                source: "my-lisp-1".to_string(),
                target: "fpga-lisp-1".to_string(),
                protocol: "TCP".to_string(),
                bandwidth_kbps: 10000,
                latency_ms: 0.8,
                active: true,
            },
            MeshConnection {
                source: "my-lisp-1".to_string(),
                target: "cml-1".to_string(),
                protocol: "TCP".to_string(),
                bandwidth_kbps: 10000,
                latency_ms: 1.5,
                active: true,
            },
            MeshConnection {
                source: "my-lisp-1".to_string(),
                target: "my-idea-1".to_string(),
                protocol: "TCP".to_string(),
                bandwidth_kbps: 10000,
                latency_ms: 1.8,
                active: true,
            },
            MeshConnection {
                source: "my-lisp-1".to_string(),
                target: "my-lisp-panini-1".to_string(),
                protocol: "TCP".to_string(),
                bandwidth_kbps: 10000,
                latency_ms: 2.1,
                active: true,
            },
            MeshConnection {
                source: "my-lisp-1".to_string(),
                target: "shiva-sutras-1".to_string(),
                protocol: "TCP".to_string(),
                bandwidth_kbps: 10000,
                latency_ms: 0.9,
                active: true,
            },
            MeshConnection {
                source: "cml-1".to_string(),
                target: "fpga-lisp-1".to_string(),
                protocol: "IPC".to_string(),
                bandwidth_kbps: 25000,
                latency_ms: 0.4,
                active: true,
            },
            MeshConnection {
                source: "cml-1".to_string(),
                target: "shiva-sutras-1".to_string(),
                protocol: "IPC".to_string(),
                bandwidth_kbps: 25000,
                latency_ms: 0.5,
                active: true,
            },
            MeshConnection {
                source: "my-lisp-panini-1".to_string(),
                target: "shiva-sutras-1".to_string(),
                protocol: "TCP".to_string(),
                bandwidth_kbps: 10000,
                latency_ms: 1.1,
                active: true,
            },
            MeshConnection {
                source: "my-idea-1".to_string(),
                target: "my-lisp-panini-1".to_string(),
                protocol: "WEBSOCKET".to_string(),
                bandwidth_kbps: 15000,
                latency_ms: 1.6,
                active: true,
            },
        ];

        let topology = SwarmMeshTopology {
            cluster_id: "swarm:mylisp-mesh-p5-alpha".to_string(),
            cluster_name: "My-Lisp Autonomous Ecosystem Mesh".to_string(),
            active_nodes_count: 6,
            total_tasks_completed: 298,
            mesh_health_pct: 100.0,
            timestamp: "2026-08-21T09:30:00+03:00".to_string(),
            nodes,
            connections,
        };

        let mut traces = HashMap::new();
        traces.insert("bhavati".to_string(), Self::bhavati_trace());
        traces.insert("dadati".to_string(), Self::dadati_trace());

        let mut phonemes = HashMap::new();
        phonemes.insert("a".to_string(), Self::phoneme_a());
        phonemes.insert("i".to_string(), Self::phoneme_i());
        phonemes.insert("u".to_string(), Self::phoneme_u());
        phonemes.insert("k".to_string(), Self::phoneme_k());
        phonemes.insert("t_palatal".to_string(), Self::phoneme_t_palatal());

        Self {
            topology: Mutex::new(topology),
            traces: Mutex::new(traces),
            phonemes: Mutex::new(phonemes),
        }
    }

    fn bhavati_trace() -> DerivationTrace {
        DerivationTrace {
            ir_version: "panini-derivation-ir/0.1".to_string(),
            derivation_id: "drv:canonical:bhavati-v0.1".to_string(),
            target_word: "भवति (bhavati)".to_string(),
            description: "Derivation of root √bhū (भू सत्तायाम्, 1st gaṇa Bhvādi) in present tense 3rd person singular (laṭ, tiṅ-tip).".to_string(),
            status: "success".to_string(),
            final_surface_form: "bhavati".to_string(),
            root: "√bhū (भू)".to_string(),
            rules: vec![
                SutraRule { sutra_id: "3.2.123".to_string(), text_deva: "वर्तमाने लट्".to_string(), text_slp1: "vartamAne laT".to_string(), classification: "VIDHI".to_string(), summary: "Affixes the lakāra \"laṭ\" to denote action in the present tense.".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "3.4.78".to_string(), text_deva: "तिप्तस्झिसिप्थस्थमिब्वस्मस्तातांझथांसाथांध्वमिड्वहिमहिङ्".to_string(), text_slp1: "tiptasjhisipTasTamibvasmastAtAMJATAMsATAMDvamidvahimahiG".to_string(), classification: "VIDHI".to_string(), summary: "Substitutes lakāra with 18 tiṅ affixes; selects 3rd singular parasmaipada \"tip\".".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "1.3.9".to_string(), text_deva: "तस्य लोपः".to_string(), text_slp1: "tasya lopaH".to_string(), classification: "VIDHI".to_string(), summary: "Elides the it-marker \"p\" while preserving the \"pit\" property tag.".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "3.1.68".to_string(), text_deva: "कर्तरि शप्".to_string(), text_slp1: "kartari Sap".to_string(), classification: "VIDHI".to_string(), summary: "Inserts vikaraṇa affix \"Śap\" after root √bhū before sārvadhātuka affix.".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "3.4.113".to_string(), text_deva: "तिङ्शित्सार्वधातुकम्".to_string(), text_slp1: "tiNSitsArvaDAtukam".to_string(), classification: "SAMJNA".to_string(), summary: "Assigns sārvadhātuka saṃjñā to \"Śap\" (Ś-it) and \"tip\" (tiṅ).".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "7.3.84".to_string(), text_deva: "सार्वधातुकार्धधातुकयोः".to_string(), text_slp1: "sArvaDAtukArDaDAtukayoH".to_string(), classification: "VIDHI".to_string(), summary: "Applies guṇa substitution to the final vowel of the aṅga (ū -> o) before sārvadhātuka \"a\".".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "6.1.78".to_string(), text_deva: "एचोऽयवायावः".to_string(), text_slp1: "eco 'yavAyAvaH".to_string(), classification: "VIDHI".to_string(), summary: "Sandhi replacement: \"o\" followed by vowel \"a\" transforms to \"av\" (bho + a -> bhav + a).".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "1.4.14".to_string(), text_deva: "सुप्तिङन्तं पदम्".to_string(), text_slp1: "suptiGantaM padam".to_string(), classification: "SAMJNA".to_string(), summary: "Designates the complete tiṅ-inflected form \"bhavati\" as a valid syntactic Pada.".to_string(), paribhasha_principle: None, blocked_sutras: None },
            ],
            states: vec![
                DerivationState { id: "state:bhavati:00-input".to_string(), hash: "state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7".to_string(), step_index: 0, schema: "panini-state/0.1".to_string(), terms: vec![DerivationTerm{id:"term:root-bhu".to_string(),kind:"dhAtu".to_string(),source_form:"bhū".to_string(),surface_form:"bhū".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string(),"bhvādi".to_string()]}], applied_rule: None, mutation_type: Some("INITIAL".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:01-lat".to_string(), hash: "state:sha256:a6b610c14b7e8d75cf701be5e921d01ee3fdf9bcf693e506990d0b04c86ec596".to_string(), step_index: 1, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-bhu".to_string(),kind:"dhAtu".to_string(),source_form:"bhū".to_string(),surface_form:"bhū".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:lakara-lat".to_string(),kind:"lakAra".to_string(),source_form:"laṭ".to_string(),surface_form:"laṭ".to_string(),designations:vec!["vartamāna".to_string(),"prathama".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"3.2.123".to_string(),text_deva:"वर्तमाने लट्".to_string(),text_slp1:"vartamAne laT".to_string(),classification:"VIDHI".to_string(),summary:"Affixes present lakāra laṭ.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("AFFIXATION".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:02-tip".to_string(), hash: "state:sha256:7bc59d81d2dfbc1cf8da294c73a216db8a13ef2b8df11559eeecfc316c4c0174".to_string(), step_index: 2, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-bhu".to_string(),kind:"dhAtu".to_string(),source_form:"bhū".to_string(),surface_form:"bhū".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:pratyaya-tip".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"tip".to_string(),designations:vec!["tiṅ".to_string(),"parasmaipada".to_string(),"pit".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"3.4.78".to_string(),text_deva:"तिप्तस्झि...".to_string(),text_slp1:"tiptasjhi...".to_string(),classification:"VIDHI".to_string(),summary:"Substitutes laṭ with tip (3rd singular parasmaipada).".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("AFFIXATION".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:03-it-lopa".to_string(), hash: "state:sha256:b1ec7daea41e06d649da391b1a7d6560daaf47aa6a4387d8cb27efc90b63cbb0".to_string(), step_index: 3, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-bhu".to_string(),kind:"dhAtu".to_string(),source_form:"bhū".to_string(),surface_form:"bhū".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string(),"pit".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"1.3.9".to_string(),text_deva:"तस्य लोपः".to_string(),text_slp1:"tasya lopaH".to_string(),classification:"VIDHI".to_string(),summary:"Elides it-marker p in tip -> ti.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("LOPA".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:04-sap".to_string(), hash: "state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627".to_string(), step_index: 4, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-bhu".to_string(),kind:"dhAtu".to_string(),source_form:"bhū".to_string(),surface_form:"bhū".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:vikarana-sap".to_string(),kind:"pratyaya".to_string(),source_form:"śap".to_string(),surface_form:"śap".to_string(),designations:vec!["vikaraṇa".to_string(),"śit".to_string(),"pit".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string(),"pit".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"3.1.68".to_string(),text_deva:"कर्तरि शप्".to_string(),text_slp1:"kartari Sap".to_string(),classification:"VIDHI".to_string(),summary:"Inserts vikaraṇa affix śap between bhū and ti.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("AFFIXATION".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:05-sap-lopa".to_string(), hash: "state:sha256:56bce457c1d3bf30f04c643b1859c25db94474b34b6b6685718dfd374aa3e843".to_string(), step_index: 5, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-bhu".to_string(),kind:"dhAtu".to_string(),source_form:"bhū".to_string(),surface_form:"bhū".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:vikarana-a".to_string(),kind:"pratyaya".to_string(),source_form:"śap".to_string(),surface_form:"a".to_string(),designations:vec!["vikaraṇa".to_string(),"sārvadhātuka".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string(),"sārvadhātuka".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"3.4.113".to_string(),text_deva:"तिङ्शित्सार्वधातुकम्".to_string(),text_slp1:"tiNSitsArvaDAtukam".to_string(),classification:"SAMJNA".to_string(),summary:"Designates śap and ti as sārvadhātuka; elides ś and p -> a.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("SAMJNA".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:06-guna".to_string(), hash: "state:sha256:0d18d4512e0e0f523173d12d09ff93ff790f977beaece5f7fe51dc1b764bfe47".to_string(), step_index: 6, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:anga-bho".to_string(),kind:"aGga".to_string(),source_form:"bhū".to_string(),surface_form:"bho".to_string(),designations:vec!["aṅga".to_string(),"guṇa-applied".to_string()]},
                    DerivationTerm{id:"term:vikarana-a".to_string(),kind:"pratyaya".to_string(),source_form:"śap".to_string(),surface_form:"a".to_string(),designations:vec!["vikaraṇa".to_string(),"sārvadhātuka".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string(),"sārvadhātuka".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"7.3.84".to_string(),text_deva:"सार्वधातुकार्धधातुकयोः".to_string(),text_slp1:"sArvaDAtukArDaDAtukayoH".to_string(),classification:"VIDHI".to_string(),summary:"Applies guṇa to root vowel: bhū -> bho.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("GUNA".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:07-sandhi".to_string(), hash: "state:sha256:8f4c281df6be48a5df9ee2ca531db5caebbc8b6fe2d721fa6252bf466986427d".to_string(), step_index: 7, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:stem-bhav".to_string(),kind:"aGga".to_string(),source_form:"bho".to_string(),surface_form:"bhav".to_string(),designations:vec!["aṅga".to_string(),"sandhi-applied".to_string()]},
                    DerivationTerm{id:"term:vikarana-a".to_string(),kind:"pratyaya".to_string(),source_form:"śap".to_string(),surface_form:"a".to_string(),designations:vec!["vikaraṇa".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"6.1.78".to_string(),text_deva:"एचोऽयवायावः".to_string(),text_slp1:"eco 'yavAyAvaH".to_string(),classification:"VIDHI".to_string(),summary:"Sandhi: o + a -> av + a (bhav + a).".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("SANDHI".to_string()), proof_verified: true },
                DerivationState { id: "state:bhavati:08-terminal".to_string(), hash: "state:sha256:e647bc51b9e6f3128ef35b2e95a9992d9dcfdcb95e1e1a539bc27fa7a26f8664".to_string(), step_index: 8, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:final-pada".to_string(),kind:"pada".to_string(),source_form:"bhū+tip".to_string(),surface_form:"bhavati".to_string(),designations:vec!["pada".to_string(),"tiṅanta".to_string(),"kartari".to_string(),"lat".to_string(),"prathama".to_string(),"ekavacana".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"1.4.14".to_string(),text_deva:"सुप्तिङन्तं पदम्".to_string(),text_slp1:"suptiGantaM padam".to_string(),classification:"SAMJNA".to_string(),summary:"Declares bhavati as a completed syntactic Pada.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("SAMJNA".to_string()), proof_verified: true },
            ],
            cryptographic_proof: ProofVerification {
                root_hash: "state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7".to_string(),
                terminal_hash: "state:sha256:e647bc51b9e6f3128ef35b2e95a9992d9dcfdcb95e1e1a539bc27fa7a26f8664".to_string(),
                algorithm: "SHA-256".to_string(),
                verified: true,
            },
        }
    }

    fn dadati_trace() -> DerivationTrace {
        DerivationTrace {
            ir_version: "panini-derivation-ir/0.1".to_string(),
            derivation_id: "drv:canonical:dadati-v0.1".to_string(),
            target_word: "ददाति (dadāti)".to_string(),
            description: "Derivation of root √dā (डुदाञ् दाने, 3rd gaṇa Juhotyādi) in present tense 3rd person singular showing Paribhāṣā conflict resolution (Ślu blocking Śap by Apavāda).".to_string(),
            status: "success".to_string(),
            final_surface_form: "dadāti".to_string(),
            root: "√dā (दा)".to_string(),
            rules: vec![
                SutraRule { sutra_id: "3.2.123".to_string(), text_deva: "वर्तमाने लट्".to_string(), text_slp1: "vartamAne laT".to_string(), classification: "VIDHI".to_string(), summary: "Affixes present lakāra laṭ.".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "3.4.78".to_string(), text_deva: "तिप्तस्झि...".to_string(), text_slp1: "tiptasjhi...".to_string(), classification: "VIDHI".to_string(), summary: "Substitutes laṭ with tip (3rd singular).".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "2.4.75".to_string(), text_deva: "जुहोत्यादिभ्यः श्लुः".to_string(), text_slp1: "juhotyAdibhyaH SluH".to_string(), classification: "VIDHI".to_string(), summary: "Replaces vikaraṇa Śap with Ślu (zero replacement) for 3rd class roots.".to_string(), paribhasha_principle: Some("Apavāda > Utsarga (Special exception 2.4.75 completely blocks general 3.1.68 Śap)".to_string()), blocked_sutras: Some(vec!["3.1.68 kartari śap".to_string()]) },
                SutraRule { sutra_id: "6.1.10".to_string(), text_deva: "श्लौ".to_string(), text_slp1: "SlO".to_string(), classification: "VIDHI".to_string(), summary: "Reduplicates the root when Ślu follows (dā -> dā + dā).".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "7.4.59".to_string(), text_deva: "ह्रस्वः".to_string(), text_slp1: "hrasvaH".to_string(), classification: "VIDHI".to_string(), summary: "Shortens the vowel of the reduplicated abhyāsa syllable: dā -> da.".to_string(), paribhasha_principle: None, blocked_sutras: None },
                SutraRule { sutra_id: "1.4.14".to_string(), text_deva: "सुप्तिङन्तं पदम्".to_string(), text_slp1: "suptiGantaM padam".to_string(), classification: "SAMJNA".to_string(), summary: "Declares dadāti as a valid Pada.".to_string(), paribhasha_principle: None, blocked_sutras: None },
            ],
            states: vec![
                DerivationState { id: "state:dadati:00-input".to_string(), hash: "state:sha256:4b9101f3b0e35261d7b322a3683fba227653bbd0a3d46386c67fa128adbc44e1".to_string(), step_index: 0, schema: "panini-state/0.1".to_string(), terms: vec![DerivationTerm{id:"term:root-da".to_string(),kind:"dhAtu".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["dhātu".to_string(),"juhotyādi".to_string()]}], applied_rule: None, mutation_type: Some("INITIAL".to_string()), proof_verified: true },
                DerivationState { id: "state:dadati:01-lat".to_string(), hash: "state:sha256:5c83b8909187a4de88d927a4e6bbcbcfd263914a8b7c4d51b3ca00d7fae01931".to_string(), step_index: 1, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-da".to_string(),kind:"dhAtu".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["dhātu".to_string()]},
                    DerivationTerm{id:"term:lakara-lat".to_string(),kind:"lakAra".to_string(),source_form:"laṭ".to_string(),surface_form:"laṭ".to_string(),designations:vec!["vartamāna".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"3.2.123".to_string(),text_deva:"वर्तमाने लट्".to_string(),text_slp1:"vartamAne laT".to_string(),classification:"VIDHI".to_string(),summary:"Affixes present lakāra laṭ.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("AFFIXATION".to_string()), proof_verified: true },
                DerivationState { id: "state:dadati:02-tip".to_string(), hash: "state:sha256:773ab485be61d67a6d10c144e0573e86cbca388d75cf7940ee95a947bc18a514".to_string(), step_index: 2, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-da".to_string(),kind:"dhAtu".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["dhātu".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string(),"pit".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"3.4.78".to_string(),text_deva:"तिप्तस्झि...".to_string(),text_slp1:"tiptasjhi...".to_string(),classification:"VIDHI".to_string(),summary:"Selects tip affix.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("AFFIXATION".to_string()), proof_verified: true },
                DerivationState { id: "state:dadati:03-slu-block".to_string(), hash: "state:sha256:99f8d17baae7752e5d95d18d4512e0e0f523173d12d09ff93ff790f977beaece".to_string(), step_index: 3, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:root-da".to_string(),kind:"dhAtu".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["dhātu".to_string(),"ślu-environment".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"2.4.75".to_string(),text_deva:"जुहोत्यादिभ्यः श्लुः".to_string(),text_slp1:"juhotyAdibhyaH SluH".to_string(),classification:"VIDHI".to_string(),summary:"Ślu replaces Śap; 3.1.68 blocked by Apavāda priority.".to_string(),paribhasha_principle:Some("Apavāda > Utsarga".to_string()),blocked_sutras:None}), mutation_type: Some("LOPA".to_string()), proof_verified: true },
                DerivationState { id: "state:dadati:04-reduplication".to_string(), hash: "state:sha256:31b1a7d6560daaf47aa6a4387d8cb27efc90b63cbb01a84f339cf01cb4d14210".to_string(), step_index: 4, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:abhyasa-da".to_string(),kind:"abhyAsa".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["abhyāsa".to_string()]},
                    DerivationTerm{id:"term:root-da".to_string(),kind:"dhAtu".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"6.1.10".to_string(),text_deva:"श्लौ".to_string(),text_slp1:"SlO".to_string(),classification:"VIDHI".to_string(),summary:"Reduplicates root: dā -> dā + dā.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("AFFIXATION".to_string()), proof_verified: true },
                DerivationState { id: "state:dadati:05-abhyasa-shorten".to_string(), hash: "state:sha256:56bce457c1d3bf30f04c643b1859c25db94474b34b6b6685718dfd374aa3e843".to_string(), step_index: 5, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:abhyasa-da-short".to_string(),kind:"abhyAsa".to_string(),source_form:"dā".to_string(),surface_form:"da".to_string(),designations:vec!["abhyāsa".to_string(),"hrasva".to_string()]},
                    DerivationTerm{id:"term:root-da".to_string(),kind:"dhAtu".to_string(),source_form:"dā".to_string(),surface_form:"dā".to_string(),designations:vec!["dhātu".to_string(),"aṅga".to_string()]},
                    DerivationTerm{id:"term:pratyaya-ti".to_string(),kind:"pratyaya".to_string(),source_form:"tip".to_string(),surface_form:"ti".to_string(),designations:vec!["tiṅ".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"7.4.59".to_string(),text_deva:"ह्रस्वः".to_string(),text_slp1:"hrasvaH".to_string(),classification:"VIDHI".to_string(),summary:"Shortens abhyāsa vowel: dā -> da.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("SAMJNA".to_string()), proof_verified: true },
                DerivationState { id: "state:dadati:06-terminal".to_string(), hash: "state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627".to_string(), step_index: 6, schema: "panini-state/0.1".to_string(), terms: vec![
                    DerivationTerm{id:"term:final-pada-dadati".to_string(),kind:"pada".to_string(),source_form:"dā+tip".to_string(),surface_form:"dadāti".to_string(),designations:vec!["pada".to_string(),"tiṅanta".to_string(),"juhotyādi".to_string(),"prathama".to_string(),"ekavacana".to_string()]},
                ], applied_rule: Some(SutraRule{sutra_id:"1.4.14".to_string(),text_deva:"सुप्तिङन्तं पदम्".to_string(),text_slp1:"suptiGantaM padam".to_string(),classification:"SAMJNA".to_string(),summary:"Declares dadāti as a valid syntactic Pada.".to_string(),paribhasha_principle:None,blocked_sutras:None}), mutation_type: Some("SAMJNA".to_string()), proof_verified: true },
            ],
            cryptographic_proof: ProofVerification {
                root_hash: "state:sha256:4b9101f3b0e35261d7b322a3683fba227653bbd0a3d46386c67fa128adbc44e1".to_string(),
                terminal_hash: "state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627".to_string(),
                algorithm: "SHA-256".to_string(),
                verified: true,
            },
        }
    }

    fn phoneme_a() -> PhonemeVectorData {
        PhonemeVectorData {
            phoneme: "a".to_string(),
            slp1: "a".to_string(),
            deva: "अ".to_string(),
            upc8: 0x00,
            upc8_hex: "0x00".to_string(),
            pvc16_raw: 0x0003,
            pvc16_hex: "0x0003".to_string(),
            is_vowel: true,
            sthana_name: "Kaṇṭhya (Velar/Guttural)".to_string(),
            prayatna_name: "Vivṛta / Saṃvṛta".to_string(),
            is_palatalized: false,
            pratyahara_bit_index: 0,
            pratyahara_mask_u64: "0x0000000000000001".to_string(),
            pratyaharas_contained: vec!["ac", "al", "ak", "aṇ", "am", "aś"]
                .into_iter()
                .map(String::from)
                .collect(),
            is_ukrainian: false,
            ipa: "[ɐ] / [ə]".to_string(),
        }
    }

    fn phoneme_i() -> PhonemeVectorData {
        PhonemeVectorData {
            phoneme: "i".to_string(),
            slp1: "i".to_string(),
            deva: "इ".to_string(),
            upc8: 0x01,
            upc8_hex: "0x01".to_string(),
            pvc16_raw: 0x0005,
            pvc16_hex: "0x0005".to_string(),
            is_vowel: true,
            sthana_name: "Tālavya (Palatal)".to_string(),
            prayatna_name: "Spṛṣṭa-vivṛta".to_string(),
            is_palatalized: false,
            pratyahara_bit_index: 1,
            pratyahara_mask_u64: "0x0000000000000002".to_string(),
            pratyaharas_contained: vec!["ac", "al", "ik", "iṇ", "ic", "ak", "aṇ"]
                .into_iter()
                .map(String::from)
                .collect(),
            is_ukrainian: false,
            ipa: "[i]".to_string(),
        }
    }

    fn phoneme_u() -> PhonemeVectorData {
        PhonemeVectorData {
            phoneme: "u".to_string(),
            slp1: "u".to_string(),
            deva: "उ".to_string(),
            upc8: 0x02,
            upc8_hex: "0x02".to_string(),
            pvc16_raw: 0x0021,
            pvc16_hex: "0x0021".to_string(),
            is_vowel: true,
            sthana_name: "Oṣṭhya (Labial)".to_string(),
            prayatna_name: "Vivṛta".to_string(),
            is_palatalized: false,
            pratyahara_bit_index: 2,
            pratyahara_mask_u64: "0x0000000000000004".to_string(),
            pratyaharas_contained: vec!["ac", "al", "uk", "ik", "ak", "aṇ"]
                .into_iter()
                .map(String::from)
                .collect(),
            is_ukrainian: false,
            ipa: "[u]".to_string(),
        }
    }

    fn phoneme_k() -> PhonemeVectorData {
        PhonemeVectorData {
            phoneme: "k".to_string(),
            slp1: "k".to_string(),
            deva: "क्".to_string(),
            upc8: 0x19,
            upc8_hex: "0x19".to_string(),
            pvc16_raw: 0x0042,
            pvc16_hex: "0x0042".to_string(),
            is_vowel: false,
            sthana_name: "Kaṇṭhya (Velar)".to_string(),
            prayatna_name: "Spṛṣṭa, Aghoṣa, Alpaprāṇa".to_string(),
            is_palatalized: false,
            pratyahara_bit_index: 25,
            pratyahara_mask_u64: "0x0000000002000000".to_string(),
            pratyaharas_contained: vec!["hal", "al", "jhay", "khay", "cay", "yar", "khar"]
                .into_iter()
                .map(String::from)
                .collect(),
            is_ukrainian: false,
            ipa: "[k]".to_string(),
        }
    }

    fn phoneme_t_palatal() -> PhonemeVectorData {
        PhonemeVectorData {
            phoneme: "t'".to_string(),
            slp1: "t'".to_string(),
            deva: "त् (т')".to_string(),
            upc8: 0x3d,
            upc8_hex: "0x3D".to_string(),
            pvc16_raw: 0x4048,
            pvc16_hex: "0x4048".to_string(),
            is_vowel: false,
            sthana_name: "Dantya + Palatalized".to_string(),
            prayatna_name: "Spṛṣṭa, Aghoṣa, M'yakiy".to_string(),
            is_palatalized: true,
            pratyahara_bit_index: 43,
            pratyahara_mask_u64: "0x0000080000000000".to_string(),
            pratyaharas_contained: vec!["ukrainian_palatalized", "hal_extended"]
                .into_iter()
                .map(String::from)
                .collect(),
            is_ukrainian: true,
            ipa: "[tʲ]".to_string(),
        }
    }
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

/// Helper function to register commands with Tauri v2 Builder.
///
/// Concrete `tauri::Wry` runtime, not generic over `R: tauri::Runtime`:
/// this app is a single-runtime desktop shell, never instantiated with
/// any other `Runtime` impl, so genericity here bought nothing. It also
/// broke: `stream_phoneme_vector` takes a bare `AppHandle` (= `AppHandle<Wry>`
/// by that type's own default), which `tauri::generate_handler!` cannot
/// match against an actually-generic `R` — rustc reported this as
/// `AppHandle: CommandArg<'_, R>` not satisfied / "Deserialize not
/// implemented for AppHandle", which is the macro falling back to
/// treating `AppHandle` as an ordinary deserializable argument once it
/// no longer recognizes it as the special injected-runtime type for a
/// still-open `R`. Confirmed via a real Windows CI compile failure
/// (gh run view --repo juv4uk/tauricode --log-failed), not predicted.
pub fn register_swarm_commands() -> impl Fn(tauri::ipc::Invoke<tauri::Wry>) -> bool {
    tauri::generate_handler![
        get_swarm_topology,
        query_derivation_trace,
        stream_phoneme_vector
    ]
}
