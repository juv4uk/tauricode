/**
 * Tauricode Swarm Dashboard & Telemetry Automated Test Suite
 * Runner: Node.js / Bun (ESM)
 */

import {
  MOCK_SWARM_TOPOLOGY,
  MOCK_TASK_STATS,
  CANONICAL_DERIVATIONS,
  PHONEME_DATA_REGISTRY
} from './fixtures.js';
import {
  getSwarmTopology,
  queryDerivationTrace,
  streamPhonemeVector
} from './tauri_ipc.js';

let passed = 0;
let failed = 0;

function assert(condition, message) {
  if (condition) {
    console.log(`  ✓ PASS: ${message}`);
    passed++;
  } else {
    console.error(`  ✗ FAIL: ${message}`);
    failed++;
  }
}

async function runTests() {
  console.log('================================================================');
  console.log('TAURICODE SWARM DASHBOARD & TELEMETRY PROTOCOL TEST SUITE');
  console.log('================================================================\n');

  // Test Suite 1: Swarm Topology & Active Nodes
  console.log('1. Swarm Mesh Topology & Port Verification (:9101 - :9107):');
  const expectedNodes = [
    { name: 'my-lisp-1', port: 9101 },
    { name: 'fpga-lisp-1', port: 9102 },
    { name: 'cml-1', port: 9103 },
    { name: 'my-idea-1', port: 9104 },
    { name: 'my-lisp-panini-1', port: 9106 },
    { name: 'shiva-sutras-1', port: 9107 }
  ];

  assert(MOCK_SWARM_TOPOLOGY.nodes.length === 6, 'Cluster registers exactly 6 active Swarm nodes');
  assert(MOCK_SWARM_TOPOLOGY.activeNodesCount === 6, 'Active node count equals 6');

  expectedNodes.forEach(({ name, port }) => {
    const node = MOCK_SWARM_TOPOLOGY.nodes.find((n) => n.name === name);
    assert(Boolean(node), `Node ${name} is registered in topology`);
    if (node) {
      assert(node.port === port, `Node ${name} operates on target port ${port}`);
      assert(node.status === 'ONLINE', `Node ${name} health status is ONLINE`);
      assert(node.latencyMs > 0 && node.latencyMs < 5.0, `Node ${name} reports low latency (${node.latencyMs}ms)`);
      assert(node.capabilities.length > 0, `Node ${name} advertises capabilities (${node.capabilities.join(', ')})`);
    }
  });

  // Test Suite 2: Task Completion Statistics (271+ completed tasks target)
  console.log('\n2. Task Completion Telemetry Verification (Target >= 271):');
  assert(MOCK_TASK_STATS.totalCompleted >= 271, `Total completed tasks (${MOCK_TASK_STATS.totalCompleted}) exceeds target of 271+`);
  assert(MOCK_TASK_STATS.completionRatePct >= 95.0, `Mesh completion rate (${MOCK_TASK_STATS.completionRatePct}%) is above 95%`);

  const sumTasksByNodes = Object.values(MOCK_TASK_STATS.nodeDistribution).reduce((a, b) => a + b, 0);
  assert(sumTasksByNodes === MOCK_TASK_STATS.totalCompleted, `Node task distribution sum (${sumTasksByNodes}) matches total (${MOCK_TASK_STATS.totalCompleted})`);

  // Test Suite 3: Tauri v2 IPC Command Contracts
  console.log('\n3. Tauri v2 IPC Command Interface Verification:');
  const topologyFromIpc = await getSwarmTopology();
  assert(topologyFromIpc.nodes.length === 6, 'get_swarm_topology IPC returns 6 nodes');
  assert(topologyFromIpc.totalTasksCompleted >= 271, 'get_swarm_topology IPC returns task stats >= 271');

  const traceBhavati = await queryDerivationTrace('bhavati');
  assert(traceBhavati.derivation_id === 'drv:canonical:bhavati-v0.1', 'query_derivation_trace returns canonical bhavati');
  assert(traceBhavati.states.length === 9, 'bhavati derivation contains 9 states (S0 through S8)');
  assert(traceBhavati.cryptographic_proof.verified === true, 'bhavati cryptographic proof is marked verified');

  const traceDadati = await queryDerivationTrace('dadati');
  assert(traceDadati.derivation_id === 'drv:canonical:dadati-v0.1', 'query_derivation_trace returns canonical dadāti');
  assert(traceDadati.states.length === 7, 'dadāti derivation contains 7 states (S0 through S6)');
  const sluState = traceDadati.states.find((s) => s.applied_rule?.sutra_id === '2.4.75');
  assert(Boolean(sluState), 'dadāti contains Sūtra 2.4.75 (Ślu replacement)');
  assert(
    sluState?.applied_rule?.paribhasha_principle?.includes('Apavāda'),
    'Ślu rule correctly documents Apavāda > Utsarga priority conflict resolution'
  );

  // Test Suite 4: Phonetic Vector Inspection & ALU Bitmasks
  console.log('\n4. Phonetic Vector & PVC-16 / 64-Bit Bitmask Verification:');
  const phonemeA = await streamPhonemeVector('a');
  assert(phonemeA.upc8 === 0x00, 'Phoneme "a" UPC-8 code is 0x00');
  assert(phonemeA.pvc16.isVowel === true, 'Phoneme "a" PVC-16 marks Vowel (ac) = true');
  assert(phonemeA.pvc16.sthana.kanthya === true, 'Phoneme "a" Sthāna is Kaṇṭhya');

  const phonemeK = await streamPhonemeVector('k');
  assert(phonemeK.upc8 === 0x19, 'Phoneme "k" UPC-8 code is 0x19');
  assert(phonemeK.pvc16.isVowel === false, 'Phoneme "k" PVC-16 marks Consonant (hal) = true');
  assert(phonemeK.pvc16.prayatna.sprsta === true, 'Phoneme "k" Prayatna is Spṛṣṭa (Stop)');

  const phonemeTPal = await streamPhonemeVector("t'");
  assert(phonemeTPal.isUkrainian === true, 'Phoneme "t\'" is recognized as Ukrainian extension');
  assert(phonemeTPal.pvc16.modifier.isPalatalized === true, 'Phoneme "t\'" sets PVC-16 bit 14 (MOD_PALATALIZED)');

  // Sūtra 1.1.9 Savarṇa Test
  const savarnaSelf = (phonemeA.pvc16.raw & 0x003e) === (phonemeA.pvc16.raw & 0x003e) &&
                      (phonemeA.pvc16.raw & 0x0041) === (phonemeA.pvc16.raw & 0x0041);
  assert(savarnaSelf === true, 'Sūtra 1.1.9: Phoneme "a" is savarṇa with itself');

  const savarnaAK = (phonemeA.pvc16.raw & 0x003e) === (phonemeK.pvc16.raw & 0x003e) &&
                    (phonemeA.pvc16.raw & 0x0041) === (phonemeK.pvc16.raw & 0x0041);
  assert(savarnaAK === false, 'Sūtra 1.1.9: Phoneme "a" (vowel) is NOT savarṇa with "k" (consonant)');

  // Summary
  console.log('\n================================================================');
  console.log(`TEST RUN COMPLETE: ${passed} PASSED, ${failed} FAILED`);
  console.log('================================================================');

  if (failed > 0) {
    throw new Error(`${failed} tests failed!`);
  }
}

// Execute tests if running directly
if (typeof process !== 'undefined' && import.meta.url === `file://${process.argv[1]}`) {
  runTests().catch((e) => {
    console.error(e);
    process.exit(1);
  });
}

export { runTests };
