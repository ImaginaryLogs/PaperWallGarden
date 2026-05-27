// src/lib/obsidian/parseCanvas.ts
import type { Node, Edge } from '@xyflow/react';

export interface CanvasData {
  nodes: ObsidianCanvasNode[];
  edges: ObsidianCanvasEdge[];
}

export function canvasToReactFlow(canvas: CanvasData): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = canvas.nodes.map((n) => ({
    id: n.id,
    position: { x: n.x, y: n.y },
    style: { width: n.width, height: n.height },
    data: {
      label: n.text || n.file || n.url || n.label || '',
      type: n.type,
      file: n.file,
      color: n.color,
    },
    type: n.type === 'group' ? 'group' : 'obsidianNode',
    // Groups use React Flow's built-in group node
    ...(n.type === 'group' && {
      style: { width: n.width, height: n.height, backgroundColor: 'rgba(200,200,200,0.1)' }
    }),
  }));

  const edges: Edge[] = canvas.edges.map((e) => ({
    id: e.id,
    source: e.fromNode,
    target: e.toNode,
    sourceHandle: e.fromSide,  // 'top'|'bottom'|'left'|'right'
    targetHandle: e.toSide,
    label: e.label,
    type: 'smoothstep',
  }));

  return { nodes, edges };
}