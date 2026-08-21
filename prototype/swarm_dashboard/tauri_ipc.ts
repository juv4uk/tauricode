/**
 * Tauricode Desktop Workbench - Tauri v2 IPC Client Bridge
 * Supports both native Tauri v2 IPC (`@tauri-apps/api/core`) and
 * transparent browser fallback simulation for standalone developer preview.
 */

import {
  SwarmMeshTopology,
  DerivationTrace,
  PhonemeVectorData,
  SwarmNode,
  DerivationState
} from './types';
import {
  MOCK_SWARM_TOPOLOGY,
  CANONICAL_DERIVATIONS,
  PHONEME_DATA_REGISTRY
} from './fixtures';

// Check if running inside Tauri Webview
const isTauriEnv = (): boolean => {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
};

/**
 * Invoke Tauri v2 command `get_swarm_topology`
 */
export async function getSwarmTopology(): Promise<SwarmMeshTopology> {
  if (isTauriEnv()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<SwarmMeshTopology>('get_swarm_topology');
    } catch (err) {
      console.warn('[Tauri IPC] Failed to invoke get_swarm_topology, falling back to mock:', err);
    }
  }

  // Simulated live telemetry fluctuation
  const clone = JSON.parse(JSON.stringify(MOCK_SWARM_TOPOLOGY)) as SwarmMeshTopology;
  clone.timestamp = new Date().toISOString();
  clone.nodes.forEach((node) => {
    node.latencyMs = Number((node.latencyMs + (Math.random() * 0.4 - 0.2)).toFixed(2));
    node.cpuUsagePct = Number(Math.min(99, Math.max(1, node.cpuUsagePct + (Math.random() * 4 - 2))).toFixed(1));
    node.lastHeartbeat = new Date().toISOString();
  });
  return clone;
}

/**
 * Invoke Tauri v2 command `query_derivation_trace`
 */
export async function queryDerivationTrace(derivationId: string): Promise<DerivationTrace> {
  if (isTauriEnv()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<DerivationTrace>('query_derivation_trace', { derivationId });
    } catch (err) {
      console.warn('[Tauri IPC] Failed to invoke query_derivation_trace, falling back to fixtures:', err);
    }
  }

  const key = derivationId.includes('dadati') ? 'dadati' : 'bhavati';
  const trace = CANONICAL_DERIVATIONS[key];
  if (!trace) {
    throw new Error(`Derivation trace not found for: ${derivationId}`);
  }
  return JSON.parse(JSON.stringify(trace));
}

/**
 * Invoke Tauri v2 command `stream_phoneme_vector`
 */
export async function streamPhonemeVector(phonemeOrCode: string | number): Promise<PhonemeVectorData> {
  if (isTauriEnv()) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      return await invoke<PhonemeVectorData>('stream_phoneme_vector', { phonemeOrCode: String(phonemeOrCode) });
    } catch (err) {
      console.warn('[Tauri IPC] Failed to invoke stream_phoneme_vector, falling back to registry:', err);
    }
  }

  const key = String(phonemeOrCode).toLowerCase();
  const matched =
    PHONEME_DATA_REGISTRY[key] ||
    Object.values(PHONEME_DATA_REGISTRY).find((p) => p.slp1 === key || p.phoneme === key || p.upc8Hex.toLowerCase() === key) ||
    PHONEME_DATA_REGISTRY['a'];

  return JSON.parse(JSON.stringify(matched));
}

/**
 * Subscribe to real-time Node Heartbeat updates
 */
export function listenNodeHeartbeat(callback: (node: SwarmNode) => void): () => void {
  if (isTauriEnv()) {
    let unlistenPromise: Promise<() => void> | null = null;
    import('@tauri-apps/api/event').then(({ listen }) => {
      unlistenPromise = listen<SwarmNode>('telemetry://node-heartbeat', (event) => {
        callback(event.payload);
      });
    });
    return () => {
      unlistenPromise?.then((unlisten) => unlisten());
    };
  }

  // Simulated browser timer
  const interval = setInterval(() => {
    const randomNode = MOCK_SWARM_TOPOLOGY.nodes[Math.floor(Math.random() * MOCK_SWARM_TOPOLOGY.nodes.length)];
    const nodeUpdate: SwarmNode = {
      ...randomNode,
      latencyMs: Number((randomNode.latencyMs + (Math.random() * 0.3 - 0.15)).toFixed(2)),
      cpuUsagePct: Number(Math.min(99, Math.max(1, randomNode.cpuUsagePct + (Math.random() * 6 - 3))).toFixed(1)),
      lastHeartbeat: new Date().toISOString()
    };
    callback(nodeUpdate);
  }, 3000);

  return () => clearInterval(interval);
}
