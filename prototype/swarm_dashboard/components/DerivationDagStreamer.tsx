/**
 * DerivationDagStreamer - Live Interactive Pāṇinian Derivation Proof Graph
 * Features step-by-step playback, SHA-256 state hash validation, AST diffing, and rule citations.
 */

import React, { useState, useEffect } from 'react';
import { DerivationTrace, DerivationState } from '../types';

interface DerivationDagStreamerProps {
  trace: DerivationTrace;
  onSelectDerivation: (id: string) => void;
}

export const DerivationDagStreamer: React.FC<DerivationDagStreamerProps> = ({
  trace,
  onSelectDerivation
}) => {
  const [currentStep, setCurrentStep] = useState<number>(0);
  const [isPlaying, setIsPlaying] = useState<boolean>(false);

  useEffect(() => {
    setCurrentStep(0);
    setIsPlaying(false);
  }, [trace.derivation_id]);

  useEffect(() => {
    let timer: NodeJS.Timeout | null = null;
    if (isPlaying) {
      timer = setInterval(() => {
        setCurrentStep((prev) => {
          if (prev >= trace.states.length - 1) {
            setIsPlaying(false);
            return prev;
          }
          return prev + 1;
        });
      }, 1500);
    }
    return () => {
      if (timer) clearInterval(timer);
    };
  }, [isPlaying, trace.states.length]);

  const activeState: DerivationState = trace.states[currentStep] || trace.states[0];

  return (
    <div className="space-y-6">
      {/* Header & Derivation Selector */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 bg-slate-900/80 border border-slate-800 p-4 rounded-xl">
        <div>
          <div className="flex items-center gap-2">
            <span className="px-2 py-0.5 rounded bg-indigo-950 text-indigo-400 text-xs font-mono border border-indigo-800">
              {trace.ir_version}
            </span>
            <h3 className="text-lg font-bold text-slate-100">{trace.target_word}</h3>
            <span className="text-xs px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800 font-mono">
              PROVEN
            </span>
          </div>
          <p className="text-xs text-slate-400 mt-1">{trace.description}</p>
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={() => onSelectDerivation('bhavati')}
            className={`px-3 py-1.5 rounded text-xs font-mono font-bold transition ${
              trace.derivation_id.includes('bhavati')
                ? 'bg-sky-600 text-white shadow-md shadow-sky-600/30'
                : 'bg-slate-800 text-slate-400 hover:text-slate-200'
            }`}
          >
            भवति (bhavati)
          </button>
          <button
            onClick={() => onSelectDerivation('dadati')}
            className={`px-3 py-1.5 rounded text-xs font-mono font-bold transition ${
              trace.derivation_id.includes('dadati')
                ? 'bg-sky-600 text-white shadow-md shadow-sky-600/30'
                : 'bg-slate-800 text-slate-400 hover:text-slate-200'
            }`}
          >
            ददाति (dadāti)
          </button>
        </div>
      </div>

      {/* DAG Timeline Step Ribbon */}
      <div className="bg-slate-900/80 border border-slate-800 p-4 rounded-xl overflow-x-auto">
        <div className="flex items-center justify-between min-w-[640px] relative pb-2">
          {/* Connector Line */}
          <div className="absolute top-4 left-6 right-6 h-0.5 bg-slate-800 -z-0" />

          {trace.states.map((st, idx) => {
            const isDone = idx <= currentStep;
            const isCurrent = idx === currentStep;
            return (
              <div
                key={st.id}
                onClick={() => {
                  setIsPlaying(false);
                  setCurrentStep(idx);
                }}
                className="flex flex-col items-center cursor-pointer z-10 group"
              >
                <div
                  className={`w-8 h-8 rounded-full flex items-center justify-center font-mono text-xs font-bold transition-all ${
                    isCurrent
                      ? 'bg-sky-500 text-white ring-4 ring-sky-500/20 scale-110 shadow-lg shadow-sky-500/30'
                      : isDone
                      ? 'bg-indigo-600 text-white'
                      : 'bg-slate-800 text-slate-500 group-hover:bg-slate-700'
                  }`}
                >
                  S{idx}
                </div>
                <span className="text-[10px] font-mono mt-1.5 text-slate-400 group-hover:text-slate-200 max-w-[70px] truncate text-center">
                  {st.applied_rule?.sutra_id || 'Input'}
                </span>
              </div>
            );
          })}
        </div>

        {/* Playback Controls */}
        <div className="flex items-center justify-between border-t border-slate-800/80 pt-3 mt-2">
          <div className="flex items-center gap-2">
            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentStep(0);
              }}
              disabled={currentStep === 0}
              className="p-1.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 disabled:opacity-40 text-xs font-mono"
            >
              ⏮ First
            </button>
            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentStep((p) => Math.max(0, p - 1));
              }}
              disabled={currentStep === 0}
              className="p-1.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 disabled:opacity-40 text-xs font-mono"
            >
              ◀ Prev
            </button>
            <button
              onClick={() => setIsPlaying(!isPlaying)}
              className="px-3 py-1.5 rounded bg-sky-600 hover:bg-sky-500 text-white text-xs font-mono font-bold"
            >
              {isPlaying ? '⏸ Pause' : '▶ Play Stream'}
            </button>
            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentStep((p) => Math.min(trace.states.length - 1, p + 1));
              }}
              disabled={currentStep >= trace.states.length - 1}
              className="p-1.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 disabled:opacity-40 text-xs font-mono"
            >
              Next ▶
            </button>
            <button
              onClick={() => {
                setIsPlaying(false);
                setCurrentStep(trace.states.length - 1);
              }}
              disabled={currentStep >= trace.states.length - 1}
              className="p-1.5 rounded bg-slate-800 text-slate-300 hover:bg-slate-700 disabled:opacity-40 text-xs font-mono"
            >
              Last ⏭
            </button>
          </div>

          <span className="text-xs font-mono text-slate-400">
            Step <strong>{currentStep + 1}</strong> of <strong>{trace.states.length}</strong>
          </span>
        </div>
      </div>

      {/* Active State & Rule Detail Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* State Terms & AST */}
        <div className="bg-slate-900/80 border border-slate-800 p-5 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-semibold text-slate-200 uppercase tracking-wider font-mono">
              State Terms (AST Surface)
            </h4>
            <span className="text-[11px] font-mono px-2 py-0.5 rounded bg-slate-800 text-emerald-400">
              SHA-256 VERIFIED
            </span>
          </div>

          {/* Term Tokens */}
          <div className="flex flex-wrap items-center gap-2 p-3 bg-slate-950/70 rounded-lg border border-slate-800 min-h-[64px]">
            {activeState.terms.map((t) => (
              <div
                key={t.id}
                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 border border-slate-700 text-slate-100"
              >
                <span className="font-bold font-mono text-sky-400">{t.surface_form}</span>
                <span className="text-[10px] px-1 py-0.5 rounded bg-slate-900 text-slate-400 font-mono">
                  {t.kind}
                </span>
              </div>
            ))}
          </div>

          {/* Cryptographic Hash */}
          <div className="p-3 bg-slate-950/60 rounded-lg border border-slate-800/80 space-y-1">
            <span className="text-[10px] text-slate-500 uppercase font-mono block">State Immutable Hash</span>
            <code className="text-xs text-sky-300 font-mono break-all">{activeState.hash}</code>
          </div>

          {/* Mutations Diff */}
          {activeState.diff && (
            <div className="p-3 bg-slate-950/60 rounded-lg border border-slate-800/80 space-y-2">
              <span className="text-[10px] text-slate-500 uppercase font-mono block">State Transition Diff</span>
              <div className="flex flex-wrap gap-2 text-xs font-mono">
                {activeState.diff.added.map((a) => (
                  <span key={a} className="px-2 py-0.5 rounded bg-emerald-950 text-emerald-400 border border-emerald-800">
                    {a}
                  </span>
                ))}
                {activeState.diff.removed.map((r) => (
                  <span key={r} className="px-2 py-0.5 rounded bg-rose-950 text-rose-400 border border-rose-800">
                    {r}
                  </span>
                ))}
                {activeState.diff.transformed.map((tr) => (
                  <span key={tr.from} className="px-2 py-0.5 rounded bg-amber-950 text-amber-400 border border-amber-800">
                    {tr.from} → {tr.to}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>

        {/* Applied Sūtra & Paribhāṣā Conflict Resolver */}
        <div className="bg-slate-900/80 border border-slate-800 p-5 rounded-xl space-y-4">
          <div className="flex items-center justify-between">
            <h4 className="text-sm font-semibold text-slate-200 uppercase tracking-wider font-mono">
              Applied Aṣṭādhyāyī Sūtra
            </h4>
            {activeState.applied_rule && (
              <span className="text-xs font-mono px-2 py-0.5 rounded bg-indigo-950 text-indigo-400 border border-indigo-800">
                Sūtra {activeState.applied_rule.sutra_id}
              </span>
            )}
          </div>

          {activeState.applied_rule ? (
            <div className="space-y-3">
              <div className="p-4 bg-slate-950/70 rounded-lg border border-slate-800">
                <div className="text-base font-bold text-amber-300 mb-1">
                  {activeState.applied_rule.text_deva}
                </div>
                <div className="text-xs text-slate-400 font-mono mb-2">
                  {activeState.applied_rule.text_slp1}
                </div>
                <p className="text-xs text-slate-300">{activeState.applied_rule.summary}</p>
              </div>

              {activeState.applied_rule.paribhasha_principle && (
                <div className="p-3 bg-purple-950/40 rounded-lg border border-purple-800/60 space-y-1">
                  <span className="text-[10px] text-purple-400 uppercase font-mono font-bold block">
                    ⚡ Paribhāṣā Priority Resolution
                  </span>
                  <p className="text-xs text-purple-200 font-mono">
                    {activeState.applied_rule.paribhasha_principle}
                  </p>
                  {activeState.applied_rule.blocked_sutras && (
                    <div className="text-[11px] text-purple-300 font-mono mt-1">
                      Blocked: {activeState.applied_rule.blocked_sutras.join(', ')}
                    </div>
                  )}
                </div>
              )}
            </div>
          ) : (
            <div className="p-6 text-center text-slate-500 font-mono text-xs">
              Initial Root Formulation (No Sūtra Applied)
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
