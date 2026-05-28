// src/index.ts
import path from "path";
import fs2 from "fs/promises";
import { globby } from "globby";

// src/WasmPlayground.tsx
import { jsx, jsxs } from "preact/jsx-runtime";
function WasmPlaygroundBody(opts) {
  return () => {
    const PlaygroundComponent = (props) => {
      const { fileData } = props;
      const moduleName = fileData.frontmatter?.[opts.frontmatterKey];
      const title = fileData.frontmatter?.title ?? moduleName;
      if (!moduleName) {
        return /* @__PURE__ */ jsx("div", { class: "wasm-playground wasm-playground--error", children: /* @__PURE__ */ jsxs("p", { children: [
          "Missing ",
          /* @__PURE__ */ jsx("code", { children: opts.frontmatterKey }),
          " in frontmatter. Add it to the Markdown stub to enable the playground."
        ] }) });
      }
      const jsUrl = `${opts.publicPrefix}/${moduleName}/${moduleName}.js`;
      const wasmUrl = `${opts.publicPrefix}/${moduleName}/${moduleName}_bg.wasm`;
      return /* @__PURE__ */ jsxs("div", { class: "wasm-playground", children: [
        /* @__PURE__ */ jsxs("header", { class: "wasm-playground__header", children: [
          /* @__PURE__ */ jsx("h1", { class: "wasm-playground__title", children: title }),
          /* @__PURE__ */ jsx("span", { class: "wasm-playground__badge", children: "Rust \u2192 WASM" })
        ] }),
        /* @__PURE__ */ jsx("div", { class: "wasm-playground__description", children: props.children }),
        /* @__PURE__ */ jsx(
          "div",
          {
            id: "wasm-sandbox",
            "data-module": moduleName,
            "data-js": jsUrl,
            "data-wasm": wasmUrl,
            children: /* @__PURE__ */ jsx("div", { class: "wasm-playground__loading", children: "Loading WASM module\u2026" })
          }
        )
      ] });
    };
    PlaygroundComponent.displayName = "WasmPlayground";
    PlaygroundComponent.css = PLAYGROUND_CSS;
    PlaygroundComponent.afterDOMLoaded = CLIENT_SCRIPT;
    return PlaygroundComponent;
  };
}
var CLIENT_SCRIPT = (
  /* javascript */
  `
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
  runBtn.textContent = "\u25B6  Run"

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
        "\\n\u2192 " + JSON.stringify(result)
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
);
var PLAYGROUND_CSS = `
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
`;

// src/pageType.ts
import { slugifyFilePath } from "@quartz-community/utils/path";
import { join } from "path";
var wasmPageMatcher = ({ fileData }) => {
  return fileData && "wasmModuleData" in fileData;
};
var WasmPlaygroundPage = (opts) => {
  const config = {
    wasmModulesDir: opts?.wasmModulesDir ?? "./wasm-modules",
    publicPrefix: opts?.publicPrefix ?? "/wasm",
    frontmatterKey: opts?.frontmatterKey ?? "wasm-module"
  };
  return {
    name: "WasmPlaygroundPage",
    priority: 30,
    fileExtensions: [".wasm", ".wat"],
    // 1. Fulfills check: "layout" in instance
    layout: "content",
    // 2. Fulfills check: "body" in instance
    body: WasmPlaygroundBody({ publicPrefix: config.publicPrefix, frontmatterKey: config.frontmatterKey }),
    // 3. Fulfills check: "match" in instance
    match: wasmPageMatcher,
    generate({ ctx }) {
      const virtualPages = [];
      const wasmSourceFiles = ctx.allFiles.filter(
        (filePath) => filePath.endsWith(".wasm") || filePath.endsWith(".wat")
      );
      for (const filePath of wasmSourceFiles) {
        const baseName = filePath.split("/").pop() ?? "WASM Module";
        const slug = slugifyFilePath(filePath);
        virtualPages.push({
          slug,
          title: baseName,
          data: {
            frontmatter: {
              title: baseName,
              tags: ["wasm-playground", "webassembly"]
            },
            wasmModuleData: {
              sourcePath: filePath,
              publicUrl: join(config.publicPrefix, baseName),
              config
            }
          }
        });
      }
      return virtualPages;
    },
    shouldPublish(_ctx, content) {
      const relativePath = content[1]?.data?.relativePath ?? "";
      return relativePath.endsWith(".wasm") || relativePath.endsWith(".wat");
    }
  };
};

// src/emitter.ts
import { join as join2 } from "path";
import fs from "fs/promises";
var WasmModuleEmitter = (opts) => {
  const config = {
    wasmModulesDir: opts?.wasmModulesDir ?? "./wasm-modules",
    publicPrefix: opts?.publicPrefix ?? "/wasm"
  };
  return {
    name: "WasmModuleEmitter",
    // This method fulfills the runtime validation criteria: "emit" in instance
    async emit(ctx, _content, _resources) {
      const emittedFiles = [];
      const outputDir = join2(ctx.argv.output, config.publicPrefix);
      await fs.mkdir(outputDir, { recursive: true });
      let filesToEmit = [];
      try {
        filesToEmit = await fs.readdir(config.wasmModulesDir);
      } catch (err) {
        return [];
      }
      for (const fileName of filesToEmit) {
        if (fileName.endsWith(".wasm") || fileName.endsWith(".wat")) {
          const sourceFilePath = join2(config.wasmModulesDir, fileName);
          const destinationPath = join2(outputDir, fileName);
          try {
            const binaryBuffer = await fs.readFile(sourceFilePath);
            await fs.writeFile(destinationPath, binaryBuffer);
            const trackedPath = join2(config.publicPrefix, fileName);
            emittedFiles.push(trackedPath);
          } catch (error) {
            console.error(`[WASM EMITTER ERROR] Failed to copy asset ${fileName}:`, error);
          }
        }
      }
      return emittedFiles;
    }
  };
};

// src/index.ts
function WasmAssetsEmitter(opts = {}) {
  const wasmDir = opts.wasmModulesDir ?? path.resolve(process.cwd(), "wasm-modules");
  const publicPfx = opts.publicPrefix ?? "/wasm";
  return {
    name: "WasmAssetsEmitter",
    getQuartzComponents: () => [],
    async emit(ctx, _content, _res) {
      const outBase = path.join(ctx.argv.output, publicPfx.replace(/^\//, ""));
      await fs2.mkdir(outBase, { recursive: true });
      const pkgDirs = await globby("*/pkg", {
        cwd: wasmDir,
        onlyDirectories: true,
        absolute: true
      });
      const written = [];
      for (const pkgDir of pkgDirs) {
        const moduleName = path.basename(path.dirname(pkgDir));
        const destDir = path.join(outBase, moduleName);
        await fs2.mkdir(destDir, { recursive: true });
        const assets = await globby(["*.wasm", "*.js", "!*_bg.js"], {
          cwd: pkgDir,
          absolute: true
        });
        for (const asset of assets) {
          const dest = path.join(destDir, path.basename(asset));
          await fs2.copyFile(asset, dest);
          written.push(dest);
        }
      }
      return written;
    }
  };
}
function WasmPlaygroundPageType(opts = {}) {
  const fmKey = opts.frontmatterKey ?? "wasm-module";
  const publicPfx = opts.publicPrefix ?? "/wasm";
  return {
    name: "WasmPlayground",
    frame: "playground-frame",
    layout: "content",
    priority: 10,
    match({ slug, fileData }) {
      return slug.startsWith("wasm-playground/") && !!fileData?.frontmatter?.[fmKey];
    },
    body: WasmPlaygroundBody({ publicPrefix: publicPfx, frontmatterKey: fmKey })
  };
}
function WasmPlayground(opts = {}) {
  return [WasmAssetsEmitter(opts), WasmPlaygroundPageType(opts)];
}
export {
  WasmModuleEmitter,
  WasmPlayground,
  WasmPlaygroundPage
};
