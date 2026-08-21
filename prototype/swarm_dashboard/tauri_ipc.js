/**
 * Tauricode Tauri v2 IPC Client Bridge (JavaScript ESM version)
 */

import {
  MOCK_SWARM_TOPOLOGY,
  CANONICAL_DERIVATIONS,
  PHONEME_DATA_REGISTRY
} from './fixtures.js';

export async function getSwarmTopology() {
  const clone = JSON.parse(JSON.stringify(MOCK_SWARM_TOPOLOGY));
  clone.timestamp = new Date().toISOString();
  return clone;
}

export async function queryDerivationTrace(derivationId) {
  const key = derivationId.includes('dadati') ? 'dadati' : 'bhavati';
  const trace = CANONICAL_DERIVATIONS[key];
  if (!trace) {
    throw new Error(`Derivation trace not found for: ${derivationId}`);
  }
  return JSON.parse(JSON.stringify(trace));
}

export async function streamPhonemeVector(phonemeOrCode) {
  const key = String(phonemeOrCode);
  const matched =
    PHONEME_DATA_REGISTRY[key] ||
    Object.values(PHONEME_DATA_REGISTRY).find((p) => p.slp1 === key || p.phoneme === key || p.upc8Hex?.toLowerCase() === key.toLowerCase()) ||
    PHONEME_DATA_REGISTRY['a'];
  return JSON.parse(JSON.stringify(matched));
}
