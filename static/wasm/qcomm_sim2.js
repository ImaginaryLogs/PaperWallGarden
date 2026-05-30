/* @ts-self-types="./qcomm_sim2.d.ts" */

export class QCommPetri {
    static __wrap(ptr) {
        const obj = Object.create(QCommPetri.prototype);
        obj.__wbg_ptr = ptr;
        QCommPetriFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        QCommPetriFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_qcommpetri_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    decoherence_events() {
        const ret = wasm.qcommpetri_decoherence_events(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    entanglements_established() {
        const ret = wasm.qcommpetri_entanglements_established(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    fidelity_score() {
        const ret = wasm.qcommpetri_fidelity_score(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    inputs_ptr() {
        const ret = wasm.qcommpetri_inputs_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    lambda_effective_ptr() {
        const ret = wasm.qcommpetri_lambda_effective_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {QCommPetri}
     */
    static new() {
        const ret = wasm.qcommpetri_new();
        return QCommPetri.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    num_places() {
        const ret = wasm.qcommpetri_num_places(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    num_transitions() {
        const ret = wasm.qcommpetri_num_transitions(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    outputs_ptr() {
        const ret = wasm.qcommpetri_outputs_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    place_x_ptr() {
        const ret = wasm.qcommpetri_place_x_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    place_y_ptr() {
        const ret = wasm.qcommpetri_place_y_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    purifications() {
        const ret = wasm.qcommpetri_purifications(this.__wbg_ptr);
        return ret >>> 0;
    }
    reset() {
        wasm.qcommpetri_reset(this.__wbg_ptr);
    }
    /**
     * @param {number} _val
     */
    set_pump_power(_val) {
        wasm.qcommpetri_set_pump_power(this.__wbg_ptr, _val);
    }
    step() {
        wasm.qcommpetri_step(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    step_count() {
        const ret = wasm.qcommpetri_step_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    tokens_ptr() {
        const ret = wasm.qcommpetri_tokens_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    trans_x_ptr() {
        const ret = wasm.qcommpetri_trans_x_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    trans_y_ptr() {
        const ret = wasm.qcommpetri_trans_y_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
}
if (Symbol.dispose) QCommPetri.prototype[Symbol.dispose] = QCommPetri.prototype.free;
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
        "./qcomm_sim2_bg.js": import0,
    };
}

const QCommPetriFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_qcommpetri_free(ptr, 1));

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
        module_or_path = new URL('qcomm_sim2_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
