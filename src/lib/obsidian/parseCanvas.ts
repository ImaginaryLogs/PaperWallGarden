import type { Node, Edge } from '@xyflow/react';

export interface CanvasNode {
  id: string;
  type: 'text' | 'file' | 'link' | 'group';
  text?: string;
  file?: string;
  url?: string;
  label?: string;
  x: number;
  y: number;
  width: number;
  height: number;
  color?: string;
}

export interface CanvasEdge {
  id: string;
  fromNode: string;
  toNode: string;
  fromSide?: 'top' | 'bottom' | 'left' | 'right';
  toSide?: 'top' | 'bottom' | 'left' | 'right';
  label?: string;
  color?: string;
}

export interface CanvasData {
  nodes: CanvasNode[];
  edges: CanvasEdge[];
}

// Obsidian color index → CSS variable name
const COLOR_MAP: Record<string, string> = {
  '1': 'var(--canvas-red)',
  '2': 'var(--canvas-orange)',
  '3': 'var(--canvas-yellow)',
  '4': 'var(--canvas-green)',
  '5': 'var(--canvas-teal)',
  '6': 'var(--canvas-purple)',
};

export function canvasToReactFlow(canvas: CanvasData): {
  nodes: Node[];
  edges: Edge[];
} {
  const nodes: Node[] = canvas.nodes.map((n) => {
    const accentColor = n.color ? COLOR_MAP[n.color] ?? n.color : undefined;

    if (n.type === 'group') {
      return {
        id: n.id,
        type: 'group',
        position: { x: n.x, y: n.y },
        style: {
          width: n.width,
          height: n.height,
        },
        data: { label: n.label ?? '' },
        className: 'canvas-group-node',
      };
    }

    return {
      id: n.id,
      type: 'obsidianNode',
      position: { x: n.x, y: n.y },
      style: { width: n.width, minHeight: n.height },
      data: {
        nodeType: n.type,
        text:     n.text,
        file:     n.file,
        url:      n.url,
        label:    n.label,
        accentColor,
      },
    };
  });

  const edges: Edge[] = canvas.edges.map((e) => ({
    id:           e.id,
    source:       e.fromNode,
    target:       e.toNode,
    sourceHandle: e.fromSide,
    targetHandle: e.toSide,
    label:        e.label,
    type:         'smoothstep',
    animated:     false,
    style:        { stroke: 'var(--canvas-edge)', strokeWidth: 1.5 },
    labelStyle:   { fill: 'var(--canvas-edge-label)', fontSize: 11 },
    labelBgStyle: { fill: 'var(--canvas-label-bg)' },
  }));

  return { nodes, edges };
}

export function filePathToSlug(filePath: string): string {
  return filePath
    .replace(/\.md$/, '')
    .replace(/\\/g, '/')
    .split('/')
    .pop()!
    .toLowerCase()
    .replace(/\s+/g, '-');
}