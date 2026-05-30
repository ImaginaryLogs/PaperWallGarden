/* @ts-self-types="./petri_sim2.d.ts" */

export class BusinessProcessSPN {
    static __wrap(ptr) {
        const obj = Object.create(BusinessProcessSPN.prototype);
        obj.__wbg_ptr = ptr;
        BusinessProcessSPNFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BusinessProcessSPNFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_businessprocessspn_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    anim_ptr() {
        const ret = wasm.businessprocessspn_anim_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    applications_done() {
        const ret = wasm.businessprocessspn_applications_done(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    applications_in() {
        const ret = wasm.businessprocessspn_applications_in(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    applications_rejected() {
        const ret = wasm.businessprocessspn_applications_rejected(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    bottleneck_prob() {
        const ret = wasm.businessprocessspn_bottleneck_prob(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    fire_count_ptr() {
        const ret = wasm.businessprocessspn_fire_count_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    fired_ptr() {
        const ret = wasm.businessprocessspn_fired_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    hmm_belief_ptr() {
        const ret = wasm.businessprocessspn_hmm_belief_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    hmm_hidden_states() {
        const ret = wasm.businessprocessspn_hmm_hidden_states(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    hmm_seq_head() {
        const ret = wasm.businessprocessspn_hmm_seq_head(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    hmm_seq_len() {
        const ret = wasm.businessprocessspn_hmm_seq_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    hmm_seq_ptr() {
        const ret = wasm.businessprocessspn_hmm_seq_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    inject_application() {
        wasm.businessprocessspn_inject_application(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    mean_sojourn() {
        const ret = wasm.businessprocessspn_mean_sojourn(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {BusinessProcessSPN}
     */
    static new() {
        const ret = wasm.businessprocessspn_new();
        return BusinessProcessSPN.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    num_places() {
        const ret = wasm.businessprocessspn_num_places(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    num_transitions() {
        const ret = wasm.businessprocessspn_num_transitions(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    place_x_ptr() {
        const ret = wasm.businessprocessspn_place_x_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    place_y_ptr() {
        const ret = wasm.businessprocessspn_place_y_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    reset() {
        wasm.businessprocessspn_reset(this.__wbg_ptr);
    }
    step() {
        wasm.businessprocessspn_step(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    step_count() {
        const ret = wasm.businessprocessspn_step_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} n
     */
    step_n(n) {
        wasm.businessprocessspn_step_n(this.__wbg_ptr, n);
    }
    /**
     * @returns {number}
     */
    throughput() {
        const ret = wasm.businessprocessspn_throughput(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} p
     * @returns {number}
     */
    tokens_at(p) {
        const ret = wasm.businessprocessspn_tokens_at(this.__wbg_ptr, p);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    tokens_ptr() {
        const ret = wasm.businessprocessspn_tokens_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    trans_x_ptr() {
        const ret = wasm.businessprocessspn_trans_x_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    trans_y_ptr() {
        const ret = wasm.businessprocessspn_trans_y_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) BusinessProcessSPN.prototype[Symbol.dispose] = BusinessProcessSPN.prototype.free;
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
        "./petri_sim2_bg.js": import0,
    };
}

const BusinessProcessSPNFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_businessprocessspn_free(ptr, 1));

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
        module_or_path = new URL('petri_sim2_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
