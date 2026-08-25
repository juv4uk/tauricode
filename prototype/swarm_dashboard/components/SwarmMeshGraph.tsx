/**
 * SwarmMeshGraph - Dynamic Animated Canvas & SVG Graph Component
 * Visualizes the 6 P2P Swarm Nodes (:9101-:9107) with pulse telemetry and connection health.
 */

import React, { useEffect, useRef, useState } from "react"
import { SwarmMeshTopology, SwarmNode } from "../types"

interface SwarmMeshGraphProps {
  topology: SwarmMeshTopology
  selectedNodeId: string | null
  onSelectNode: (node: SwarmNode) => void
}

export const SwarmMeshGraph: React.FC<SwarmMeshGraphProps> = ({ topology, selectedNodeId, onSelectNode }) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null)
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null)

  // Fixed angular layout for the 6 nodes in a hexagon
  const nodePositions = React.useMemo(() => {
    const map = new Map<string, { x: number; y: number; node: SwarmNode }>()
    const count = topology.nodes.length
    const centerX = 360
    const centerY = 240
    const radius = 160

    topology.nodes.forEach((node, idx) => {
      const angle = (idx / count) * 2 * Math.PI - Math.PI / 2
      const x = centerX + radius * Math.cos(angle)
      const y = centerY + radius * Math.sin(angle)
      map.set(node.name, { x, y, node })
    })
    return map
  }, [topology.nodes])

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas) return
    const ctx = canvas.getContext("2d")
    if (!ctx) return

    let animFrameId: number
    let tick = 0

    const render = () => {
      tick += 0.03
      ctx.clearRect(0, 0, canvas.width, canvas.height)

      // 1. Draw Mesh Background Grid & Radial Glow
      const grad = ctx.createRadialGradient(360, 240, 20, 360, 240, 260)
      grad.addColorStop(0, "rgba(56, 189, 248, 0.08)")
      grad.addColorStop(0.6, "rgba(129, 140, 248, 0.03)")
      grad.addColorStop(1, "rgba(15, 23, 42, 0)")
      ctx.fillStyle = grad
      ctx.fillRect(0, 0, canvas.width, canvas.height)

      // Central Hub / Ring
      ctx.beginPath()
      ctx.arc(360, 240, 60, 0, Math.PI * 2)
      ctx.strokeStyle = "rgba(56, 189, 248, 0.15)"
      ctx.lineWidth = 1.5
      ctx.setLineDash([4, 4])
      ctx.stroke()
      ctx.setLineDash([])

      ctx.font = "11px monospace"
      ctx.fillStyle = "rgba(148, 163, 184, 0.7)"
      ctx.textAlign = "center"
      ctx.fillText("SWARM MESH", 360, 236)
      ctx.fillStyle = "#38bdf8"
      ctx.font = "bold 12px monospace"
      ctx.fillText(":9101 - :9107", 360, 252)

      // 2. Draw Connections & Telemetry Pulses
      topology.connections.forEach((conn, cIdx) => {
        const srcPos = nodePositions.get(conn.source)
        const tgtPos = nodePositions.get(conn.target)
        if (!srcPos || !tgtPos) return

        const isHighlighted =
          selectedNodeId === srcPos.node.id ||
          selectedNodeId === tgtPos.node.id ||
          hoveredNodeId === srcPos.node.id ||
          hoveredNodeId === tgtPos.node.id

        // Line style
        ctx.beginPath()
        ctx.moveTo(srcPos.x, srcPos.y)
        ctx.lineTo(tgtPos.x, tgtPos.y)
        ctx.strokeStyle = isHighlighted
          ? "rgba(56, 189, 248, 0.7)"
          : conn.protocol === "IPC"
            ? "rgba(168, 85, 247, 0.25)"
            : "rgba(56, 189, 248, 0.2)"
        ctx.lineWidth = isHighlighted ? 2.5 : 1.2
        ctx.stroke()

        // Traveling Pulse Packet
        const pulseT = (tick * 0.8 + cIdx * 0.3) % 1
        const px = srcPos.x + (tgtPos.x - srcPos.x) * pulseT
        const py = srcPos.y + (tgtPos.y - srcPos.y) * pulseT

        ctx.beginPath()
        ctx.arc(px, py, isHighlighted ? 3.5 : 2.2, 0, Math.PI * 2)
        ctx.fillStyle = conn.protocol === "IPC" ? "#c084fc" : "#38bdf8"
        ctx.shadowColor = conn.protocol === "IPC" ? "#a855f7" : "#38bdf8"
        ctx.shadowBlur = 6
        ctx.fill()
        ctx.shadowBlur = 0
      })

      // 3. Draw Nodes
      nodePositions.forEach(({ x, y, node }) => {
        const isSelected = selectedNodeId === node.id
        const isHovered = hoveredNodeId === node.id

        // Halo Ring
        ctx.beginPath()
        ctx.arc(x, y, isSelected ? 34 : 28, 0, Math.PI * 2)
        ctx.fillStyle = isSelected
          ? "rgba(56, 189, 248, 0.2)"
          : isHovered
            ? "rgba(56, 189, 248, 0.12)"
            : "rgba(30, 41, 59, 0.8)"
        ctx.strokeStyle = isSelected ? "#38bdf8" : isHovered ? "#7dd3fc" : "#334155"
        ctx.lineWidth = isSelected ? 2.5 : 1.5
        ctx.fill()
        ctx.stroke()

        // Status Core
        ctx.beginPath()
        ctx.arc(x, y, 6, 0, Math.PI * 2)
        ctx.fillStyle = node.status === "ONLINE" ? "#22c55e" : "#eab308"
        ctx.shadowColor = node.status === "ONLINE" ? "#22c55e" : "#eab308"
        ctx.shadowBlur = 8
        ctx.fill()
        ctx.shadowBlur = 0

        // Node Label
        ctx.textAlign = "center"
        ctx.fillStyle = isSelected ? "#ffffff" : "#e2e8f0"
        ctx.font = "bold 12px sans-serif"
        ctx.fillText(node.name, x, y - 36)

        // Port Badge
        ctx.fillStyle = "#38bdf8"
        ctx.font = "10px monospace"
        ctx.fillText(`:${node.port} · ${node.latencyMs}ms`, x, y + 44)
      })

      animFrameId = requestAnimationFrame(render)
    }

    render()

    return () => cancelAnimationFrame(animFrameId)
  }, [topology, selectedNodeId, hoveredNodeId, nodePositions])

  const handleCanvasClick = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = canvasRef.current?.getBoundingClientRect()
    if (!rect) return
    const clickX = e.clientX - rect.left
    const clickY = e.clientY - rect.top

    nodePositions.forEach(({ x, y, node }) => {
      const dist = Math.hypot(clickX - x, clickY - y)
      if (dist <= 35) {
        onSelectNode(node)
      }
    })
  }

  const handleMouseMove = (e: React.MouseEvent<HTMLCanvasElement>) => {
    const rect = canvasRef.current?.getBoundingClientRect()
    if (!rect) return
    const mx = e.clientX - rect.left
    const my = e.clientY - rect.top

    let found: string | null = null
    nodePositions.forEach(({ x, y, node }) => {
      const dist = Math.hypot(mx - x, my - y)
      if (dist <= 35) {
        found = node.id
      }
    })
    setHoveredNodeId(found)
  }

  return (
    <div className="relative w-full rounded-xl bg-slate-900/80 border border-slate-800 p-4 shadow-xl backdrop-blur">
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <span className="h-3 w-3 rounded-full bg-emerald-500 animate-ping inline-block" />
          <h3 className="text-sm font-semibold text-slate-200 uppercase tracking-wider">
            Swarm Mesh Network Topology (P2P :9101 - :9107)
          </h3>
        </div>
        <div className="flex items-center gap-4 text-xs font-mono text-slate-400">
          <span>
            Active Nodes: <strong className="text-emerald-400">{topology.activeNodesCount}/6</strong>
          </span>
          <span>
            Mesh Health: <strong className="text-sky-400">{topology.meshHealthPct}%</strong>
          </span>
          <span>
            Tasks: <strong className="text-purple-400">{topology.totalTasksCompleted}</strong>
          </span>
        </div>
      </div>

      <div className="flex justify-center">
        <canvas
          ref={canvasRef}
          width={720}
          height={480}
          onClick={handleCanvasClick}
          onMouseMove={handleMouseMove}
          className="cursor-pointer max-w-full h-auto rounded-lg"
        />
      </div>

      <div className="mt-3 flex items-center justify-center gap-6 text-xs text-slate-400">
        <span className="flex items-center gap-1.5">
          <span className="w-2.5 h-2.5 rounded-full bg-emerald-500 inline-block" /> Online
        </span>
        <span className="flex items-center gap-1.5">
          <span className="w-3 h-0.5 bg-sky-400 inline-block" /> TCP Peer Link
        </span>
        <span className="flex items-center gap-1.5">
          <span className="w-3 h-0.5 bg-purple-400 inline-block" /> High-Speed IPC
        </span>
        <span className="text-slate-500">Click any node to view detailed telemetry</span>
      </div>
    </div>
  )
}
