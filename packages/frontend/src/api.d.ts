// Type declarations for things established in preload.ts

export type Core = typeof import("circuitsim-glue");

export async function showModal(
    type: "save",
    config: Electron.SaveDialogOptions,
): Promise<Electron.SaveDialogReturnValue>;
export async function showModal(
    type: "open",
    config: Electron.OpenDialogOptions,
): Promise<Electron.OpenDialogReturnValue>;
export async function showModal(
    type: "box",
    config: Electron.MessageBoxOptions,
): Promise<Electron.MessageBoxReturnValue>;
export async function showModal(type: "menu", config: string[]): Promise<number>;
export type DialogBindings = {
    showModal: typeof showModal;
};

export type StorageBindings = {
    get(k: string): any;
    set(k: string, v: any): void;
    getAll(): object;
    setAll(data: object): void;
};

export type FSBindings = {
    read(fp: string): Promise<string>;
    write(fp: string, content: string): Promise<void>;
    exists(fp: string): boolean;
    basename(fp: string): string;
    getPath(f: File): string;
};

export type API = {
    core: Core;
    dialog: DialogBindings;
    storage: StorageBindings;
    fs: FSBindings;
};

export type Handler<F> = (
    e: Electron.IpcMainInvokeEvent,
    ...args: Parameters<F>
) => ReturnType<F> | Awaited<ReturnType<F>>;
export type SyncHandler<F> = (
    e: Omit<Electron.IpcMainEvent, "returnValue"> & { returnValue: ReturnType<F> },
    ...args: Parameters<F>
) => void;

declare global {
    interface Window {
        api: API;
    }
}
