/* tslint:disable */
/* eslint-disable */
/**
* @returns {Promise<void>}
*/
export function connect_imkey(): Promise<void>;
/**
* @param {string} apdu
* @returns {Promise<string>}
*/
export function send_command(apdu: string): Promise<string>;
/**
* @param {string} file_path
* @returns {Promise<string>}
*/
export function bind_check(file_path: string): Promise<string>;
/**
* @param {string} file_path
* @returns {Promise<string>}
*/
export function bind_acquire(file_path: string): Promise<string>;
/**
* @returns {Promise<void>}
*/
export function bind_display_code(): Promise<void>;
/**
* @param {string} seg_wit
* @param {string} network
* @param {string} path
* @returns {Promise<string>}
*/
export function get_address(seg_wit: string, network: string, path: string): Promise<string>;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly connect_imkey: () => number;
  readonly send_command: (a: number, b: number) => number;
  readonly bind_check: (a: number, b: number) => number;
  readonly bind_acquire: (a: number, b: number) => number;
  readonly bind_display_code: () => number;
  readonly get_address: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly rustsecp256k1_v0_6_1_context_create: (a: number) => number;
  readonly rustsecp256k1_v0_6_1_context_destroy: (a: number) => void;
  readonly rustsecp256k1_v0_6_1_default_illegal_callback_fn: (a: number, b: number) => void;
  readonly rustsecp256k1_v0_6_1_default_error_callback_fn: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly wasm_bindgen__convert__closures__invoke1_mut__h2d5a35eab42c80ea: (a: number, b: number, c: number) => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly wasm_bindgen__convert__closures__invoke2_mut__h73bef5e2f08a4d85: (a: number, b: number, c: number, d: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
