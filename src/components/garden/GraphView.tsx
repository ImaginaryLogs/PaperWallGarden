import { useEffect, useRef, useState } from 'react';
import * as d3 from 'd3';

export interface GraphNode {
  id:    string;
  title: string;
  tags:  string[];
  slug:  string;
}

export interface GraphLink {
  source: string;
  target: string;
}

interface Props {
  nodes: GraphNode[];
  links: GraphLink[];
  currentSlug?: string;
  height?: number;
}

const TAG_COLORS: Record<string, string> = {
  quantum:            '#74c69d',
  ml:                 '#9b59b6',
  'deep-learning':    '#9b59b6',
  'quant-fi':         '#f39c12',
  automata:           '#e74c3c',
  fundamentals:       '#52b788',
  visualization:      '#3498db',
};

function tagColor(tags: string[]): string {
  for (const t of tags) {
    if (TAG_COLORS[t]) return TAG_COLORS[t];
  }
  return 'var(--text-3)';
}

export default function GraphView({ nodes, links, currentSlug, height = 500 }: Props) {
  const svgRef   = useRef<SVGSVGElement>(null);
  const wrapRef  = useRef<HTMLDivElement>(null);
  const [hovered, setHovered] = useState<GraphNode | null>(null);

  useEffect(() => {
    if (!svgRef.current || !wrapRef.current) return;
    const svg = d3.select(svgRef.current);
    svg.selectAll('*').remove();

    const W = wrapRef.current.clientWidth || 700;
    const H = height;
    svg.attr('viewBox', `0 0 ${W} ${H}`);

    const gRoot = svg.append('g');

    // ── Zoom behaviour ──────────────────────────────────────
    svg.call(
      d3.zoom<SVGSVGElement, unknown>()
        .scaleExtent([0.2, 4])
        .on('zoom', (event) => gRoot.attr('transform', event.transform))
    );

    // ── Simulation ──────────────────────────────────────────
    const nodeData = nodes.map((n) => ({ ...n })) as (GraphNode & d3.SimulationNodeDatum)[];
    const linkData = links
      .map((l) => ({
        source: nodeData.find((n) => n.id === l.source || n.slug === l.source),
        target: nodeData.find((n) => n.id === l.target || n.slug === l.target),
      }))
      .filter((l) => l.source && l.target) as d3.SimulationLinkDatum<typeof nodeData[0]>[];

    const sim = d3.forceSimulation(nodeData)
      .force('link',   d3.forceLink(linkData).id((d: any) => d.id).distance(90).strength(0.6))
      .force('charge', d3.forceManyBody().strength(-220))
      .force('center', d3.forceCenter(W / 2, H / 2))
      .force('collide', d3.forceCollide(22));

    // ── Edges ───────────────────────────────────────────────
    const edgeG = gRoot.append('g').attr('class', 'edges');
    const edge  = edgeG.selectAll('line')
      .data(linkData)
      .join('line')
      .attr('stroke', 'var(--border-2)')
      .attr('stroke-width', 1)
      .attr('stroke-opacity', 0.7);

    // ── Nodes ───────────────────────────────────────────────
    const nodeG = gRoot.append('g').attr('class', 'nodes');
    const node  = nodeG.selectAll('g.node-group')
      .data(nodeData)
      .join('g')
      .attr('class', 'node-group')
      .style('cursor', 'pointer')
      .call(
        d3.drag<SVGGElement, typeof nodeData[0]>()
          .on('start', (event, d) => { if (!event.active) sim.alphaTarget(0.3).restart(); d.fx = d.x; d.fy = d.y; })
          .on('drag',  (event, d) => { d.fx = event.x; d.fy = event.y; })
          .on('end',   (event, d) => { if (!event.active) sim.alphaTarget(0); d.fx = null; d.fy = null; })
      )
      .on('click', (_, d) => { window.location.href = `/garden/${d.slug}`; })
      .on('mouseenter', (_, d) => setHovered(d))
      .on('mouseleave', () => setHovered(null));

    // glow for current node
    const defs = svg.append('defs');
    defs.append('filter').attr('id', 'glow')
      .append('feDropShadow')
      .attr('dx', 0).attr('dy', 0)
      .attr('stdDeviation', 4)
      .attr('flood-color', 'var(--accent)')
      .attr('flood-opacity', 0.7);

    node.append('circle')
      .attr('r', (d) => d.slug === currentSlug ? 10 : 7)
      .attr('fill', (d) => tagColor(d.tags))
      .attr('stroke', (d) => d.slug === currentSlug ? 'var(--accent)' : 'var(--bg)')
      .attr('stroke-width', (d) => d.slug === currentSlug ? 2.5 : 1.5)
      .attr('filter', (d) => d.slug === currentSlug ? 'url(#glow)' : null);

    node.append('text')
      .text((d) => d.title.length > 20 ? d.title.slice(0, 18) + '…' : d.title)
      .attr('font-family', 'IBM Plex Mono, monospace')
      .attr('font-size', 10)
      .attr('fill', 'var(--text-2)')
      .attr('text-anchor', 'middle')
      .attr('dy', 20)
      .attr('pointer-events', 'none');

    // ── Tick ────────────────────────────────────────────────
    sim.on('tick', () => {
      edge
        .attr('x1', (d: any) => d.source.x)
        .attr('y1', (d: any) => d.source.y)
        .attr('x2', (d: any) => d.target.x)
        .attr('y2', (d: any) => d.target.y);

      node.attr('transform', (d: any) => `translate(${d.x},${d.y})`);
    });

    return () => { sim.stop(); };
  }, [nodes, links, currentSlug, height]);

  return (
    <div ref={wrapRef} style={{ position: 'relative', width: '100%' }}>
      <svg
        ref={svgRef}
        width="100%"
        height={height}
        style={{
          display: 'block',
          background: 'var(--canvas-bg)',
          borderRadius: 12,
          border: '1px solid var(--border)',
        }}
      />
      {/* Hover tooltip */}
      {hovered && (
        <div style={{
          position: 'absolute',
          bottom: 12,
          left: 12,
          background: 'var(--surface)',
          border: '1px solid var(--border)',
          borderRadius: 8,
          padding: '8px 14px',
          pointerEvents: 'none',
          fontFamily: 'IBM Plex Mono, monospace',
          fontSize: 11,
          color: 'var(--text-2)',
          maxWidth: 240,
        }}>
          <div style={{ fontWeight: 500, color: 'var(--text)', marginBottom: 3 }}>{hovered.title}</div>
          {hovered.tags.length > 0 && (
            <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap' }}>
              {hovered.tags.map((t) => (
                <span key={t} style={{
                  padding: '1px 6px',
                  borderRadius: 99,
                  background: 'var(--tag-bg)',
                  color: 'var(--tag-text)',
                  fontSize: 10,
                }}>#{t}</span>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}