import React from 'react';

interface Backlink {
  title: string;
  slug: string;
}

interface BacklinkPanelProps {
  backlinks: Backlink[];
}

export default function BacklinkPanel({ backlinks }: BacklinkPanelProps) {
  if (!backlinks || backlinks.length === 0) {
    return (
      <div className="backlink-panel empty">
        <h4>Backlinks</h4>
        <p className="no-links">No incoming connections to this note.</p>
      </div>
    );
  }

  return (
    <div className="backlink-panel">
      <h4>Linked References</h4>
      <ul>
        {backlinks.map((link) => (
          <li key={link.slug}>
            <a href={`/garden/${link.slug}`} className="backlink-item">
              🌿 {link.title}
            </a>
          </li>
        ))}
      </ul>
    </div>
  );
}