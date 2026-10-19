import { ComponentMetadata } from "@/lib/types";

import Gate from "./Gate.vue";

export const gateKinds = ["and", "or", "xor", "nand", "nor", "xnor"] as const;
export type GateKind = (typeof gateKinds)[number];

export const gateShapes = {
    and: { body: "and", negated: false },
    or: { body: "or", negated: false },
    xor: { body: "xor", negated: false },
    nand: { body: "and", negated: true },
    nor: { body: "or", negated: true },
    xnor: { body: "xor", negated: true },
} as const;

export function isGateKind(type: string): type is GateKind {
    return (gateKinds as readonly string[]).includes(type);
}

function makeGate(kind: GateKind): ComponentMetadata {
    return {
        displayName: kind.toUpperCase(),
        component: Gate,
        getDefaultDimensions: () => ({ width: 4, height: 4 }),
        getDefaultPorts() {
            return [
                { x: 0, y: 1, label: "0" },
                { x: 0, y: 3, label: "1" },
                { x: 4, y: 2, label: "Out" },
            ];
        },
    };
}

export const gateMetadata = Object.fromEntries(
    gateKinds.map((kind) => [kind, makeGate(kind)]),
) as Record<GateKind, ComponentMetadata>;
