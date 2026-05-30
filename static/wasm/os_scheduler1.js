/* @ts-self-types="./os_scheduler1.d.ts" */

export class Scheduler {
    static __wrap(ptr) {
        const obj = Object.create(Scheduler.prototype);
        obj.__wbg_ptr = ptr;
        SchedulerFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SchedulerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_scheduler_free(ptr, 0);
    }
    /**
     * @param {number} n
     */
    advance(n) {
        wasm.scheduler_advance(this.__wbg_ptr, n);
    }
    /**
     * @returns {number}
     */
    completed_count() {
        const ret = wasm.scheduler_completed_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    cpu_util() {
        const ret = wasm.scheduler_cpu_util(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    current_tick() {
        const ret = wasm.scheduler_current_tick(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    gantt_head() {
        const ret = wasm.scheduler_gantt_head(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    gantt_len() {
        const ret = wasm.scheduler_gantt_len(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    gantt_pids_ptr() {
        const ret = wasm.scheduler_gantt_pids_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    gantt_prios_ptr() {
        const ret = wasm.scheduler_gantt_prios_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    max_procs() {
        const ret = wasm.scheduler_max_procs(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Scheduler}
     */
    static new() {
        const ret = wasm.scheduler_new();
        return Scheduler.__wrap(ret);
    }
    /**
     * @returns {number}
     */
    num_queues() {
        const ret = wasm.scheduler_num_queues(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    pcb_fields() {
        const ret = wasm.scheduler_pcb_fields(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    pcb_ptr() {
        const ret = wasm.scheduler_pcb_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    queue_lengths_ptr() {
        const ret = wasm.scheduler_queue_lengths_ptr(this.__wbg_ptr);
        return ret >>> 0;
    }
    reset() {
        wasm.scheduler_reset(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    running_slot() {
        const ret = wasm.scheduler_running_slot(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} a
     */
    set_algorithm(a) {
        wasm.scheduler_set_algorithm(this.__wbg_ptr, a);
    }
    spawn_new() {
        wasm.scheduler_spawn_new(this.__wbg_ptr);
    }
    tick() {
        wasm.scheduler_tick(this.__wbg_ptr);
    }
}
if (Symbol.dispose) Scheduler.prototype[Symbol.dispose] = Scheduler.prototype.free;
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
        "./os_scheduler1_bg.js": import0,
    };
}

const SchedulerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_scheduler_free(ptr, 1));

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
        module_or_path = new URL('os_scheduler1_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
