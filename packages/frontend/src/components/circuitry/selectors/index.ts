import { ComponentMap } from "@/lib/types";
import { mux } from "./mux";
import { demux } from "./demux";
import { decoder } from "./decoder";

export const selectors = {
    mux,
    demux,
    decoder,
} satisfies Partial<ComponentMap>;
