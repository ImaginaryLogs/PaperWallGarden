/* @ts-self-types="./qcomm_sim.d.ts" */

export class QuantumChannel {
    static __wrap(ptr) {
        const obj = Object.create(QuantumChannel.prototype);
        obj.__wbg_ptr = ptr;
        QuantumChannelFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        QuantumChannelFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_quantumchannel_free(ptr, 0);
    }
    /**
     * Apply CNOT: control=q0, target=q1
     * For a separable state |c⟩|t⟩ this is an approximation;
     * true entanglement requires a full 4-qubit state vector.
     * Here we implement it via the classical approximation valid for
     * computational basis states (sufficient for visualisation).
     * @param {number} control
     * @param {number} target
     */
    apply_cnot(control, target) {
        wasm.quantumchannel_apply_cnot(this.__wbg_ptr, control, target);
    }
    /**
     * @param {number} qubit
     * @param {number} gate
     */
    apply_gate(qubit, gate) {
        wasm.quantumchannel_apply_gate(this.__wbg_ptr, qubit, gate);
    }
    /**
     * @returns {number}
     */
    average_qber() {
        const ret = wasm.quantumchannel_average_qber(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    bloch_coords_ptr() {
        const ret = wasm.quantumchannel_bloch_coords_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {boolean}
     */
    eve_active() {
        const ret = wasm.quantumchannel_eve_active(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_secure() {
        const ret = wasm.quantumchannel_is_secure(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    key_bits_ptr() {
        const ret = wasm.quantumchannel_key_bits_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    key_len() {
        const ret = wasm.quantumchannel_key_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    log_head() {
        const ret = wasm.quantumchannel_log_head(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    log_len() {
        const ret = wasm.quantumchannel_log_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    max_qubits() {
        const ret = wasm.quantumchannel_max_qubits(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} qubit
     * @returns {number}
     */
    measure(qubit) {
        const ret = wasm.quantumchannel_measure(this.__wbg_ptr, qubit);
        return ret;
    }
    /**
     * @returns {number}
     */
    measurement_log_ptr() {
        const ret = wasm.quantumchannel_measurement_log_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {QuantumChannel}
     */
    static new() {
        const ret = wasm.quantumchannel_new();
        return QuantumChannel.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    noise() {
        const ret = wasm.quantumchannel_noise(this.__wbg_ptr);
        return ret;
    }
    /**
     * Prepare a Bell state on qubits 0 and 1
     * |Φ+⟩ = (|00⟩+|11⟩)/√2 (default)
     * @param {number} state_id
     */
    prepare_bell_state(state_id) {
        wasm.quantumchannel_prepare_bell_state(this.__wbg_ptr, state_id);
    }
    /**
     * Probability of measuring |0⟩ for qubit q
     * @param {number} qubit
     * @returns {number}
     */
    prob0(qubit) {
        const ret = wasm.quantumchannel_prob0(this.__wbg_ptr, qubit);
        return ret;
    }
    /**
     * @returns {number}
     */
    qber_history_ptr() {
        const ret = wasm.quantumchannel_qber_history_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    qkd_rounds() {
        const ret = wasm.quantumchannel_qkd_rounds(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    qubit_states_ptr() {
        const ret = wasm.quantumchannel_qubit_states_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    reset_all() {
        wasm.quantumchannel_reset_all(this.__wbg_ptr);
    }
    /**
     * @param {number} qubit
     */
    reset_qubit(qubit) {
        wasm.quantumchannel_reset_qubit(this.__wbg_ptr, qubit);
    }
    /**
     * @returns {number}
     */
    run_bb84_round() {
        const ret = wasm.quantumchannel_run_bb84_round(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {boolean} active
     */
    set_eve(active) {
        wasm.quantumchannel_set_eve(this.__wbg_ptr, active);
    }
    /**
     * @param {number} n
     */
    set_noise(n) {
        wasm.quantumchannel_set_noise(this.__wbg_ptr, n);
    }
}
if (Symbol.dispose) QuantumChannel.prototype[Symbol.dispose] = QuantumChannel.prototype.free;
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
        "./qcomm_sim_bg.js": import0,
    };
}

const QuantumChannelFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_quantumchannel_free(ptr, 1));

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
        module_or_path = new URL('qcomm_sim_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
