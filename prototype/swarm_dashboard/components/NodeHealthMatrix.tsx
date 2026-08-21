/**
 * NodeHealthMatrix - Detailed Grid & Metric Cards for all 6 Swarm Nodes
 * Displays node health, role, port (:9101-:9107), completed tasks, CPU, memory, and capabilities.
 */

import React from 'react';
import { SwarmNode } from '../types';

interface NodeHealthMatrixProps {
  nodes: SwarmNode[];
  selectedNodeId: string | null;
  onSelectNode: (node: SwarmNode) => void;
}

export const NodeHealthMatrix: React.FC<NodeHealthMatrixProps> = ({
  nodes,
  selectedNodeId,
  onSelectNode
}) => {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {nodes.map((node) => {
        const isSelected = selectedNodeId === node.id;
        return (
          <div
            key={node.id}
            onClick={() => onSelectNode(node)}
            className={`cursor-pointer rounded-xl p-4 transition-all duration-200 border ${
              isSelected
                ? 'bg-slate-800/90 border-sky-500 shadow-lg shadow-sky-500/10 ring-1 ring-sky-500'
                : 'bg-slate-900/70 border-slate-800 hover:border-slate-700 hover:bg-slate-800/50'
            }`}
          >
            {/* Header */}
            <div className="flex items-start justify-between">
              <div>
                <div className="flex items-center gap-2">
                  <span
                    className={`w-2.5 h-2.5 rounded-full ${
                      node.status === 'ONLINE' ? 'bg-emerald-400 shadow-sm shadow-emerald-400' : 'bg-amber-400'
                    }`}
                  />
                  <h4 className="font-bold text-slate-100 text-base">{node.name}</h4>
                  <span className="text-xs px-2 py-0.5 rounded bg-sky-950 text-sky-400 border border-sky-800 font-mono">
                    :{node.port}
                  </span>
                </div>
                <p className="text-xs text-slate-400 mt-1 line-clamp-1">{node.role}</p>
              </div>
              <span className="text-xs font-mono px-2 py-0.5 rounded bg-slate-800 text-slate-300">
                {node.latencyMs} ms
              </span>
            </div>

            {/* Epistemic Layer & Repo */}
            <div className="mt-3 text-xs text-slate-400 flex items-center justify-between border-t border-slate-800/80 pt-2 font-mono">
              <span className="text-slate-500 truncate max-w-[180px]">{node.layer}</span>
              <span className="text-indigo-400">repo: {node.repo}</span>
            </div>

            {/* Telemetry Bars */}
            <div className="mt-3 space-y-2">
              <div>
                <div className="flex justify-between text-xs font-mono text-slate-400 mb-1">
                  <span>Tasks Completed</span>
                  <span className="text-purple-400 font-bold">{node.completedTasks}/{node.totalTasks}</span>
                </div>
                <div className="w-full h-1.5 bg-slate-800 rounded-full overflow-hidden">
                  <div
                    className="h-full bg-gradient-to-r from-purple-500 to-indigo-500 rounded-full"
                    style={{ width: `${(node.completedTasks / Math.max(1, node.totalTasks)) * 100}%` }}
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-2 pt-1">
                <div className="bg-slate-950/60 rounded p-1.5 text-center">
                  <span className="text-[10px] text-slate-500 uppercase block">CPU</span>
                  <span className="text-xs font-mono font-bold text-sky-400">{node.cpuUsagePct}%</span>
                </div>
                <div className="bg-slate-950/60 rounded p-1.5 text-center">
                  <span className="text-[10px] text-slate-500 uppercase block">Memory</span>
                  <span className="text-xs font-mono font-bold text-slate-200">{node.memoryMb} MB</span>
                </div>
              </div>
            </div>

            {/* Capabilities */}
            <div className="mt-3 flex flex-wrap gap-1">
              {node.capabilities.slice(0, 4).map((cap) => (
                <span
                  key={cap}
                  className="text-[10px] px-1.5 py-0.5 rounded bg-slate-800 text-slate-300 font-mono"
                >
                  #{cap}
                </span>
              ))}
              {node.capabilities.length > 4 && (
                <span className="text-[10px] px-1 py-0.5 text-slate-500 font-mono">
                  +{node.capabilities.length - 4}
                </span>
              )}
            </div>
          </div>
        );
      })}
    </div>
  );
};
