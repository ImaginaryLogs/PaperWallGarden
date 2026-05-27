import { useCallback, useState, useMemo } from 'react';
import {
  ReactFlow,
  Background,
  Controls,
  MiniMap,
  useNodesState,
  useEdgesState,
  type NodeProps,
  type Node,
  BackgroundVariant,
} from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import katex from 'katex';
import 'katex/dist/katex.min.css';
import { canvasToReactFlow, filePathToSlug, type CanvasData } from '../../lib/obsidian/parseCanvas';

/* ── Markdown renderer (very lightweight) ──────────────────── */
function renderMd(text: string): string {
  return text
    .replace(/^#{1,3} (.+)$/gm, (_, t) => `<strong>${t}</strong>`)
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/\*(.+?)\*/g,     '<em>$1</em>')
    .replace(/`(.+?)`/g,       '<code>$1</code>')
    .replace(/\n/g,            '<br/>');
}

/* ── Custom: text / file / link node ───────────────────────── */
function ObsidianNode({ data }: NodeProps) {
  const [hovered, setHovered] = useState(false);
  const accent = data.accentColor as string | undefined;

  const borderColor = accent ?? 'var(--canvas-node-border)';
  const headerBg    = accent
    ? `color-mix(in srgb, ${accent} 14%, transparent)`
    : 'var(--bg-2)';

  if (data.nodeType === 'file') {
    const file  = data.file as string;
    const slug  = filePathToSlug(file);
    const name  = file.split('/').pop()?.replace(/\.md$/, '') ?? file;
    const folder = file.split('/').slice(0, -1).join('/');
    return (
      <div
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
        style={{
          width: '100%',
          height: '100%',
          background: 'var(--canvas-node-bg)',
          border: `1.5px solid ${hovered ? (accent ?? 'var(--accent)') : borderColor}`,
          borderRadius: 10,
          overflow: 'hidden',
          transition: 'border-color 150ms ease, box-shadow 150ms ease',
          boxShadow: hovered ? `0 0 0 3px color-mix(in srgb, ${accent ?? 'var(--accent)'} 18%, transparent)` : 'none',
          cursor: 'pointer',
          display: 'flex',
          flexDirection: 'column',
        }}
        onClick={() => { window.location.href = `/garden/${slug}`; }}
      >
        <div style={{
          padding: '8px 12px',
          background: headerBg,
          borderBottom: `1px solid ${borderColor}`,
          display: 'flex',
          alignItems: 'center',
          gap: 6,
        }}>
          <span style={{ fontSize: 11, opacity: 0.5, fontFamily: 'var(--font-mono)' }}>📄 {folder}</span>
        </div>
        <div style={{ padding: '10px 14px', flex: 1 }}>
          <div style={{
            fontFamily: 'var(--font-serif)',
            fontSize: 13,
            fontWeight: 500,
            color: 'var(--text)',
            marginBottom: 4,
          }}>
            {name}
          </div>
          <div style={{
            fontFamily: 'var(--font-mono)',
            fontSize: 10,
            color: 'var(--accent)',
            opacity: hovered ? 1 : 0,
            transition: 'opacity 150ms ease',
          }}>
            Open note →
          </div>
        </div>
      </div>
    );
  }

  if (data.nodeType === 'link') {
    const url   = data.url as string;
    const label = data.label as string ?? url;
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          background: 'var(--canvas-node-bg)',
          border: `1.5px solid ${borderColor}`,
          borderRadius: 10,
          padding: '10px 14px',
          display: 'flex',
          flexDirection: 'column',
          gap: 6,
        }}
      >
        <span style={{ fontFamily: 'var(--font-mono)', fontSize: 10, color: 'var(--text-3)' }}>🔗 External</span>
        <a
          href={url}
          target="_blank"
          rel="noopener noreferrer"
          style={{ fontSize: 13, color: 'var(--link)', fontFamily: 'var(--font-serif)' }}
        >
          {label}
        </a>
      </div>
    );
  }

  // Default: text node
  const rawText = (data.text as string) ?? '';
  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        background: 'var(--canvas-node-bg)',
        border: `1.5px solid ${borderColor}`,
        borderRadius: 10,
        padding: '12px 16px',
        overflow: 'hidden',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      {accent && (
        <div style={{
          position: 'absolute',
          top: 0, left: 0, right: 0,
          height: 3,
          background: accent,
          borderRadius: '10px 10px 0 0',
        }} />
      )}
      <div
        style={{
          fontFamily: 'var(--font-serif)',
          fontSize: 13,
          lineHeight: 1.65,
          color: 'var(--text)',
          flex: 1,
          overflow: 'hidden',
        }}
        dangerouslySetInnerHTML={{ __html: renderMd(rawText) }}
      />
    </div>
  );
}

/* ── Group node label ──────────────────────────────────────── */
function GroupNode({ data }: NodeProps) {
  return (
    <div style={{
      width: '100%',
      height: '100%',
      border: '1px dashed var(--border-2)',
      borderRadius: 14,
      position: 'relative',
      pointerEvents: 'none',
    }}>
      {data.label && (
        <div style={{
          position: 'absolute',
          top: -11,
          left: 12,
          background: 'var(--bg)',
          padding: '0 8px',
          fontFamily: 'var(--font-mono)',
          fontSize: 10,
          fontWeight: 500,
          letterSpacing: '0.08em',
          color: 'var(--text-3)',
          textTransform: 'uppercase',
        }}>
          {data.label as string}
        </div>
      )}
    </div>
  );
}

/* ── Upgraded Math & Markdown Live Content Parser ──────────────── */
function renderCanvasMarkdown(text: string): string {
  if (!text) return '';
  
  let processed = text;

  // 1. Process Display/Block Equations: $$ ... $$
  processed = processed.replace(/\$\$([\s\S]+?)\$\$/g, (_, equation) => {
    try {
      return `<div class="canvas-math-block">${katex.renderToString(equation.trim(), { displayMode: true })}</div>`;
    } catch (err) {
      return `<code class="text-red-400">${equation}</code>`;
    }
  });

  // 2. Process Inline Equations: $ ... $
  processed = processed.replace(/\$([^$\n]+?)\$/g, (_, equation) => {
    try {
      return `<span class="canvas-math-inline">${katex.renderToString(equation.trim(), { displayMode: false })}</span>`;
    } catch (err) {
      return `<code class="text-red-400">${equation}</code>`;
    }
  });

  // 3. Fallback standard notation transformations
  return processed
    .replace(/^#{1,3} (.+)$/gm, (_, t) => `<strong class="block text-base border-b border-slate-800 pb-1 mb-1">${t}</strong>`)
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/`(.+?)`/g, '<code class="bg-slate-950 px-1 rounded text-cyan-400">$1</code>')
    .replace(/\n/g, '<br/>');
}

const nodeTypes = {
  obsidianNode: ObsidianNode,
  group:        GroupNode,
};

/* ── Main canvas component ─────────────────────────────────── */
interface Props {
  canvasJson: CanvasData;
  title?: string;
}

export default function ObsidianCanvas({ canvasJson, title }: Props) {
  const { nodes: initNodes, edges: initEdges } = useMemo(
    () => {
      const data = canvasToReactFlow(canvasJson)
      console.log("Canvas Nodes Loaded: ", data.nodes)
      return data;
    },
    [canvasJson]
  );

  const [nodes, , onNodesChange] = useNodesState(initNodes);
  const [edges, , onEdgesChange] = useEdgesState(initEdges);

  return (
    <div style={{
      width: '100%',
      height: '600px',
      background: 'var(--canvas-bg)',
      borderRadius: 12,
      overflow: 'hidden',
      position: 'relative'
    }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        fitViewOptions={{ padding: 0.15, maxZoom: 1.2 }}
        minZoom={0.1}
        maxZoom={3}
        proOptions={{ hideAttribution: true }}
        onInit={(instance) => instance.fitView()}
        style={{ background: 'transparent' }}
      >
        <Background
          variant={BackgroundVariant.Dots}
          gap={24}
          size={1}
          color="var(--border-2)"
        />
        <Controls
          style={{
            background: 'var(--canvas-node-bg)',
            border: '1px solid var(--border)',
            borderRadius: 8,
          }}
        />
        <MiniMap
          style={{
            background: 'var(--bg-2)',
            border: '1px solid var(--border)',
            borderRadius: 8,
          }}
          nodeColor="var(--border-2)"
          maskColor="color-mix(in srgb, var(--bg) 70%, transparent)"
        />
      </ReactFlow>
    </div>
  );
}