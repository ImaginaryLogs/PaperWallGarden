# Quartz v5

> “[One] who works with the door open gets all kinds of interruptions, but [they] also occasionally gets clues as to what the world is and what might be important.” — Richard Hamming

Quartz is a set of tools that helps you publish your [digital garden](https://jzhao.xyz/posts/networked-thought) and notes as a website for free.

🔗 Read the documentation and get started: https://quartz.jzhao.xyz/

[Join the Discord Community](https://discord.gg/cRFFHYye7t)

## Sponsors

<p align="center">
  <a href="https://github.com/sponsors/jackyzha0">
    <img src="https://cdn.jsdelivr.net/gh/jackyzha0/jackyzha0/sponsorkit/sponsors.svg" />
  </a>
</p>

# Quartz WebAssembly Playground Plugin (quartz-plugin-wasm-playground)

A specialized Quartz v5 plugin ecosystem that compiles, orchestrates, and mirrors multi-language WebAssembly modules (like Rust wasm-pack binaries) directly into your dynamic digital garden layouts.

This plugin bridges the gap between high-performance systems-level computing (Rust memory linear sandboxes) and web views using custom structural routing contexts.

## Compilation Pipeline
Because the playground utilizes a nested architectural flow (Rust Code $\rightarrow$ WASM Package $\rightarrow$ Quartz Plugin Bundle $\rightarrow$ Public Garden Dist), changing underlying simulation code requires compiling from the bottom up.Whenever you update your Rust structures (lib.rs) or add a completely new module folder, run this sequence to guarantee Quartz doesn't serve cached assets:

```bash
# STEP 1: Compile the Rust Core into WASM Binaries + JS Glue Bindings
cd wasm-modules/quantum-sim
wasm-pack build --target web

# STEP 2: Rebuild the Quartz Local Plugin Workspace
cd ../../quartz-plugin-wasm-playground
npm run build

# STEP 3: Recompile Your Quartz Site & Serve to Local Network
cd ..
npm run quartz build -- --
```