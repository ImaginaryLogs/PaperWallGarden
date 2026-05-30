/* @ts-self-types="./federated_sim1.d.ts" */

export class FederatedSim {
    static __wrap(ptr) {
        const obj = Object.create(FederatedSim.prototype);
        obj.__wbg_ptr = ptr;
        FederatedSimFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FederatedSimFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_federatedsim_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    classes() {
        const ret = wasm.federatedsim_classes(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    client_count() {
        const ret = wasm.federatedsim_client_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * L2 distance between global model and client c's model
     * @param {number} c
     * @returns {number}
     */
    client_divergence(c) {
        const ret = wasm.federatedsim_client_divergence(this.__wbg_ptr, c);
        return ret;
    }
    /**
     * @returns {number}
     */
    client_losses_ptr() {
        const ret = wasm.federatedsim_client_losses_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    client_weights_ptr() {
        const ret = wasm.federatedsim_client_weights_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    current_round() {
        const ret = wasm.federatedsim_current_round(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    features() {
        const ret = wasm.federatedsim_features(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Mean absolute weight of the global model (proxy for model magnitude)
     * @returns {number}
     */
    global_model_norm() {
        const ret = wasm.federatedsim_global_model_norm(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    global_weights_ptr() {
        const ret = wasm.federatedsim_global_weights_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    loss_history_ptr() {
        const ret = wasm.federatedsim_loss_history_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    max_rounds() {
        const ret = wasm.federatedsim_max_rounds(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    model_size() {
        const ret = wasm.federatedsim_model_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {FederatedSim}
     */
    static new() {
        const ret = wasm.federatedsim_new();
        return FederatedSim.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    participation_ptr() {
        const ret = wasm.federatedsim_participation_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    reset() {
        wasm.federatedsim_reset(this.__wbg_ptr);
    }
    run_round() {
        wasm.federatedsim_run_round(this.__wbg_ptr);
    }
    /**
     * @param {number} n
     */
    run_rounds(n) {
        wasm.federatedsim_run_rounds(this.__wbg_ptr, n);
    }
    /**
     * @param {number} m
     */
    set_agg_method(m) {
        wasm.federatedsim_set_agg_method(this.__wbg_ptr, m);
    }
    /**
     * @param {number} p
     */
    set_dropout(p) {
        wasm.federatedsim_set_dropout(this.__wbg_ptr, p);
    }
    /**
     * @param {number} lr
     */
    set_lr(lr) {
        wasm.federatedsim_set_lr(this.__wbg_ptr, lr);
    }
}
if (Symbol.dispose) FederatedSim.prototype[Symbol.dispose] = FederatedSim.prototype.free;
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_1506f2235d1bdba0: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./federated_sim1_bg.js": import0,
    };
}

const FederatedSimFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_federatedsim_free(ptr, 1));

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('federated_sim1_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
