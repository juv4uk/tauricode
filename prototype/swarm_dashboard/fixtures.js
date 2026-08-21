/**
 * Tauricode Swarm Telemetry Fixtures (JavaScript ESM version)
 */

export const MOCK_SWARM_TOPOLOGY = {
  clusterId: 'swarm:mylisp-mesh-p5-alpha',
  clusterName: 'My-Lisp Autonomous Ecosystem Mesh',
  activeNodesCount: 6,
  totalTasksCompleted: 298,
  meshHealthPct: 100,
  timestamp: '2026-08-21T09:30:00+03:00',
  nodes: [
    {
      id: 'node:my-lisp-1',
      name: 'my-lisp-1',
      port: 9101,
      role: 'Core Lisp VM / Semantic Oracle / Knowledge Store',
      repo: 'my-lisp',
      layer: 'Layer 6 (VM & Runtime) / Layer 5 (Proof Engine)',
      status: 'ONLINE',
      latencyMs: 1.2,
      completedTasks: 82,
      totalTasks: 82,
      capabilities: ['eval', 'lisp', 'vm', 'ast', 'proof', 'oracle', 'alist-parser'],
      endpoint: 'tcp://127.0.0.1:9101',
      cpuUsagePct: 14.5,
      memoryMb: 128.4,
      uptimeSeconds: 864200,
      lastHeartbeat: '2026-08-21T09:29:58+03:00',
      version: 'v0.9.4-p5',
      activePeers: ['fpga-lisp-1', 'cml-1', 'my-idea-1', 'my-lisp-panini-1', 'shiva-sutras-1']
    },
    {
      id: 'node:fpga-lisp-1',
      name: 'fpga-lisp-1',
      port: 9102,
      role: 'Hardware Synthesizer / ALU & Accelerator Co-processor',
      repo: 'fpga-lisp',
      layer: 'Layer 6 (Hardware Synthesis & RTL)',
      status: 'ONLINE',
      latencyMs: 0.8,
      completedTasks: 38,
      totalTasks: 38,
      capabilities: ['verilog', 'fpga', 'alu', 'synthesis', 'hardware', 'pvc16-alu', 'pratyahara-rom'],
      endpoint: 'tcp://127.0.0.1:9102',
      cpuUsagePct: 8.2,
      memoryMb: 64.1,
      uptimeSeconds: 864200,
      lastHeartbeat: '2026-08-21T09:29:59+03:00',
      version: 'v0.4.1-tang25k',
      activePeers: ['my-lisp-1', 'cml-1', 'shiva-sutras-1']
    },
    {
      id: 'node:cml-1',
      name: 'cml-1',
      port: 9103,
      role: 'CML Compiler Architecture & Lowering Middle-End',
      repo: 'cml',
      layer: 'Layer 6 (Compiler & Hardware Co-Design)',
      status: 'ONLINE',
      latencyMs: 1.5,
      completedTasks: 44,
      totalTasks: 44,
      capabilities: ['compiler', 'rust', 'lowering', 'proof', 'phonetic', 'constant-folding', 'c99-emitter'],
      endpoint: 'tcp://127.0.0.1:9103',
      cpuUsagePct: 21.0,
      memoryMb: 195.8,
      uptimeSeconds: 864200,
      lastHeartbeat: '2026-08-21T09:29:57+03:00',
      version: 'v0.8.2-lowering',
      activePeers: ['my-lisp-1', 'fpga-lisp-1', 'my-lisp-panini-1', 'shiva-sutras-1']
    },
    {
      id: 'node:my-idea-1',
      name: 'my-idea-1',
      port: 9104,
      role: 'IDE & Visual Tooling Agent / System Observatory',
      repo: 'my-idea',
      layer: 'Layer 6 (Workbench & Developer Tooling)',
      status: 'ONLINE',
      latencyMs: 1.8,
      completedTasks: 41,
      totalTasks: 41,
      capabilities: ['ide', 'visualizer', 'clojurescript', 'tauri', 'codemirror', 'dag-inspector'],
      endpoint: 'tcp://127.0.0.1:9104',
      cpuUsagePct: 18.7,
      memoryMb: 245.2,
      uptimeSeconds: 864200,
      lastHeartbeat: '2026-08-21T09:29:58+03:00',
      version: 'v0.7.0-observatory',
      activePeers: ['my-lisp-1', 'my-lisp-panini-1', 'shiva-sutras-1']
    },
    {
      id: 'node:my-lisp-panini-1',
      name: 'my-lisp-panini-1',
      port: 9106,
      role: 'Panini Grammar Machine / Derivation IR Engine',
      repo: 'my-lisp-panini',
      layer: 'Layer 2 (Grammar Mechanics) / Layer 5 (Derivation Proofs)',
      status: 'ONLINE',
      latencyMs: 2.1,
      completedTasks: 46,
      totalTasks: 46,
      capabilities: ['grammar', 'panini', 'derivation', 'proof-graph', 'ir', 'paribhasha', 'ganapatha'],
      endpoint: 'tcp://127.0.0.1:9106',
      cpuUsagePct: 28.4,
      memoryMb: 312.0,
      uptimeSeconds: 864200,
      lastHeartbeat: '2026-08-21T09:29:59+03:00',
      version: 'v0.5.2-ir-v0.1',
      activePeers: ['my-lisp-1', 'cml-1', 'my-idea-1', 'shiva-sutras-1']
    },
    {
      id: 'node:shiva-sutras-1',
      name: 'shiva-sutras-1',
      port: 9107,
      role: 'Phonetic Engine / Śiva Sūtras & UPC-8 / PVC-16 Canon',
      repo: 'shiva-sutras',
      layer: 'Layer 1 (Canonical Text) / Layer 6 (Phonetic Codecs)',
      status: 'ONLINE',
      latencyMs: 0.9,
      completedTasks: 47,
      totalTasks: 47,
      capabilities: ['phonetics', 'shiva', 'upc8', 'canon', 'pratyahara', 'slavic-phonetics', 'tokenizers'],
      endpoint: 'tcp://127.0.0.1:9107',
      cpuUsagePct: 11.2,
      memoryMb: 88.5,
      uptimeSeconds: 864200,
      lastHeartbeat: '2026-08-21T09:29:58+03:00',
      version: 'v1.0.0-adr002',
      activePeers: ['my-lisp-1', 'fpga-lisp-1', 'cml-1', 'my-idea-1', 'my-lisp-panini-1']
    }
  ],
  connections: [
    { source: 'my-lisp-1', target: 'fpga-lisp-1', protocol: 'TCP', bandwidthKbps: 10000, latencyMs: 0.8, active: true },
    { source: 'my-lisp-1', target: 'cml-1', protocol: 'TCP', bandwidthKbps: 10000, latencyMs: 1.5, active: true },
    { source: 'my-lisp-1', target: 'my-idea-1', protocol: 'TCP', bandwidthKbps: 10000, latencyMs: 1.8, active: true },
    { source: 'my-lisp-1', target: 'my-lisp-panini-1', protocol: 'TCP', bandwidthKbps: 10000, latencyMs: 2.1, active: true },
    { source: 'my-lisp-1', target: 'shiva-sutras-1', protocol: 'TCP', bandwidthKbps: 10000, latencyMs: 0.9, active: true },
    { source: 'cml-1', target: 'fpga-lisp-1', protocol: 'IPC', bandwidthKbps: 25000, latencyMs: 0.4, active: true },
    { source: 'cml-1', target: 'shiva-sutras-1', protocol: 'IPC', bandwidthKbps: 25000, latencyMs: 0.5, active: true },
    { source: 'my-lisp-panini-1', target: 'shiva-sutras-1', protocol: 'TCP', bandwidthKbps: 10000, latencyMs: 1.1, active: true },
    { source: 'my-idea-1', target: 'my-lisp-panini-1', protocol: 'WEBSOCKET', bandwidthKbps: 15000, latencyMs: 1.6, active: true }
  ]
};

export const MOCK_TASK_STATS = {
  totalCompleted: 298,
  totalPending: 0,
  totalQueued: 3,
  completionRatePct: 99.0,
  capabilityDistribution: {
    'lisp/eval': 82,
    'phonetics/shiva': 47,
    'grammar/panini': 46,
    'compiler/lowering': 44,
    'ide/visualizer': 41,
    'fpga/hardware': 38
  },
  nodeDistribution: {
    'my-lisp-1': 82,
    'shiva-sutras-1': 47,
    'my-lisp-panini-1': 46,
    'cml-1': 44,
    'my-idea-1': 41,
    'fpga-lisp-1': 38
  },
  recentTasks: []
};

export const CANONICAL_DERIVATIONS = {
  bhavati: {
    ir_version: 'panini-derivation-ir/0.1',
    derivation_id: 'drv:canonical:bhavati-v0.1',
    target_word: 'भवति (bhavati)',
    description: 'Derivation of root √bhū in present tense 3rd singular.',
    status: 'success',
    final_surface_form: 'bhavati',
    root: '√bhū (भू)',
    states: [
      { id: 'S0', step_index: 0, terms: [{ surface_form: 'bhū', kind: 'dhAtu' }], hash: 'state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7' },
      { id: 'S1', step_index: 1, terms: [{ surface_form: 'bhū', kind: 'dhAtu' }, { surface_form: 'laṭ', kind: 'lakAra' }], hash: 'state:sha256:a6b610c14b7e8d75cf701be5e921d01ee3fdf9bcf693e506990d0b04c86ec596' },
      { id: 'S2', step_index: 2, terms: [{ surface_form: 'bhū', kind: 'dhAtu' }, { surface_form: 'tip', kind: 'pratyaya' }], hash: 'state:sha256:7bc59d81d2dfbc1cf8da294c73a216db8a13ef2b8df11559eeecfc316c4c0174' },
      { id: 'S3', step_index: 3, terms: [{ surface_form: 'bhū', kind: 'dhAtu' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:b1ec7daea41e06d649da391b1a7d6560daaf47aa6a4387d8cb27efc90b63cbb0' },
      { id: 'S4', step_index: 4, terms: [{ surface_form: 'bhū', kind: 'dhAtu' }, { surface_form: 'śap', kind: 'pratyaya' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627' },
      { id: 'S5', step_index: 5, terms: [{ surface_form: 'bhū', kind: 'dhAtu' }, { surface_form: 'a', kind: 'pratyaya' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:56bce457c1d3bf30f04c643b1859c25db94474b34b6b6685718dfd374aa3e843' },
      { id: 'S6', step_index: 6, terms: [{ surface_form: 'bho', kind: 'aGga' }, { surface_form: 'a', kind: 'pratyaya' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:0d18d4512e0e0f523173d12d09ff93ff790f977beaece5f7fe51dc1b764bfe47' },
      { id: 'S7', step_index: 7, terms: [{ surface_form: 'bhav', kind: 'aGga' }, { surface_form: 'a', kind: 'pratyaya' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:8f4c281df6be48a5df9ee2ca531db5caebbc8b6fe2d721fa6252bf466986427d' },
      { id: 'S8', step_index: 8, terms: [{ surface_form: 'bhavati', kind: 'pada' }], hash: 'state:sha256:e647bc51b9e6f3128ef35b2e95a9992d9dcfdcb95e1e1a539bc27fa7a26f8664' }
    ],
    cryptographic_proof: { verified: true }
  },
  dadati: {
    ir_version: 'panini-derivation-ir/0.1',
    derivation_id: 'drv:canonical:dadati-v0.1',
    target_word: 'ददाति (dadāti)',
    description: 'Derivation of root √dā showing Paribhāṣā conflict resolution.',
    status: 'success',
    final_surface_form: 'dadāti',
    root: '√dā (दा)',
    states: [
      { id: 'S0', step_index: 0, terms: [{ surface_form: 'dā', kind: 'dhAtu' }], hash: 'state:sha256:4b9101f3b0e35261d7b322a3683fba227653bbd0a3d46386c67fa128adbc44e1' },
      { id: 'S1', step_index: 1, terms: [{ surface_form: 'dā', kind: 'dhAtu' }, { surface_form: 'laṭ', kind: 'lakAra' }], hash: 'state:sha256:5c83b8909187a4de88d927a4e6bbcbcfd263914a8b7c4d51b3ca00d7fae01931' },
      { id: 'S2', step_index: 2, terms: [{ surface_form: 'dā', kind: 'dhAtu' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:773ab485be61d67a6d10c144e0573e86cbca388d75cf7940ee95a947bc18a514' },
      { id: 'S3', step_index: 3, terms: [{ surface_form: 'dā', kind: 'dhAtu' }, { surface_form: 'ti', kind: 'pratyaya' }], applied_rule: { sutra_id: '2.4.75', paribhasha_principle: 'Apavāda > Utsarga' }, hash: 'state:sha256:99f8d17baae7752e5d95d18d4512e0e0f523173d12d09ff93ff790f977beaece' },
      { id: 'S4', step_index: 4, terms: [{ surface_form: 'dā', kind: 'abhyAsa' }, { surface_form: 'dā', kind: 'dhAtu' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:31b1a7d6560daaf47aa6a4387d8cb27efc90b63cbb01a84f339cf01cb4d14210' },
      { id: 'S5', step_index: 5, terms: [{ surface_form: 'da', kind: 'abhyAsa' }, { surface_form: 'dā', kind: 'dhAtu' }, { surface_form: 'ti', kind: 'pratyaya' }], hash: 'state:sha256:56bce457c1d3bf30f04c643b1859c25db94474b34b6b6685718dfd374aa3e843' },
      { id: 'S6', step_index: 6, terms: [{ surface_form: 'dadāti', kind: 'pada' }], hash: 'state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627' }
    ],
    cryptographic_proof: { verified: true }
  }
};

export const PHONEME_DATA_REGISTRY = {
  a: {
    phoneme: 'a',
    slp1: 'a',
    deva: 'अ',
    upc8: 0x00,
    upc8Hex: '0x00',
    pvc16: {
      raw: 0x0003,
      isVowel: true,
      sthana: { kanthya: true, talavya: false, murdhanya: false, dantya: false, osthya: false, name: 'Kaṇṭhya (Velar)' },
      prayatna: { sprsta: false, mahaprana: false, ghosa: true, anunasika: false, name: 'Vivṛta' },
      modifier: { isPalatalized: false, isExtension: false }
    },
    isUkrainian: false
  },
  k: {
    phoneme: 'k',
    slp1: 'k',
    deva: 'क्',
    upc8: 0x19,
    upc8Hex: '0x19',
    pvc16: {
      raw: 0x0042,
      isVowel: false,
      sthana: { kanthya: true, talavya: false, murdhanya: false, dantya: false, osthya: false, name: 'Kaṇṭhya (Velar)' },
      prayatna: { sprsta: true, mahaprana: false, ghosa: false, anunasika: false, name: 'Spṛṣṭa, Aghoṣa' },
      modifier: { isPalatalized: false, isExtension: false }
    },
    isUkrainian: false
  },
  "t'": {
    phoneme: "t'",
    slp1: "t'",
    deva: 'त् (т\')',
    upc8: 0x3d,
    upc8Hex: '0x3D',
    pvc16: {
      raw: 0x4048,
      isVowel: false,
      sthana: { kanthya: false, talavya: false, murdhanya: false, dantya: true, osthya: false, name: 'Dantya' },
      prayatna: { sprsta: true, mahaprana: false, ghosa: false, anunasika: false, name: 'Spṛṣṭa, M\'yakiy' },
      modifier: { isPalatalized: true, isExtension: true }
    },
    isUkrainian: true
  }
};
