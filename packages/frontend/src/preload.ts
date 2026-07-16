// https://www.electronjs.org/docs/latest/tutorial/process-model#preload-scripts

import * as core from "circuitsim-glue";
import { contextBridge, ipcRenderer, webUtils } from "electron";

contextBridge.exposeInMainWorld("api", {
    core,
    dialog: {
        async showModal(type: string, config: any): Promise<any> {
            return ipcRenderer.invoke("show_modal", type, config);
        },
    },
    storage: {
        get(k: string): any {
            return ipcRenderer.sendSync("config_get", k);
        },
        set(k: string, v: any): void {
            return ipcRenderer.sendSync("config_set", k, v);
        },
        getAll(): object {
            return ipcRenderer.sendSync("config_get_all");
        },
        setAll(data: object): void {
            return ipcRenderer.sendSync("config_set_all", data);
        },
    },
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
});
