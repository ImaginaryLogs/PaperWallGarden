import { QuartzPageTypePlugin, QuartzEmitterPlugin, QuartzEmitterPluginInstance, QuartzPageTypePluginInstance } from '@quartz-community/types';

interface WasmPlaygroundOptions {
    wasmModulesDir?: string;
    publicPrefix?: string;
    frontmatterKey?: string;
}
declare const WasmPlaygroundPage: QuartzPageTypePlugin<WasmPlaygroundOptions>;

interface WasmEmitterOptions {
    wasmModulesDir?: string;
    publicPrefix?: string;
}
declare const WasmModuleEmitter: QuartzEmitterPlugin<WasmEmitterOptions>;

declare function WasmPlayground(opts?: WasmPlaygroundOptions): (QuartzEmitterPluginInstance | QuartzPageTypePluginInstance)[];

export { type WasmEmitterOptions, WasmModuleEmitter, WasmPlayground, type WasmPlaygroundOptions, WasmPlaygroundPage };
