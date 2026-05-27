// src/components/garden/ObsidianCanvas.tsx
'use client'; // Astro Island — only hydrates this component
import { useCallback, useState } from 'react';
import {
  ReactFlow, Background, Controls, MiniMap,
  type Node, type NodeProps,
} from '@xyflow/react';
import { marked } from 'marked';
import { canvasToReactFlow } from '../../lib/obsidian/parseCanvas';
import type { CanvasData } from '../../lib/obsidian/parseCanvas';

// Custom node that renders Obsidian markdown text nodes
function ObsidianNode({ data }: NodeProps) {
  const [expanded, setExpanded] = useState(false);

  if (data.type === 'file') {
    return (
      <div className="canvas-node canvas-node--file">
        <div className="canvas-node__header">📄 {data.file}</div>
        {/* Link resolves to the ODG page for that file */}
        <a href={`/garden/${slugifyPath(data.file as string)}`} className="canvas-node__link">
          Open note →
        </a>
      </div>
    );
  }

  return (
    <div className="canvas-node canvas-node--text" onClick={() => setExpanded(!expanded)}>
      <div
        className="canvas-node__content"
        dangerouslySetInnerHTML={{ __html: marked(data.label as string) }}
      />
    </div>
  );
}

const nodeTypes = { obsidianNode: ObsidianNode };

interface Props {
  canvasJson: CanvasData;
  title?: string;
}

export default function ObsidianCanvas({ canvasJson, title }: Props) {
  const { nodes, edges } = canvasToReactFlow(canvasJson);

  return (
    <div className="obsidian-canvas" style={{ height: '70vh', width: '100%' }}>
      {title && <h2 className="canvas-title">{title}</h2>}
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        fitView
        attributionPosition="bottom-right"
      >
        <Background />
        <Controls />
        <MiniMap />
      </ReactFlow>
    </div>
  );
}

function slugifyPath(filePath: string): string {
  return filePath.replace(/\.md$/, '').replace(/\s+/g, '-').toLowerCase();
}