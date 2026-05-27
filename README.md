# Astro Starter Kit: Minimal

```sh
npm create astro@latest -- --template minimal
```

> 🧑‍🚀 **Seasoned astronaut?** Delete this file. Have fun!

## 🚀 Project Structure

Inside of your Astro project, you'll see the following folders and files:

```text
my-garden/
│
├── .github/
│   └── workflows/
│       └── deploy.yml              # GitHub Actions: build + deploy to gh-pages
│
├── obsidian-vault/                 # ← Your actual Obsidian vault (open this in Obsidian)
│   ├── .obsidian/                  # Obsidian app config (gitignored for sensitive parts)
│   │   ├── app.json
│   │   └── plugins/
│   ├── _attachments/               # Images, PDFs vault-wide
│   ├── quantum/
│   │   ├── bloch-sphere-notes.md
│   │   └── qubit-overview.canvas
│   ├── ml/
│   │   └── transformer-arch.canvas
│   ├── automata/
│   │   └── turing-machine.md
│   └── quant-fi/
│       └── black-scholes.md
│
├── src/                            # Astro source
│   ├── content/                    # Astro Content Collections config
│   │   ├── config.ts               # defineCollection schemas
│   │   ├── garden -> ../../obsidian-vault  # SYMLINK to vault
│   │   └── playground/             # MDX files for IDP
│   │       ├── quantum-circuit.mdx
│   │       ├── hmm-inference.mdx
│   │       └── black-scholes-sim.mdx
│   │
│   ├── components/
│   │   ├── garden/
│   │   │   ├── ObsidianCanvas.tsx   # Canvas JSON → React Flow
│   │   │   ├── WikiLink.astro       # [[wikilink]] resolver
│   │   │   ├── Callout.astro        # > [!note] Obsidian callouts
│   │   │   ├── BacklinkPanel.astro  # Backlink graph sidebar
│   │   │   └── GraphView.tsx        # D3/React Force-graph
│   │   │
│   │   ├── playground/
│   │   │   ├── WasmLoader.tsx       # Generic Wasm module wrapper
│   │   │   ├── BlochSphere.tsx      # Three.js quantum viz
│   │   │   ├── CircuitEditor.tsx    # Quantum circuit drag-and-drop
│   │   │   ├── TuringMachine.tsx    # Automata simulator
│   │   │   ├── StochasticChart.tsx  # Monte Carlo / GBM viz
│   │   │   └── MLInference.tsx      # ONNX Runtime Web component
│   │   │
│   │   └── shared/
│   │       ├── CrossLink.astro      # ODG ↔ IDP cross-reference links
│   │       └── Layout.astro         # Unified site shell
│   │
│   ├── pages/
│   │   ├── index.astro              # Homepage / hub
│   │   ├── garden/
│   │   │   └── [...slug].astro      # Dynamic ODG routing
│   │   └── playground/
│   │       └── [...slug].astro      # Dynamic IDP routing
│   │
│   ├── lib/
│   │   ├── obsidian/
│   │   │   ├── parseCanvas.ts       # .canvas JSON parser
│   │   │   ├── resolveWikilinks.ts  # [[link]] → URL resolver
│   │   │   └── buildBacklinks.ts    # Backlink index builder
│   │   └── wasm/
│   │       └── loader.ts            # Wasm module loader utility
│   │
│   └── styles/
│       ├── global.css
│       ├── garden.css               # Obsidian-like typography
│       └── playground.css           # Code-lab aesthetic
│
├── wasm-modules/                    # Rust/C++ source for Wasm
│   ├── quantum-sim/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs               # Quantum gate math
│   ├── black-scholes/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs               # Options pricing engine
│   └── compiled/                    # Build output (.wasm + .js glue)
│       ├── quantum_sim_bg.wasm
│       └── black_scholes_bg.wasm
│
├── public/                          # Static assets (copied verbatim)
│   ├── fonts/
│   └── og-images/
│
├── astro.config.mjs
├── tsconfig.json
├── package.json
└── .gitignore
```

Astro looks for `.astro` or `.md` files in the `src/pages/` directory. Each page is exposed as a route based on its file name.

There's nothing special about `src/components/`, but that's where we like to put any Astro/React/Vue/Svelte/Preact components.

Any static assets, like images, can be placed in the `public/` directory.

## 🧞 Commands

All commands are run from the root of the project, from a terminal:

| Command                   | Action                                           |
| :------------------------ | :----------------------------------------------- |
| `npm install`             | Installs dependencies                            |
| `npm run dev`             | Starts local dev server at `localhost:4321`      |
| `npm run build`           | Build your production site to `./dist/`          |
| `npm run preview`         | Preview your build locally, before deploying     |
| `npm run astro ...`       | Run CLI commands like `astro add`, `astro check` |
| `npm run astro -- --help` | Get help using the Astro CLI                     |

## 👀 Want to learn more?

Feel free to check [our documentation](https://docs.astro.build) or jump into our [Discord server](https://astro.build/chat).
