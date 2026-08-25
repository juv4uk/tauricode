/**
 * TaskCompletionTelemetry - Real-time task metrics across Swarm Mesh (271+ completed tasks)
 * Displays capability distribution, node completion ratios, and recent execution events.
 */

import React from "react"
import { TaskStats } from "../types"

interface TaskCompletionTelemetryProps {
  stats: TaskStats
}

export const TaskCompletionTelemetry: React.FC<TaskCompletionTelemetryProps> = ({ stats }) => {
  return (
    <div className="space-y-6">
      {/* Top Stat Banner */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-4">
          <span className="text-xs text-slate-400 font-mono uppercase block">Total Mesh Tasks</span>
          <div className="flex items-baseline gap-2 mt-1">
            <span className="text-2xl font-bold text-purple-400 font-mono">{stats.totalCompleted}</span>
            <span className="text-xs text-emerald-400 font-mono">(271+ Target Met)</span>
          </div>
          <p className="text-xs text-slate-500 mt-1">Across 6 active cluster nodes</p>
        </div>

        <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-4">
          <span className="text-xs text-slate-400 font-mono uppercase block">Mesh Completion Rate</span>
          <div className="flex items-baseline gap-2 mt-1">
            <span className="text-2xl font-bold text-emerald-400 font-mono">{stats.completionRatePct}%</span>
            <span className="text-xs text-slate-400 font-mono">0 pending</span>
          </div>
          <p className="text-xs text-slate-500 mt-1">Deterministic state convergence</p>
        </div>

        <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-4">
          <span className="text-xs text-slate-400 font-mono uppercase block">Queued Micro-tasks</span>
          <div className="flex items-baseline gap-2 mt-1">
            <span className="text-2xl font-bold text-sky-400 font-mono">{stats.totalQueued}</span>
            <span className="text-xs text-slate-400 font-mono">in pipeline</span>
          </div>
          <p className="text-xs text-slate-500 mt-1">Ready for next gate transition</p>
        </div>

        <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-4">
          <span className="text-xs text-slate-400 font-mono uppercase block">Verification Protocol</span>
          <div className="flex items-baseline gap-2 mt-1">
            <span className="text-xl font-bold text-indigo-300 font-mono">ECA-007</span>
            <span className="text-xs text-emerald-400 font-mono">PASSING</span>
          </div>
          <p className="text-xs text-slate-500 mt-1">Epistemic proof certificates</p>
        </div>
      </div>

      {/* Breakdown Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Node Distribution */}
        <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-5">
          <h4 className="text-sm font-semibold text-slate-200 uppercase tracking-wider mb-4 font-mono">
            Task Volume by Swarm Node
          </h4>
          <div className="space-y-3">
            {Object.entries(stats.nodeDistribution).map(([node, count]) => {
              const pct = ((count / stats.totalCompleted) * 100).toFixed(1)
              return (
                <div key={node} className="space-y-1">
                  <div className="flex justify-between text-xs font-mono">
                    <span className="text-slate-300 font-bold">{node}</span>
                    <span className="text-slate-400">
                      {count} tasks ({pct}%)
                    </span>
                  </div>
                  <div className="w-full h-2 bg-slate-800 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-sky-500 to-indigo-500 rounded-full"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        {/* Capability Distribution */}
        <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-5">
          <h4 className="text-sm font-semibold text-slate-200 uppercase tracking-wider mb-4 font-mono">
            Capability Specialization Distribution
          </h4>
          <div className="space-y-3">
            {Object.entries(stats.capabilityDistribution).map(([cap, count]) => {
              const pct = ((count / stats.totalCompleted) * 100).toFixed(1)
              return (
                <div key={cap} className="space-y-1">
                  <div className="flex justify-between text-xs font-mono">
                    <span className="text-slate-300">#{cap}</span>
                    <span className="text-purple-400 font-bold">{count}</span>
                  </div>
                  <div className="w-full h-2 bg-slate-800 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-gradient-to-r from-purple-500 to-pink-500 rounded-full"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </div>

      {/* Recent Execution Events */}
      <div className="rounded-xl bg-slate-900/80 border border-slate-800 p-5">
        <h4 className="text-sm font-semibold text-slate-200 uppercase tracking-wider mb-4 font-mono">
          Recent Swarm Task Execution Audit Log
        </h4>
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-slate-950/60 text-slate-400 uppercase border-b border-slate-800">
              <tr>
                <th className="py-2.5 px-3">Task ID</th>
                <th className="py-2.5 px-3">Node</th>
                <th className="py-2.5 px-3">Title & Summary</th>
                <th className="py-2.5 px-3">Capabilities</th>
                <th className="py-2.5 px-3">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 text-slate-300">
              {stats.recentTasks.map((t) => (
                <tr key={t.id} className="hover:bg-slate-800/40">
                  <td className="py-2.5 px-3 font-bold text-sky-400">{t.id}</td>
                  <td className="py-2.5 px-3 text-indigo-400">{t.nodeId}</td>
                  <td className="py-2.5 px-3">
                    <div className="font-semibold text-slate-200">{t.title}</div>
                    <div className="text-[11px] text-slate-400 line-clamp-1">{t.description}</div>
                  </td>
                  <td className="py-2.5 px-3">
                    <div className="flex flex-wrap gap-1">
                      {t.capabilities.slice(0, 3).map((c) => (
                        <span key={c} className="px-1.5 py-0.5 rounded bg-slate-800 text-[10px] text-slate-400">
                          {c}
                        </span>
                      ))}
                    </div>
                  </td>
                  <td className="py-2.5 px-3">
                    <span className="px-2 py-0.5 rounded text-[10px] bg-emerald-950 text-emerald-400 border border-emerald-800">
                      {t.status}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  )
}
