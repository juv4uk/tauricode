/**
 * PhoneticInspector - Interactive Articulatory Workbench & Bitmask Tester
 * Features 16-bit PVC-16 vector inspection, 64-bit Pratyāhāra ALU testing, and Savarṇa (1.1.9) calculator.
 */

import React, { useState } from 'react';
import { PhonemeVectorData } from '../types';
import { PHONEME_DATA_REGISTRY } from '../fixtures';

export const PhoneticInspector: React.FC = () => {
  const [selectedPhoneme, setSelectedPhoneme] = useState<string>('a');
  const [comparePhoneme, setComparePhoneme] = useState<string>('i');
  const [rawPvc16, setRawPvc16] = useState<number>(0x0003);

  const activeData: PhonemeVectorData =
    PHONEME_DATA_REGISTRY[selectedPhoneme] || PHONEME_DATA_REGISTRY['a'];
  const compData: PhonemeVectorData =
    PHONEME_DATA_REGISTRY[comparePhoneme] || PHONEME_DATA_REGISTRY['i'];

  const handleSelectPhoneme = (p: string) => {
    setSelectedPhoneme(p);
    const item = PHONEME_DATA_REGISTRY[p];
    if (item) {
      setRawPvc16(item.pvc16.raw);
    }
  };

  const toggleBit = (bitIndex: number) => {
    setRawPvc16((prev) => prev ^ (1 << bitIndex));
  };

  // Sūtra 1.1.9 Savarṇa Test: (sthāna == sthāna) && (prayatna == prayatna) && (is_vowel == is_vowel)
  const isSavarna =
    (activeData.pvc16.raw & 0x003e) === (compData.pvc16.raw & 0x003e) &&
    (activeData.pvc16.raw & 0x0041) === (compData.pvc16.raw & 0x0041);

  return (
    <div className="space-y-6">
      {/* Top Phoneme Picker Bar */}
      <div className="bg-slate-900/80 border border-slate-800 p-4 rounded-xl">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h3 className="text-sm font-semibold text-slate-200 uppercase tracking-wider font-mono">
              Phonetic Engine Workbench (UPC-8 & PVC-16)
            </h3>
            <p className="text-xs text-slate-400 mt-0.5">
              16-bit articulatory register & 64-bit Pratyāhāra ALU inspection
            </p>
          </div>

          {/* Quick Phoneme Selectors */}
          <div className="flex flex-wrap gap-2">
            {Object.keys(PHONEME_DATA_REGISTRY).map((p) => {
              const item = PHONEME_DATA_REGISTRY[p];
              const isSel = selectedPhoneme === p;
              return (
                <button
                  key={p}
                  onClick={() => handleSelectPhoneme(p)}
                  className={`px-3 py-1.5 rounded-lg text-xs font-mono font-bold transition ${
                    isSel
                      ? 'bg-sky-600 text-white ring-2 ring-sky-400'
                      : 'bg-slate-800 text-slate-300 hover:bg-slate-700'
                  }`}
                >
                  <span className="text-sm mr-1">{item.deva}</span>
                  <span>{item.phoneme}</span>
                  {item.isUkrainian && <span className="ml-1 text-[10px] text-amber-300">UA</span>}
                </button>
              );
            })}
          </div>
        </div>
      </div>

      {/* Main Vector Inspector Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* PVC-16 Bit Register Panel */}
        <div className="lg:col-span-2 bg-slate-900/80 border border-slate-800 p-5 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="px-2 py-0.5 rounded bg-sky-950 text-sky-400 text-xs font-mono border border-sky-800">
                PVC-16 REGISTER
              </span>
              <h4 className="font-mono font-bold text-slate-100">
                0x{rawPvc16.toString(16).toUpperCase().padStart(4, '0')}
              </h4>
            </div>
            <span className="text-xs font-mono text-slate-400">16-bit Articulatory Vector</span>
          </div>

          {/* 16 Interactive Bit Cells */}
          <div className="grid grid-cols-8 sm:grid-cols-16 gap-1.5 pt-2">
            {Array.from({ length: 16 }).map((_, i) => {
              const bit = 15 - i;
              const isSet = Boolean((rawPvc16 >> bit) & 1);
              return (
                <button
                  key={bit}
                  onClick={() => toggleBit(bit)}
                  className={`flex flex-col items-center justify-center p-2 rounded border transition font-mono ${
                    isSet
                      ? 'bg-sky-600/30 border-sky-500 text-sky-300'
                      : 'bg-slate-950/60 border-slate-800 text-slate-600 hover:border-slate-700'
                  }`}
                >
                  <span className="text-[9px] text-slate-400">{bit}</span>
                  <span className="text-xs font-bold">{isSet ? '1' : '0'}</span>
                </button>
              );
            })}
          </div>

          {/* Bit Field Legends */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2 pt-2 text-xs font-mono">
            <div className="bg-slate-950/60 p-2 rounded border border-slate-800/80">
              <span className="text-[10px] text-slate-500 uppercase block">Bit 0: Class</span>
              <span className="font-bold text-sky-400">
                {(rawPvc16 & 1) ? 'VOWEL (ac)' : 'CONSONANT (hal)'}
              </span>
            </div>
            <div className="bg-slate-950/60 p-2 rounded border border-slate-800/80">
              <span className="text-[10px] text-slate-500 uppercase block">Bits [5:1]: Sthāna</span>
              <span className="font-bold text-indigo-400">
                {activeData.pvc16.sthana.name}
              </span>
            </div>
            <div className="bg-slate-950/60 p-2 rounded border border-slate-800/80">
              <span className="text-[10px] text-slate-500 uppercase block">Bits [9:6]: Prayatna</span>
              <span className="font-bold text-purple-400">
                {activeData.pvc16.prayatna.name}
              </span>
            </div>
            <div className="bg-slate-950/60 p-2 rounded border border-slate-800/80">
              <span className="text-[10px] text-slate-500 uppercase block">Bit 14: Modifier</span>
              <span className={`font-bold ${activeData.pvc16.modifier.isPalatalized ? 'text-amber-400' : 'text-slate-400'}`}>
                {activeData.pvc16.modifier.isPalatalized ? 'Palatalized [ь]' : 'Plain'}
              </span>
            </div>
          </div>

          {/* Pratyāhāra Membership List */}
          <div className="p-3 bg-slate-950/60 rounded-lg border border-slate-800/80 space-y-2">
            <div className="flex items-center justify-between text-xs font-mono">
              <span className="text-slate-400">Pratyāhāras Containing '{activeData.phoneme}':</span>
              <span className="text-sky-400">64-bit Mask: {activeData.pratyaharaMaskU64}</span>
            </div>
            <div className="flex flex-wrap gap-1.5">
              {activeData.pratyaharasContained.map((pr) => (
                <span
                  key={pr}
                  className="px-2 py-0.5 rounded bg-indigo-950 text-indigo-300 border border-indigo-800 text-xs font-mono"
                >
                  {pr}
                </span>
              ))}
            </div>
          </div>
        </div>

        {/* Savarṇa Comparator Panel (Sūtra 1.1.9) */}
        <div className="bg-slate-900/80 border border-slate-800 p-5 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <span className="px-2 py-0.5 rounded bg-purple-950 text-purple-400 text-xs font-mono border border-purple-800">
              SŪTRA 1.1.9
            </span>
            <h4 className="text-xs font-mono text-slate-400">tulyāsyaprayatnaṁ savarṇam</h4>
          </div>

          <p className="text-xs text-slate-400">
            Computes articulatory homogeneity in 1 clock cycle via bitwise mask:
          </p>

          {/* Comparator Selection */}
          <div className="flex items-center justify-between gap-3 p-3 bg-slate-950/70 rounded-lg border border-slate-800">
            <div className="text-center flex-1">
              <span className="text-[10px] text-slate-500 block">Sound A</span>
              <span className="text-lg font-bold text-sky-400 font-mono">{activeData.phoneme}</span>
              <span className="text-xs text-slate-400 block">{activeData.deva}</span>
            </div>
            <div className="text-xs font-mono text-slate-500 font-bold">VS</div>
            <div className="text-center flex-1">
              <span className="text-[10px] text-slate-500 block">Sound B</span>
              <select
                value={comparePhoneme}
                onChange={(e) => setComparePhoneme(e.target.value)}
                className="bg-slate-800 border border-slate-700 text-slate-200 text-xs rounded px-2 py-1 font-mono mt-1"
              >
                {Object.keys(PHONEME_DATA_REGISTRY).map((p) => (
                  <option key={p} value={p}>
                    {PHONEME_DATA_REGISTRY[p].phoneme} ({PHONEME_DATA_REGISTRY[p].deva})
                  </option>
                ))}
              </select>
            </div>
          </div>

          {/* Savarṇa Result Gauge */}
          <div
            className={`p-4 rounded-xl border text-center transition ${
              isSavarna
                ? 'bg-emerald-950/50 border-emerald-500/80 shadow-lg shadow-emerald-500/10'
                : 'bg-rose-950/50 border-rose-500/80 shadow-lg shadow-rose-500/10'
            }`}
          >
            <span className="text-[11px] uppercase font-mono tracking-wider block text-slate-400">
              Savarṇa Homogeneity Result
            </span>
            <div className={`text-xl font-bold font-mono mt-1 ${isSavarna ? 'text-emerald-400' : 'text-rose-400'}`}>
              {isSavarna ? 'SAVARṆA (HOMOGENEOUS)' : 'ASAVARṆA (DISTINCT)'}
            </div>
            <span className="text-xs text-slate-400 font-mono mt-1 block">
              FPGA ALU Cost: 8 LUTs · Latency: 1 Cycle (~0.3 ns)
            </span>
          </div>
        </div>
      </div>
    </div>
  );
};
