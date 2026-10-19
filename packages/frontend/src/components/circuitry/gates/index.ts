import { ComponentMap } from "@/lib/types";
import { gateMetadata } from "./gate";
import { not } from "./not";
import { buffer } from "./buffer";

export const gates = {
    ...gateMetadata,
    not,
    buffer,
} satisfies Partial<ComponentMap>;
