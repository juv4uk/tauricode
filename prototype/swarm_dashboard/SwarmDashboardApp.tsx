/**
 * SwarmDashboardApp - Tauricode Desktop Workbench & Swarm Telemetry Master Application
 */

import React, { useState, useEffect } from "react"
import { SwarmMeshTopology, SwarmNode, DerivationTrace, TaskStats } from "./types"
import { MOCK_SWARM_TOPOLOGY, MOCK_TASK_STATS, CANONICAL_DERIVATIONS } from "./fixtures"
import { getSwarmTopology, queryDerivationTrace, listenNodeHeartbeat } from "./tauri_ipc"
import { SwarmMeshGraph } from "./components/SwarmMeshGraph"
import { NodeHealthMatrix } from "./components/NodeHealthMatrix"
import { TaskCompletionTelemetry } from "./components/TaskCompletionTelemetry"
import { DerivationDagStreamer } from "./components/DerivationDagStreamer"
import { PhoneticInspector } from "./components/PhoneticInspector"

type ActiveTab = "topology" | "nodes" | "tasks" | "derivation" | "phonetics" | "ipc"

export const SwarmDashboardApp: React.FC = () => {
  const [activeTab, setActiveTab] = useState<ActiveTab>("topology")
  const [topology, setTopology] = useState<SwarmMeshTopology>(MOCK_SWARM_TOPOLOGY)
  const [taskStats, setTaskStats] = useState<TaskStats>(MOCK_TASK_STATS)
  const [selectedNode, setSelectedNode] = useState<SwarmNode | null>(topology.nodes[0] || null)
  const [activeDerivationId, setActiveDerivationId] = useState<string>("bhavati")
  const [derivationTrace, setDerivationTrace] = useState<DerivationTrace>(CANONICAL_DERIVATIONS["bhavati"])
  const [ipcLogs, setIpcLogs] = useState<Array<{ timestamp: string; command: string; payload: string }>>([])

  // Fetch initial topology and listen to heartbeats
  useEffect(() => {
    let isMounted = true

    const fetchTopology = async () => {
      try {
        const top = await getSwarmTopology()
        if (isMounted) {
          setTopology(top)
          logIpc("get_swarm_topology", `Loaded ${top.nodes.length} nodes, ${top.totalTasksCompleted} tasks`)
        }
      } catch (e) {
        console.error("Failed to load topology:", e)
      }
    }

    fetchTopology()

    const unsubscribe = listenNodeHeartbeat((nodeUpdate) => {
      if (!isMounted) return
      setTopology((prev) => ({
        ...prev,
        nodes: prev.nodes.map((n) => (n.id === nodeUpdate.id ? nodeUpdate : n)),
      }))
      logIpc("telemetry://node-heartbeat", `Node ${nodeUpdate.name} heartbeat (${nodeUpdate.latencyMs}ms)`)
    })

    return () => {
      isMounted = false
      unsubscribe()
    }
  }, [])

  // Fetch Derivation Trace on selection change
  useEffect(() => {
    let isMounted = true
    queryDerivationTrace(activeDerivationId).then((tr) => {
      if (isMounted) {
        setDerivationTrace(tr)
        logIpc("query_derivation_trace", `Retrieved ${tr.target_word} (${tr.states.length} states)`)
      }
    })
    return () => {
      isMounted = false
    }
  }, [activeDerivationId])

  const logIpc = (command: string, payload: string) => {
    setIpcLogs((prev) => [{ timestamp: new Date().toLocaleTimeString(), command, payload }, ...prev.slice(0, 49)])
  }

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col font-sans selection:bg-sky-500 selection:text-white">
      {/* Top Navigation Bar */}
      <header className="sticky top-0 z-30 bg-slate-900/90 border-b border-slate-800 backdrop-blur px-6 py-3 flex flex-wrap items-center justify-between gap-4">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-gradient-to-tr from-sky-500 to-indigo-600 flex items-center justify-center font-bold text-white shadow-md shadow-sky-500/20">
            τ
          </div>
          <div>
            <h1 className="text-base font-bold tracking-wide text-slate-100 flex items-center gap-2">
              TauriCode Workbench
              <span className="text-[10px] px-2 py-0.5 rounded bg-sky-950 text-sky-400 border border-sky-800 font-mono">
                Tauri v2 IPC
              </span>
            </h1>
            <p className="text-xs text-slate-400">Swarm Mesh Telemetry (:9101 - :9107)</p>
          </div>
        </div>

        {/* Tab Buttons */}
        <nav className="flex items-center gap-1 bg-slate-950/60 p-1 rounded-lg border border-slate-800 text-xs font-mono">
          <button
            onClick={() => setActiveTab("topology")}
            className={`px-3 py-1.5 rounded transition ${
              activeTab === "topology" ? "bg-sky-600 text-white font-bold" : "text-slate-400 hover:text-slate-200"
            }`}
          >
            🌐 Mesh Topology
          </button>
          <button
            onClick={() => setActiveTab("nodes")}
            className={`px-3 py-1.5 rounded transition ${
              activeTab === "nodes" ? "bg-sky-600 text-white font-bold" : "text-slate-400 hover:text-slate-200"
            }`}
          >
            📡 Nodes (:9101-:9107)
          </button>
          <button
            onClick={() => setActiveTab("tasks")}
            className={`px-3 py-1.5 rounded transition ${
              activeTab === "tasks" ? "bg-sky-600 text-white font-bold" : "text-slate-400 hover:text-slate-200"
            }`}
          >
            📊 Tasks ({topology.totalTasksCompleted})
          </button>
          <button
            onClick={() => setActiveTab("derivation")}
            className={`px-3 py-1.5 rounded transition ${
              activeTab === "derivation" ? "bg-sky-600 text-white font-bold" : "text-slate-400 hover:text-slate-200"
            }`}
          >
            🌿 Derivation DAG
          </button>
          <button
            onClick={() => setActiveTab("phonetics")}
            className={`px-3 py-1.5 rounded transition ${
              activeTab === "phonetics" ? "bg-sky-600 text-white font-bold" : "text-slate-400 hover:text-slate-200"
            }`}
          >
            🔊 Phonetics Lab
          </button>
          <button
            onClick={() => setActiveTab("ipc")}
            className={`px-3 py-1.5 rounded transition ${
              activeTab === "ipc" ? "bg-sky-600 text-white font-bold" : "text-slate-400 hover:text-slate-200"
            }`}
          >
            ⚡ IPC Console
          </button>
        </nav>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 p-6 max-w-7xl w-full mx-auto space-y-6">
        {activeTab === "topology" && (
          <div className="space-y-6">
            <SwarmMeshGraph
              topology={topology}
              selectedNodeId={selectedNode?.id || null}
              onSelectNode={(node) => {
                setSelectedNode(node)
                setActiveTab("nodes")
              }}
            />
            <NodeHealthMatrix
              nodes={topology.nodes}
              selectedNodeId={selectedNode?.id || null}
              onSelectNode={(node) => setSelectedNode(node)}
            />
          </div>
        )}

        {activeTab === "nodes" && (
          <NodeHealthMatrix
            nodes={topology.nodes}
            selectedNodeId={selectedNode?.id || null}
            onSelectNode={(node) => setSelectedNode(node)}
          />
        )}

        {activeTab === "tasks" && <TaskCompletionTelemetry stats={taskStats} />}

        {activeTab === "derivation" && (
          <DerivationDagStreamer trace={derivationTrace} onSelectDerivation={(id) => setActiveDerivationId(id)} />
        )}

        {activeTab === "phonetics" && <PhoneticInspector />}

        {activeTab === "ipc" && (
          <div className="bg-slate-900/80 border border-slate-800 p-5 rounded-xl space-y-4">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-semibold text-slate-200 uppercase tracking-wider font-mono">
                Tauri v2 IPC Stream & Command Audit Log
              </h3>
              <span className="text-xs font-mono text-emerald-400">IPC Bridge Active</span>
            </div>
            <div className="bg-slate-950 p-4 rounded-lg border border-slate-800/80 font-mono text-xs max-h-96 overflow-y-auto space-y-2">
              {ipcLogs.map((log, idx) => (
                <div key={idx} className="flex items-start gap-3 text-slate-300">
                  <span className="text-slate-500">{log.timestamp}</span>
                  <span className="text-sky-400 font-bold">{log.command}</span>
                  <span className="text-slate-400 break-all">{log.payload}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </main>

      {/* Footer */}
      <footer className="bg-slate-900/60 border-t border-slate-800 px-6 py-3 text-xs font-mono text-slate-500 flex items-center justify-between">
        <span>TauriCode Workbench · Swarm Mesh :9101-:9107 · P5 Gate Readiness</span>
        <span>Epistemic Compliance: ECA-007</span>
      </footer>
    </div>
  )
}
