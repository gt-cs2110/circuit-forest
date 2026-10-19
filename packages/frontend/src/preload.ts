// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import * as core from "circuitsim-glue";
import { contextBridge, ipcRenderer, webFrame, webUtils } from "electron";
import type { DialogBindings, StorageBindings, ZoomDirection } from "./api";

contextBridge.exposeInMainWorld("api", {
    core,
    dialog: {
        async showModal(type: string, config: unknown) {
            return ipcRenderer.invoke("show_modal", type, config);
        },
    } satisfies DialogBindings,
    storage: {
        get(k: string): unknown {
            return ipcRenderer.sendSync("config_get", k);
        },
        set(k: string, v: unknown): void {
            return ipcRenderer.sendSync("config_set", k, v);
        },
        getAll(): object {
            return ipcRenderer.sendSync("config_get_all");
        },
        setAll(data: object): void {
            return ipcRenderer.sendSync("config_set_all", data);
        },
    } satisfies StorageBindings,
    fs: {
        async read(fp: string): Promise<string> {
            return ipcRenderer.invoke("fs_read", fp);
        },
        async write(fp: string, content: string): Promise<void> {
            return ipcRenderer.invoke("fs_write", fp, content);
        },
        exists(fp: string): boolean {
            return ipcRenderer.sendSync("fs_exists", fp);
        },
        basename(fp: string): string {
            return ipcRenderer.sendSync("fs_path_basename", fp);
        },
        getPath(file: File): string {
            return webUtils.getPathForFile(file);
        },
    },
    menu: {
        onZoom(callback: (direction: ZoomDirection) => void): () => void {
            const listener = (_: unknown, direction: ZoomDirection) => callback(direction);
            ipcRenderer.on("menu_zoom", listener);
            return () => ipcRenderer.off("menu_zoom", listener);
        },
    },
    view: {
        getZoomFactor(): number {
            return webFrame.getZoomFactor();
        },
        setZoomFactor(factor: number): void {
            webFrame.setZoomFactor(factor);
        },
    },
});
