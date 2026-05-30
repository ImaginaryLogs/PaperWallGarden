/* @ts-self-types="./federated_sim2.d.ts" */

export class FederatedGridCA {
    static __wrap(ptr) {
        const obj = Object.create(FederatedGridCA.prototype);
        obj.__wbg_ptr = ptr;
        FederatedGridCAFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FederatedGridCAFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_federatedgridca_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    byzantine_count() {
        const ret = wasm.federatedgridca_byzantine_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    byzantine_ptr() {
        const ret = wasm.federatedgridca_byzantine_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    clear_byzantine() {
        wasm.federatedgridca_clear_byzantine(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    converged_count() {
        const ret = wasm.federatedgridca_converged_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    gossip_weight() {
        const ret = wasm.federatedgridca_gossip_weight(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    height() {
        const ret = wasm.federatedgridca_height(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} row
     * @param {number} col
     */
    inject_byzantine(row, col) {
        wasm.federatedgridca_inject_byzantine(this.__wbg_ptr, row, col);
    }
    /**
     * @returns {number}
     */
    loss_history_ptr() {
        const ret = wasm.federatedgridca_loss_history_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    loss_ptr() {
        const ret = wasm.federatedgridca_loss_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    lr() {
        const ret = wasm.federatedgridca_lr(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    mean_loss() {
        const ret = wasm.federatedgridca_mean_loss(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    model_ptr() {
        const ret = wasm.federatedgridca_model_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    n() {
        const ret = wasm.federatedgridca_n(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {FederatedGridCA}
     */
    static new() {
        const ret = wasm.federatedgridca_new();
        return FederatedGridCA.__wrap(ret);
    }
    reset() {
        wasm.federatedgridca_reset(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    round() {
        const ret = wasm.federatedgridca_round(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} w
     */
    set_gossip_weight(w) {
        wasm.federatedgridca_set_gossip_weight(this.__wbg_ptr, w);
    }
    /**
     * @param {number} lr
     */
    set_lr(lr) {
        wasm.federatedgridca_set_lr(this.__wbg_ptr, lr);
    }
    /**
     * @returns {number}
     */
    state_ptr() {
        const ret = wasm.federatedgridca_state_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    step() {
        wasm.federatedgridca_step(this.__wbg_ptr);
    }
    /**
     * @param {number} n
     */
    step_n(n) {
        wasm.federatedgridca_step_n(this.__wbg_ptr, n);
    }
    /**
     * @returns {number}
     */
    target_ptr() {
        const ret = wasm.federatedgridca_target_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    trust_ptr() {
        const ret = wasm.federatedgridca_trust_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    width() {
        const ret = wasm.federatedgridca_width(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) FederatedGridCA.prototype[Symbol.dispose] = FederatedGridCA.prototype.free;
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
        "./federated_sim2_bg.js": import0,
    };
}

const FederatedGridCAFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_federatedgridca_free(ptr, 1));

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
        module_or_path = new URL('federated_sim2_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
