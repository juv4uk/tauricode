/**
 * Tauricode Desktop Workbench & Swarm Telemetry Type Definitions
 * Specification: tauricode-swarm-dashboard-v0.1
 * Author: TauriCode Desktop Workbench & Telemetry Agent
 */

export type NodeStatus = 'ONLINE' | 'DEGRADED' | 'OFFLINE' | 'SYNCING';

export interface SwarmNode {
  id: string;
  name: string;
  port: number;
  role: string;
  repo: string;
  layer: string;
  status: NodeStatus;
  latencyMs: number;
  completedTasks: number;
  totalTasks: number;
  capabilities: string[];
  endpoint: string;
  cpuUsagePct: number;
  memoryMb: number;
  uptimeSeconds: number;
  lastHeartbeat: string;
  version: string;
  activePeers: string[];
}

export interface SwarmMeshTopology {
  clusterId: string;
  clusterName: string;
  activeNodesCount: number;
  totalTasksCompleted: number;
  meshHealthPct: number;
  timestamp: string;
  nodes: SwarmNode[];
  connections: Array<{
    source: string;
    target: string;
    protocol: 'TCP' | 'IPC' | 'REST' | 'WEBSOCKET';
    bandwidthKbps: number;
    latencyMs: number;
    active: boolean;
  }>;
}

export interface TaskRecord {
  id: string;
  nodeId: string;
  title: string;
  description: string;
  status: 'COMPLETED' | 'IN_PROGRESS' | 'QUEUED';
  priority: number;
  capabilities: string[];
  context: string;
  completedAt: string;
}

export interface TaskStats {
  totalCompleted: number;
  totalPending: number;
  totalQueued: number;
  completionRatePct: number;
  capabilityDistribution: Record<string, number>;
  nodeDistribution: Record<string, number>;
  recentTasks: TaskRecord[];
}

export interface SutraRule {
  sutra_id: string;
  text_deva: string;
  text_slp1: string;
  classification: 'VIDHI' | 'SAMJNA' | 'PARIBHASHA' | 'ADHIKARA' | 'NISIIDHA';
  summary: string;
  paribhasha_principle?: string;
  blocked_sutras?: string[];
}

export interface DerivationTerm {
  id: string;
  kind: 'dhAtu' | 'pratyaya' | 'lakAra' | 'abhyAsa' | 'aGga' | 'pada';
  source_form: string;
  surface_form: string;
  designations: string[];
  it_tags?: string[];
  anubandhas?: string[];
}

export interface DerivationState {
  id: string;
  hash: string;
  step_index: number;
  schema: string;
  terms: DerivationTerm[];
  applied_rule?: SutraRule;
  mutation_type?: 'INITIAL' | 'AFFIXATION' | 'LOPA' | 'GUNA' | 'VRIDDHI' | 'SANDHI' | 'SAMJNA';
  diff?: {
    added: string[];
    removed: string[];
    transformed: Array<{ from: string; to: string }>;
  };
  proof_verified: boolean;
}

export interface DerivationTrace {
  ir_version: string;
  derivation_id: string;
  target_word: string;
  description: string;
  status: 'success' | 'failed' | 'in_progress';
  final_surface_form: string;
  root: string;
  rules: SutraRule[];
  states: DerivationState[];
  cryptographic_proof: {
    root_hash: string;
    terminal_hash: string;
    algorithm: 'SHA-256';
    verified: boolean;
  };
}

export interface Pvc16Features {
  raw: number;
  hex: string;
  binary: string;
  isVowel: boolean;
  sthana: {
    kanthya: boolean;
    talavya: boolean;
    murdhanya: boolean;
    dantya: boolean;
    osthya: boolean;
    name: string;
  };
  prayatna: {
    sprsta: boolean;
    mahaprana: boolean;
    ghosa: boolean;
    anunasika: boolean;
    name: string;
  };
  svara: {
    length: 'hrasva' | 'dirgha' | 'pluta';
    accent: 'udatta' | 'anudatta' | 'svarita';
  };
  modifier: {
    isPalatalized: boolean; // Ukrainian [ь]
    isExtension: boolean;
  };
}

export interface PhonemeVectorData {
  phoneme: string;
  slp1: string;
  deva: string;
  upc8: number;
  upc8Hex: string;
  pvc16: Pvc16Features;
  pratyaharaBitIndex: number;
  pratyaharaMaskU64: string;
  pratyaharasContained: string[];
  isUkrainian: boolean;
  ipa: string;
}

/**
 * Tauri v2 IPC Command & Event Interface
 */
export interface TauriSwarmIpcApi {
  get_swarm_topology: () => Promise<SwarmMeshTopology>;
  query_derivation_trace: (derivationId: string) => Promise<DerivationTrace>;
  stream_phoneme_vector: (phonemeOrCode: string | number) => Promise<PhonemeVectorData>;
  listen_node_heartbeat: (callback: (node: SwarmNode) => void) => () => void;
  listen_derivation_event: (callback: (state: DerivationState) => void) => () => void;
}
