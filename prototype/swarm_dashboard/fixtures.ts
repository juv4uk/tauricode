/**
 * Tauricode Swarm Telemetry Fixtures & Canonical Data
 * Nodes:
 *   - my-lisp-1       (:9101) - Core Lisp VM / Semantic Oracle / Knowledge Store
 *   - fpga-lisp-1     (:9102) - Hardware Synthesizer / ALU & Accelerator Co-processor
 *   - cml-1           (:9103) - CML Compiler Architecture & Lowering Middle-End
 *   - my-idea-1       (:9104) - IDE & Visual Tooling Agent / System Observatory
 *   - my-lisp-panini-1(:9106) - Panini Grammar Machine / Derivation IR Engine
 *   - shiva-sutras-1  (:9107) - Phonetic Engine / Śiva Sūtras & UPC-8 / PVC-16 Canon
 */

import { SwarmMeshTopology, TaskStats, DerivationTrace, PhonemeVectorData } from './types';

export const MOCK_SWARM_TOPOLOGY: SwarmMeshTopology = {
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

export const MOCK_TASK_STATS: TaskStats = {
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
  recentTasks: [
    {
      id: 'TAURICODE-SWARM-DASHBOARD-PROTOTYPE',
      nodeId: 'tauricode',
      title: 'TauriCode Swarm Node Mesh Dashboard & IPC Integration',
      description: 'Implement real-time Swarm Mesh visualizer (ports 9101-9107), live Derivation DAG streamer, and phonetic inspection workbench.',
      status: 'COMPLETED',
      priority: 0.95,
      capabilities: ['tauri', 'dashboard', 'telemetry', 'ipc', 'react', 'phonetics'],
      context: 'done: prototype/swarm_dashboard completed with standalone demo, IPC contracts, and recommendations report.',
      completedAt: '2026-08-21T09:30:00+03:00'
    },
    {
      id: 'CML-PHONETIC-LOWERING-PROTOTYPE',
      nodeId: 'cml-1',
      title: 'CML Compiler Lowering Passes for 64-Bit Bitmasks & PVC-16',
      description: 'Implement constant folding for pratyāhāra set algebra and single-cycle Sūtra 1.1.9 savarṇa predicates.',
      status: 'COMPLETED',
      priority: 0.85,
      capabilities: ['compiler', 'rust', 'lowering', 'testing', 'verilog', 'proof'],
      context: 'done: prototype/cml_lowering/ suite verified with test_cml_lowering.py.',
      completedAt: '2026-08-21T09:15:00+03:00'
    },
    {
      id: 'FPGA-ALU-PHONETIC-CORE',
      nodeId: 'fpga-lisp-1',
      title: 'Synthesizable Verilog Phonetic ALU & 42-Pratyāhāra ROM',
      description: 'Synthesizable 1-cycle Savarṇa unit, palatalization modifier, and distributed 64-bit ROM on Gowin GW5A.',
      status: 'COMPLETED',
      priority: 0.9,
      capabilities: ['verilog', 'fpga', 'alu', 'synthesis'],
      context: 'done: fpga_alu.v verified on Gowin GW5A-25A (<45 LUTs, 184 MHz).',
      completedAt: '2026-08-21T08:50:00+03:00'
    },
    {
      id: 'PANINI-PROOF-GRAPH-GENERATOR',
      nodeId: 'my-lisp-panini-1',
      title: 'Canonical Derivation DAG Generator & Paribhāṣā Resolver',
      description: 'Deterministic 32-bit DerivationCell and 5-tier Paribhāṣā priority resolver for bhavati and dadāti.',
      status: 'COMPLETED',
      priority: 0.9,
      capabilities: ['grammar', 'panini', 'derivation', 'proof-graph'],
      context: 'done: proof certificates generated with SHA-256 state hashes.',
      completedAt: '2026-08-21T08:30:00+03:00'
    },
    {
      id: 'SHIVA-UPC8-ARCHITECTURE-DECISION',
      nodeId: 'shiva-sutras-1',
      title: 'Ratification of ADR-002 UPC-8 Character Set & Collision Resolution',
      description: 'Formalize 8-bit phonetic allocation, 6 marker-sound collision resolution, and Slavic phonetic extensions.',
      status: 'COMPLETED',
      priority: 0.95,
      capabilities: ['phonetics', 'shiva', 'upc8', 'canon'],
      context: 'done: ADR-002 ratified and verified across 42 canonical sounds.',
      completedAt: '2026-08-21T08:00:00+03:00'
    }
  ]
};

export const CANONICAL_DERIVATIONS: Record<string, DerivationTrace> = {
  bhavati: {
    ir_version: 'panini-derivation-ir/0.1',
    derivation_id: 'drv:canonical:bhavati-v0.1',
    target_word: 'भवति (bhavati)',
    description: 'Derivation of root √bhū (भू सत्तायाम्, 1st gaṇa Bhvādi) in present tense 3rd person singular (laṭ, tiṅ-tip).',
    status: 'success',
    final_surface_form: 'bhavati',
    root: '√bhū (भू)',
    rules: [
      {
        sutra_id: '3.2.123',
        text_deva: 'वर्तमाने लट्',
        text_slp1: 'vartamAne laT',
        classification: 'VIDHI',
        summary: 'Affixes the lakāra "laṭ" to denote action in the present tense.'
      },
      {
        sutra_id: '3.4.78',
        text_deva: 'तिप्तस्झिसिप्थस्थमिब्वस्मस्तातांझथांसाथांध्वमिड्वहिमहिङ्',
        text_slp1: 'tiptasjhisipTasTamibvasmastAtAMJATAMsATAMDvamidvahimahiG',
        classification: 'VIDHI',
        summary: 'Substitutes lakāra with 18 tiṅ affixes; selects 3rd singular parasmaipada "tip".'
      },
      {
        sutra_id: '1.3.9',
        text_deva: 'तस्य लोपः',
        text_slp1: 'tasya lopaH',
        classification: 'VIDHI',
        summary: 'Elides the it-marker "p" while preserving the "pit" property tag.'
      },
      {
        sutra_id: '3.1.68',
        text_deva: 'कर्तरि शप्',
        text_slp1: 'kartari Sap',
        classification: 'VIDHI',
        summary: 'Inserts vikaraṇa affix "Śap" after root √bhū before sārvadhātuka affix.'
      },
      {
        sutra_id: '3.4.113',
        text_deva: 'तिङ्शित्सार्वधातुकम्',
        text_slp1: 'tiNSitsArvaDAtukam',
        classification: 'SAMJNA',
        summary: 'Assigns sārvadhātuka saṃjñā to "Śap" (Ś-it) and "tip" (tiṅ).'
      },
      {
        sutra_id: '7.3.84',
        text_deva: 'सार्वधातुकार्धधातुकयोः',
        text_slp1: 'sArvaDAtukArDaDAtukayoH',
        classification: 'VIDHI',
        summary: 'Applies guṇa substitution to the final vowel of the aṅga (ū -> o) before sārvadhātuka "a".'
      },
      {
        sutra_id: '6.1.78',
        text_deva: 'एचोऽयवायावः',
        text_slp1: 'eco \'yavAyAvaH',
        classification: 'VIDHI',
        summary: 'Sandhi replacement: "o" followed by vowel "a" transforms to "av" (bho + a -> bhav + a).'
      },
      {
        sutra_id: '1.4.14',
        text_deva: 'सुप्तिङन्तं पदम्',
        text_slp1: 'suptiGantaM padam',
        classification: 'SAMJNA',
        summary: 'Designates the complete tiṅ-inflected form "bhavati" as a valid syntactic Pada.'
      }
    ],
    states: [
      {
        id: 'state:bhavati:00-input',
        hash: 'state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7',
        step_index: 0,
        schema: 'panini-state/0.1',
        terms: [{ id: 'term:root-bhu', kind: 'dhAtu', source_form: 'bhū', surface_form: 'bhū', designations: ['dhātu', 'aṅga', 'bhvādi'] }],
        mutation_type: 'INITIAL',
        proof_verified: true
      },
      {
        id: 'state:bhavati:01-lat',
        hash: 'state:sha256:a6b610c14b7e8d75cf701be5e921d01ee3fdf9bcf693e506990d0b04c86ec596',
        step_index: 1,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-bhu', kind: 'dhAtu', source_form: 'bhū', surface_form: 'bhū', designations: ['dhātu', 'aṅga'] },
          { id: 'term:lakara-lat', kind: 'lakAra', source_form: 'laṭ', surface_form: 'laṭ', designations: ['vartamāna', 'prathama'] }
        ],
        applied_rule: {
          sutra_id: '3.2.123',
          text_deva: 'वर्तमाने लट्',
          text_slp1: 'vartamAne laT',
          classification: 'VIDHI',
          summary: 'Affixes present lakāra laṭ.'
        },
        mutation_type: 'AFFIXATION',
        diff: { added: ['+laṭ'], removed: [], transformed: [] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:02-tip',
        hash: 'state:sha256:7bc59d81d2dfbc1cf8da294c73a216db8a13ef2b8df11559eeecfc316c4c0174',
        step_index: 2,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-bhu', kind: 'dhAtu', source_form: 'bhū', surface_form: 'bhū', designations: ['dhātu', 'aṅga'] },
          { id: 'term:pratyaya-tip', kind: 'pratyaya', source_form: 'tip', surface_form: 'tip', designations: ['tiṅ', 'parasmaipada', 'pit'] }
        ],
        applied_rule: {
          sutra_id: '3.4.78',
          text_deva: 'तिप्तस्झि...',
          text_slp1: 'tiptasjhi...',
          classification: 'VIDHI',
          summary: 'Substitutes laṭ with tip (3rd singular parasmaipada).'
        },
        mutation_type: 'AFFIXATION',
        diff: { added: ['+tip'], removed: ['-laṭ'], transformed: [] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:03-it-lopa',
        hash: 'state:sha256:b1ec7daea41e06d649da391b1a7d6560daaf47aa6a4387d8cb27efc90b63cbb0',
        step_index: 3,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-bhu', kind: 'dhAtu', source_form: 'bhū', surface_form: 'bhū', designations: ['dhātu', 'aṅga'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ', 'pit'] }
        ],
        applied_rule: {
          sutra_id: '1.3.9',
          text_deva: 'तस्य लोपः',
          text_slp1: 'tasya lopaH',
          classification: 'VIDHI',
          summary: 'Elides it-marker p in tip -> ti.'
        },
        mutation_type: 'LOPA',
        diff: { added: [], removed: ['-p (it)'], transformed: [{ from: 'tip', to: 'ti' }] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:04-sap',
        hash: 'state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627',
        step_index: 4,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-bhu', kind: 'dhAtu', source_form: 'bhū', surface_form: 'bhū', designations: ['dhātu', 'aṅga'] },
          { id: 'term:vikarana-sap', kind: 'pratyaya', source_form: 'śap', surface_form: 'śap', designations: ['vikaraṇa', 'śit', 'pit'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ', 'pit'] }
        ],
        applied_rule: {
          sutra_id: '3.1.68',
          text_deva: 'कर्तरि शप्',
          text_slp1: 'kartari Sap',
          classification: 'VIDHI',
          summary: 'Inserts vikaraṇa affix śap between bhū and ti.'
        },
        mutation_type: 'AFFIXATION',
        diff: { added: ['+śap'], removed: [], transformed: [] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:05-sap-lopa',
        hash: 'state:sha256:56bce457c1d3bf30f04c643b1859c25db94474b34b6b6685718dfd374aa3e843',
        step_index: 5,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-bhu', kind: 'dhAtu', source_form: 'bhū', surface_form: 'bhū', designations: ['dhātu', 'aṅga'] },
          { id: 'term:vikarana-a', kind: 'pratyaya', source_form: 'śap', surface_form: 'a', designations: ['vikaraṇa', 'sārvadhātuka'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ', 'sārvadhātuka'] }
        ],
        applied_rule: {
          sutra_id: '3.4.113',
          text_deva: 'तिङ्शित्सार्वधातुकम्',
          text_slp1: 'tiNSitsArvaDAtukam',
          classification: 'SAMJNA',
          summary: 'Designates śap and ti as sārvadhātuka; elides ś and p -> a.'
        },
        mutation_type: 'SAMJNA',
        diff: { added: [], removed: ['-ś (it)', '-p (it)'], transformed: [{ from: 'śap', to: 'a' }] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:06-guna',
        hash: 'state:sha256:0d18d4512e0e0f523173d12d09ff93ff790f977beaece5f7fe51dc1b764bfe47',
        step_index: 6,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:anga-bho', kind: 'aGga', source_form: 'bhū', surface_form: 'bho', designations: ['aṅga', 'guṇa-applied'] },
          { id: 'term:vikarana-a', kind: 'pratyaya', source_form: 'śap', surface_form: 'a', designations: ['vikaraṇa', 'sārvadhātuka'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ', 'sārvadhātuka'] }
        ],
        applied_rule: {
          sutra_id: '7.3.84',
          text_deva: 'सार्वधातुकार्धधातुकयोः',
          text_slp1: 'sArvaDAtukArDaDAtukayoH',
          classification: 'VIDHI',
          summary: 'Applies guṇa to root vowel: bhū -> bho.'
        },
        mutation_type: 'GUNA',
        diff: { added: [], removed: [], transformed: [{ from: 'bhū', to: 'bho' }] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:07-sandhi',
        hash: 'state:sha256:8f4c281df6be48a5df9ee2ca531db5caebbc8b6fe2d721fa6252bf466986427d',
        step_index: 7,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:stem-bhav', kind: 'aGga', source_form: 'bho', surface_form: 'bhav', designations: ['aṅga', 'sandhi-applied'] },
          { id: 'term:vikarana-a', kind: 'pratyaya', source_form: 'śap', surface_form: 'a', designations: ['vikaraṇa'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ'] }
        ],
        applied_rule: {
          sutra_id: '6.1.78',
          text_deva: 'एचोऽयवायावः',
          text_slp1: 'eco \'yavAyAvaH',
          classification: 'VIDHI',
          summary: 'Sandhi: o + a -> av + a (bhav + a).'
        },
        mutation_type: 'SANDHI',
        diff: { added: [], removed: [], transformed: [{ from: 'bho + a', to: 'bhav + a' }] },
        proof_verified: true
      },
      {
        id: 'state:bhavati:08-terminal',
        hash: 'state:sha256:e647bc51b9e6f3128ef35b2e95a9992d9dcfdcb95e1e1a539bc27fa7a26f8664',
        step_index: 8,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:final-pada', kind: 'pada', source_form: 'bhū+tip', surface_form: 'bhavati', designations: ['pada', 'tiṅanta', 'kartari', 'lat', 'prathama', 'ekavacana'] }
        ],
        applied_rule: {
          sutra_id: '1.4.14',
          text_deva: 'सुप्तिङन्तं पदम्',
          text_slp1: 'suptiGantaM padam',
          classification: 'SAMJNA',
          summary: 'Declares bhavati as a completed syntactic Pada.'
        },
        mutation_type: 'SAMJNA',
        diff: { added: ['bhavati (Pada)'], removed: [], transformed: [] },
        proof_verified: true
      }
    ],
    cryptographic_proof: {
      root_hash: 'state:sha256:d8c5208bcae41a0ba1a29f8f2b7d483fb3ceb8b09d0cb093ffea75bbcaee1ea7',
      terminal_hash: 'state:sha256:e647bc51b9e6f3128ef35b2e95a9992d9dcfdcb95e1e1a539bc27fa7a26f8664',
      algorithm: 'SHA-256',
      verified: true
    }
  },
  dadati: {
    ir_version: 'panini-derivation-ir/0.1',
    derivation_id: 'drv:canonical:dadati-v0.1',
    target_word: 'ददाति (dadāti)',
    description: 'Derivation of root √dā (डुदाञ् दाने, 3rd gaṇa Juhotyādi) in present tense 3rd person singular showing Paribhāṣā conflict resolution (Ślu blocking Śap by Apavāda).',
    status: 'success',
    final_surface_form: 'dadāti',
    root: '√dā (दा)',
    rules: [
      {
        sutra_id: '3.2.123',
        text_deva: 'वर्तमाने लट्',
        text_slp1: 'vartamAne laT',
        classification: 'VIDHI',
        summary: 'Affixes present lakāra laṭ.'
      },
      {
        sutra_id: '3.4.78',
        text_deva: 'तिप्तस्झि...',
        text_slp1: 'tiptasjhi...',
        classification: 'VIDHI',
        summary: 'Substitutes laṭ with tip (3rd singular).'
      },
      {
        sutra_id: '2.4.75',
        text_deva: 'जुहोत्यादिभ्यः श्लुः',
        text_slp1: 'juhotyAdibhyaH SluH',
        classification: 'VIDHI',
        summary: 'Replaces vikaraṇa Śap with Ślu (zero replacement) for 3rd class roots.',
        paribhasha_principle: 'Apavāda > Utsarga (Special exception 2.4.75 completely blocks general 3.1.68 Śap)',
        blocked_sutras: ['3.1.68 kartari śap']
      },
      {
        sutra_id: '6.1.10',
        text_deva: 'श्लौ',
        text_slp1: 'SlO',
        classification: 'VIDHI',
        summary: 'Reduplicates the root when Ślu follows (dā -> dā + dā).'
      },
      {
        sutra_id: '7.4.59',
        text_deva: 'ह्रस्वः',
        text_slp1: 'hrasvaH',
        classification: 'VIDHI',
        summary: 'Shortens the vowel of the reduplicated abhyāsa syllable: dā -> da.'
      },
      {
        sutra_id: '1.4.14',
        text_deva: 'सुप्तिङन्तं पदम्',
        text_slp1: 'suptiGantaM padam',
        classification: 'SAMJNA',
        summary: 'Declares dadāti as a valid Pada.'
      }
    ],
    states: [
      {
        id: 'state:dadati:00-input',
        hash: 'state:sha256:4b9101f3b0e35261d7b322a3683fba227653bbd0a3d46386c67fa128adbc44e1',
        step_index: 0,
        schema: 'panini-state/0.1',
        terms: [{ id: 'term:root-da', kind: 'dhAtu', source_form: 'dā', surface_form: 'dā', designations: ['dhātu', 'juhotyādi'] }],
        mutation_type: 'INITIAL',
        proof_verified: true
      },
      {
        id: 'state:dadati:01-lat',
        hash: 'state:sha256:5c83b8909187a4de88d927a4e6bbcbcfd263914a8b7c4d51b3ca00d7fae01931',
        step_index: 1,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-da', kind: 'dhAtu', source_form: 'dā', surface_form: 'dā', designations: ['dhātu'] },
          { id: 'term:lakara-lat', kind: 'lakAra', source_form: 'laṭ', surface_form: 'laṭ', designations: ['vartamāna'] }
        ],
        applied_rule: {
          sutra_id: '3.2.123',
          text_deva: 'वर्तमाने लट्',
          text_slp1: 'vartamAne laT',
          classification: 'VIDHI',
          summary: 'Affixes present lakāra laṭ.'
        },
        mutation_type: 'AFFIXATION',
        proof_verified: true
      },
      {
        id: 'state:dadati:02-tip',
        hash: 'state:sha256:773ab485be61d67a6d10c144e0573e86cbca388d75cf7940ee95a947bc18a514',
        step_index: 2,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-da', kind: 'dhAtu', source_form: 'dā', surface_form: 'dā', designations: ['dhātu'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ', 'pit'] }
        ],
        applied_rule: {
          sutra_id: '3.4.78',
          text_deva: 'तिप्तस्झि...',
          text_slp1: 'tiptasjhi...',
          classification: 'VIDHI',
          summary: 'Selects tip affix.'
        },
        mutation_type: 'AFFIXATION',
        proof_verified: true
      },
      {
        id: 'state:dadati:03-slu-block',
        hash: 'state:sha256:99f8d17baae7752e5d95d18d4512e0e0f523173d12d09ff93ff790f977beaece',
        step_index: 3,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:root-da', kind: 'dhAtu', source_form: 'dā', surface_form: 'dā', designations: ['dhātu', 'ślu-environment'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ'] }
        ],
        applied_rule: {
          sutra_id: '2.4.75',
          text_deva: 'जुहोत्यादिभ्यः श्लुः',
          text_slp1: 'juhotyAdibhyaH SluH',
          classification: 'VIDHI',
          summary: 'Ślu replaces Śap; 3.1.68 blocked by Apavāda priority.',
          paribhasha_principle: 'Apavāda > Utsarga'
        },
        mutation_type: 'LOPA',
        diff: { added: ['Ślu (zero-affix)'], removed: ['Śap (blocked)'], transformed: [] },
        proof_verified: true
      },
      {
        id: 'state:dadati:04-reduplication',
        hash: 'state:sha256:31b1a7d6560daaf47aa6a4387d8cb27efc90b63cbb01a84f339cf01cb4d14210',
        step_index: 4,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:abhyasa-da', kind: 'abhyAsa', source_form: 'dā', surface_form: 'dā', designations: ['abhyāsa'] },
          { id: 'term:root-da', kind: 'dhAtu', source_form: 'dā', surface_form: 'dā', designations: ['dhātu', 'aṅga'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ'] }
        ],
        applied_rule: {
          sutra_id: '6.1.10',
          text_deva: 'श्लौ',
          text_slp1: 'SlO',
          classification: 'VIDHI',
          summary: 'Reduplicates root: dā -> dā + dā.'
        },
        mutation_type: 'AFFIXATION',
        diff: { added: ['+dā (abhyāsa)'], removed: [], transformed: [] },
        proof_verified: true
      },
      {
        id: 'state:dadati:05-abhyasa-shorten',
        hash: 'state:sha256:56bce457c1d3bf30f04c643b1859c25db94474b34b6b6685718dfd374aa3e843',
        step_index: 5,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:abhyasa-da-short', kind: 'abhyAsa', source_form: 'dā', surface_form: 'da', designations: ['abhyāsa', 'hrasva'] },
          { id: 'term:root-da', kind: 'dhAtu', source_form: 'dā', surface_form: 'dā', designations: ['dhātu', 'aṅga'] },
          { id: 'term:pratyaya-ti', kind: 'pratyaya', source_form: 'tip', surface_form: 'ti', designations: ['tiṅ'] }
        ],
        applied_rule: {
          sutra_id: '7.4.59',
          text_deva: 'ह्रस्वः',
          text_slp1: 'hrasvaH',
          classification: 'VIDHI',
          summary: 'Shortens abhyāsa vowel: dā -> da.'
        },
        mutation_type: 'SAMJNA',
        diff: { added: [], removed: [], transformed: [{ from: 'dā (abhyāsa)', to: 'da' }] },
        proof_verified: true
      },
      {
        id: 'state:dadati:06-terminal',
        hash: 'state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627',
        step_index: 6,
        schema: 'panini-state/0.1',
        terms: [
          { id: 'term:final-pada-dadati', kind: 'pada', source_form: 'dā+tip', surface_form: 'dadāti', designations: ['pada', 'tiṅanta', 'juhotyādi', 'prathama', 'ekavacana'] }
        ],
        applied_rule: {
          sutra_id: '1.4.14',
          text_deva: 'सुप्तिङन्तं पदम्',
          text_slp1: 'suptiGantaM padam',
          classification: 'SAMJNA',
          summary: 'Declares dadāti as a valid syntactic Pada.'
        },
        mutation_type: 'SAMJNA',
        diff: { added: ['dadāti (Pada)'], removed: [], transformed: [] },
        proof_verified: true
      }
    ],
    cryptographic_proof: {
      root_hash: 'state:sha256:4b9101f3b0e35261d7b322a3683fba227653bbd0a3d46386c67fa128adbc44e1',
      terminal_hash: 'state:sha256:1a84f339cf01cb4d14210459c3d40ef6731be7a7bf734e40e6573c52e8055627',
      algorithm: 'SHA-256',
      verified: true
    }
  }
};

export const PHONEME_DATA_REGISTRY: Record<string, PhonemeVectorData> = {
  a: {
    phoneme: 'a',
    slp1: 'a',
    deva: 'अ',
    upc8: 0x00,
    upc8Hex: '0x00',
    pvc16: {
      raw: 0x0003,
      hex: '0x0003',
      binary: '0000000000000011',
      isVowel: true,
      sthana: { kanthya: true, talavya: false, murdhanya: false, dantya: false, osthya: false, name: 'Kaṇṭhya (Velar/Guttural)' },
      prayatna: { sprsta: false, mahaprana: false, ghosa: true, anunasika: false, name: 'Vivṛta / Saṃvṛta' },
      svara: { length: 'hrasva', accent: 'udatta' },
      modifier: { isPalatalized: false, isExtension: false }
    },
    pratyaharaBitIndex: 0,
    pratyaharaMaskU64: '0x0000000000000001',
    pratyaharasContained: ['ac', 'al', 'ak', 'aṇ', 'am', 'aś'],
    isUkrainian: false,
    ipa: '[ɐ] / [ə]'
  },
  i: {
    phoneme: 'i',
    slp1: 'i',
    deva: 'इ',
    upc8: 0x01,
    upc8Hex: '0x01',
    pvc16: {
      raw: 0x0005,
      hex: '0x0005',
      binary: '0000000000000101',
      isVowel: true,
      sthana: { kanthya: false, talavya: true, murdhanya: false, dantya: false, osthya: false, name: 'Tālavya (Palatal)' },
      prayatna: { sprsta: false, mahaprana: false, ghosa: true, anunasika: false, name: 'Spṛṣṭa-vivṛta' },
      svara: { length: 'hrasva', accent: 'udatta' },
      modifier: { isPalatalized: false, isExtension: false }
    },
    pratyaharaBitIndex: 1,
    pratyaharaMaskU64: '0x0000000000000002',
    pratyaharasContained: ['ac', 'al', 'ik', 'iṇ', 'ic', 'ak', 'aṇ'],
    isUkrainian: false,
    ipa: '[i]'
  },
  u: {
    phoneme: 'u',
    slp1: 'u',
    deva: 'उ',
    upc8: 0x02,
    upc8Hex: '0x02',
    pvc16: {
      raw: 0x0021,
      hex: '0x0021',
      binary: '0000000000100001',
      isVowel: true,
      sthana: { kanthya: false, talavya: false, murdhanya: false, dantya: false, osthya: true, name: 'Oṣṭhya (Labial)' },
      prayatna: { sprsta: false, mahaprana: false, ghosa: true, anunasika: false, name: 'Vivṛta' },
      svara: { length: 'hrasva', accent: 'udatta' },
      modifier: { isPalatalized: false, isExtension: false }
    },
    pratyaharaBitIndex: 2,
    pratyaharaMaskU64: '0x0000000000000004',
    pratyaharasContained: ['ac', 'al', 'uk', 'ik', 'ak', 'aṇ'],
    isUkrainian: false,
    ipa: '[u]'
  },
  k: {
    phoneme: 'k',
    slp1: 'k',
    deva: 'क्',
    upc8: 0x19,
    upc8Hex: '0x19',
    pvc16: {
      raw: 0x0042,
      hex: '0x0042',
      binary: '0000000001000010',
      isVowel: false,
      sthana: { kanthya: true, talavya: false, murdhanya: false, dantya: false, osthya: false, name: 'Kaṇṭhya (Velar)' },
      prayatna: { sprsta: true, mahaprana: false, ghosa: false, anunasika: false, name: 'Spṛṣṭa, Aghoṣa, Alpaprāṇa' },
      svara: { length: 'hrasva', accent: 'udatta' },
      modifier: { isPalatalized: false, isExtension: false }
    },
    pratyaharaBitIndex: 25,
    pratyaharaMaskU64: '0x0000000002000000',
    pratyaharasContained: ['hal', 'al', 'jhay', 'khay', 'cay', 'yar', 'khar'],
    isUkrainian: false,
    ipa: '[k]'
  },
  t_palatal: {
    phoneme: "t'",
    slp1: "t'",
    deva: 'त् (т\')',
    upc8: 0x3d,
    upc8Hex: '0x3D',
    pvc16: {
      raw: 0x4048,
      hex: '0x4048',
      binary: '0100000001001000',
      isVowel: false,
      sthana: { kanthya: false, talavya: false, murdhanya: false, dantya: true, osthya: false, name: 'Dantya + Palatalized' },
      prayatna: { sprsta: true, mahaprana: false, ghosa: false, anunasika: false, name: 'Spṛṣṭa, Aghoṣa, M'yakiy' },
      svara: { length: 'hrasva', accent: 'udatta' },
      modifier: { isPalatalized: true, isExtension: true }
    },
    pratyaharaBitIndex: 43,
    pratyaharaMaskU64: '0x0000080000000000',
    pratyaharasContained: ['ukrainian_palatalized', 'hal_extended'],
    isUkrainian: true,
    ipa: '[tʲ]'
  }
};
