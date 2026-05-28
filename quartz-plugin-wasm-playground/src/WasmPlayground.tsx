import { h } from "preact"
import type { QuartzComponentConstructor, QuartzComponentProps } from "@quartz-community/types"

interface PlaygroundOpts {
  publicPrefix: string
  frontmatterKey: string
}

export function WasmPlaygroundBody(opts: PlaygroundOpts): QuartzComponentConstructor {
  return () => {
    const PlaygroundComponent = (props: QuartzComponentProps) => {
      const { fileData } = props
      const moduleName = fileData.frontmatter?.[opts.frontmatterKey] as string | undefined
      const title      = (fileData.frontmatter?.title as string) ?? moduleName

      if (!moduleName) {
        return (
          <div class="wasm-playground wasm-playground--error">
            <p>
              Missing <code>{opts.frontmatterKey}</code> in frontmatter. Add it to the
              Markdown stub to enable the playground.
            </p>
          </div>
        )
      }

      const jsUrl   = `${opts.publicPrefix}/${moduleName}/${moduleName}.js`
      const wasmUrl = `${opts.publicPrefix}/${moduleName}/${moduleName}_bg.wasm`

      return (
        <div class="wasm-playground">
          <header class="wasm-playground__header">
            <h1 class="wasm-playground__title">{title}</h1>
            <span class="wasm-playground__badge">Rust → WASM</span>
          </header>

          {/* Markdown body rendered by Quartz above the REPL */}
          <div class="wasm-playground__description">{props.children}</div>

          {/* data-* attributes are read by the afterDOMLoaded script */}
          <div
            id="wasm-sandbox"
            data-module={moduleName}
            data-js={jsUrl}
            data-wasm={wasmUrl}
          >
            <div class="wasm-playground__loading">Loading WASM module…</div>
          </div>
        </div>
      )
    }

    PlaygroundComponent.displayName = "WasmPlayground"
    PlaygroundComponent.css = PLAYGROUND_CSS
    PlaygroundComponent.afterDOMLoaded = CLIENT_SCRIPT

    return PlaygroundComponent
  }
}

// ── Client-side loader + REPL ─────────────────────────────────────────────────
// Embedded as a string; Quartz injects it via a <script> tag after the DOM is
// ready. It dynamically imports the wasm-pack ES module, calls default init(),
// then introspects the exports to build the interactive REPL.

const CLIENT_SCRIPT = /* javascript */`
(async () => {
  const sandbox = document.getElementById("wasm-sandbox")
  if (!sandbox) return

  const jsUrl   = sandbox.dataset.js
  const wasmUrl = sandbox.dataset.wasm

  try {
    const wasmModule = await import(jsUrl)
    // wasm-pack's default export is the init function
    await wasmModule.default(wasmUrl)
    renderREPL(sandbox, wasmModule)
  } catch (err) {
    sandbox.innerHTML =
      '<p class="wasm-playground__error">Failed to load WASM module: ' + err.message + '</p>'
  }
})()

function renderREPL(container, mod) {
  // Filter out wasm-bindgen internals and the init default export
  const exports = Object.entries(mod).filter(
    ([k, v]) => typeof v === "function" && k !== "default" && !k.startsWith("__")
  )

  if (exports.length === 0) {
    container.innerHTML = '<p class="wasm-playground__error">No exported functions found. Make sure your Rust functions use #[wasm_bindgen].</p>'
    return
  }

  container.innerHTML = ""

  const repl = document.createElement("div")
  repl.className = "wasm-repl"

  // Function selector
  const select = document.createElement("select")
  select.className = "wasm-repl__select"
  exports.forEach(([name]) => {
    const opt = document.createElement("option")
    opt.value = name
    opt.textContent = name
    select.appendChild(opt)
  })

  const inputArea = document.createElement("div")
  inputArea.className = "wasm-repl__inputs"

  const output = document.createElement("pre")
  output.className = "wasm-repl__output"
  output.textContent = "// output will appear here"

  const runBtn = document.createElement("button")
  runBtn.className = "wasm-repl__run"
  runBtn.textContent = "▶  Run"

  function buildInputs(fnName) {
    inputArea.innerHTML = ""
    const fn = mod[fnName]
    if (!fn) return
    for (let i = 0; i < fn.length; i++) {
      const label = document.createElement("label")
      label.className = "wasm-repl__label"
      label.textContent = "arg" + (i + 1)
      const input = document.createElement("input")
      input.className = "wasm-repl__input"
      input.type = "text"
      input.placeholder = "argument " + (i + 1)
      input.dataset.index = String(i)
      label.appendChild(input)
      inputArea.appendChild(label)
    }
  }

  select.addEventListener("change", () => buildInputs(select.value))
  buildInputs(exports[0][0])

  runBtn.addEventListener("click", () => {
    const fnName = select.value
    const fn = mod[fnName]
    if (!fn) return

    const args = [...inputArea.querySelectorAll("input")].map(inp => {
      const v = inp.value
      return isNaN(Number(v)) || v.trim() === "" ? v : Number(v)
    })

    try {
      const result = fn(...args)
      output.textContent =
        fnName + "(" + args.map(a => JSON.stringify(a)).join(", ") + ")" +
        "\\n→ " + JSON.stringify(result)
    } catch (e) {
      output.textContent = "Error: " + e.message
    }
  })

  repl.appendChild(select)
  repl.appendChild(inputArea)
  repl.appendChild(runBtn)
  repl.appendChild(output)
  container.appendChild(repl)
}
`

// ── Scoped CSS ────────────────────────────────────────────────────────────────

const PLAYGROUND_CSS = `
.wasm-playground {
  max-width: 800px;
  margin: 0 auto;
  padding: 2rem 1rem;
  font-family: var(--bodyFont, sans-serif);
}
.wasm-playground__header {
  display: flex;
  align-items: baseline;
  gap: 1rem;
  margin-bottom: 1.5rem;
}
.wasm-playground__title { margin: 0; font-size: 1.8rem; }
.wasm-playground__badge {
  font-size: 0.75rem;
  font-family: var(--codeFont, monospace);
  background: var(--highlight, #f0f0f0);
  color: var(--darkgray, #444);
  padding: 2px 8px;
  border-radius: 4px;
}
.wasm-playground__loading { color: var(--gray, #888); font-style: italic; padding: 1rem 0; }
.wasm-playground__error {
  color: #c0392b;
  background: #fdecea;
  padding: 1rem;
  border-radius: 6px;
}
.wasm-repl {
  display: flex;
  flex-direction: column;
  gap: 0.75rem;
  margin-top: 1.5rem;
  border: 1px solid var(--lightgray, #e0e0e0);
  border-radius: 8px;
  padding: 1.25rem;
  background: var(--light, #fafafa);
}
.wasm-repl__select {
  font-family: var(--codeFont, monospace);
  font-size: 0.95rem;
  padding: 6px 10px;
  border: 1px solid var(--lightgray, #ccc);
  border-radius: 4px;
  background: var(--light);
  color: var(--darkgray);
  cursor: pointer;
}
.wasm-repl__inputs { display: flex; flex-wrap: wrap; gap: 0.5rem; }
.wasm-repl__label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 0.8rem;
  color: var(--gray);
  font-family: var(--codeFont, monospace);
}
.wasm-repl__input {
  font-family: var(--codeFont, monospace);
  font-size: 0.9rem;
  padding: 6px 10px;
  border: 1px solid var(--lightgray, #ccc);
  border-radius: 4px;
  background: var(--light);
  color: var(--darkgray);
  width: 140px;
}
.wasm-repl__run {
  align-self: flex-start;
  font-family: var(--codeFont, monospace);
  font-size: 0.9rem;
  padding: 8px 20px;
  background: var(--secondary, #4a90e2);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  transition: opacity 0.15s;
}
.wasm-repl__run:hover { opacity: 0.85; }
.wasm-repl__output {
  font-family: var(--codeFont, monospace);
  font-size: 0.88rem;
  background: var(--dark, #1e1e1e);
  color: var(--lightgray, #d4d4d4);
  padding: 1rem;
  border-radius: 6px;
  white-space: pre-wrap;
  word-break: break-all;
  min-height: 60px;
  margin: 0;
}
`