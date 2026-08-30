import type { useDrag } from "@/composables/useDrag";
import type {
    TransientComponentState,
    TransientWireState,
    Location,
    Key,
    Orientation,
    Handedness,
    SignType,
} from "circuitsim-glue";
import { Component } from "vue";

export const componentCategories = {
    wiring: ["constant", "probe", "tunnel"],
    gates: ["and", "nand", "or", "nor", "xor", "xnor", "not", "buffer"],
    plexers: ["mux", "demux", "decoder"],
    arithmetic: ["adder", "subtractor", "divider", "multiplier"],
} as const;
export type ComponentType = (typeof componentCategories)[keyof typeof componentCategories][number];
export const componentTypes = [
    ...componentCategories.wiring,
    ...componentCategories.gates,
    ...componentCategories.plexers,
] as const satisfies readonly ComponentType[];

export type Dimensions = { width: number; height: number };

export type Port = Location & {
    label?: string;
    value?: string;
    issues?: string[];
};

export type CircuitComponentProps = {
    component?: CircuitComponent;
};

export type ComponentMetadata = {
    displayName: string;
    component: Component<CircuitComponentProps>;
    getDefaultDimensions: () => Dimensions;
    getDefaultPorts: () => Port[]; //Default Ports are only used for preview visualization before a component is created
};

export type ComponentMap = Record<string, ComponentMetadata>;

export type CircuitComponent = 
    TransientComponentState & {
        pos: Location;
        frontendId: number;
        type: string;
        label: string;
        bitsize: number;
        selsize: number;
        isInput: boolean; //For Pins
        textContent: string; //For textboxes
        constantValue: string;
        signType: SignType;

        inputs: number;
        orientation: Orientation;
        handedness: Handedness;
        labelOrientation: Orientation;
    };

export type WireDirection = "H" | "V";
export type Wire = TransientWireState; // FIXME: Wire should not be using this type

export type Subcircuit = {
    frontendId: number;
    backendKey: Key;
    name: string;
    components: Map<number, CircuitComponent>;
    wires: Wire[];
};

export type DragState = ReturnType<typeof useDrag>["drag"];

// FIXME: This needs to use `ComponentType[]`, not `string[]` for types
//Component Types gate, not, buffer, constant, pin, splitter, power ground, tunnel, probe, mux, demux, decode, text subcircuit
const propertyGroups = [
    {
        types: ["and", "nand", "nor", "or", "xnor", "xor"],
        properties: ["label", "label_orientation", "orientation", "bitsize", "inputs"],
    },
    {
        types: ["demux", "mux"],
        properties: [
            "label",
            "label_orientation",
            "orientation",
            "handedness",
            "bitsize",
            "selsize",
        ],
    },
    {
        types: ["buffer", "splitter"],
        properties: ["label", "label_orientation", "orientation", "handedness", "bitsize"],
    },
    {
        types: ["not", "pin", "probe", "tunnel", "adder", "subtractor"],
        properties: ["label", "label_orientation", "orientation", "bitsize"],
    },
    {
        types: ["constant"],
        properties: ["label", "label_orientation", "orientation", "bitsize", "constantValue"],
    },
    {
        types: ["decoder"],
        properties: ["label", "label_orientation", "orientation", "handedness", "selsize"],
    },
    {
        types: ["ground", "power", "subcircuit", "tunnel"],
        properties: ["label", "label_orientation", "orientation"],
    },
    {
        types: ["text"],
        properties: [],
    },
     {
        types: ["divider", "multiplier"],
        properties: ["label", "label_orientation", "orientation", "bitsize", "signedness"],
    },
];

//Build a lookup map
export const componentPropertiesMap: Record<string, string[]> = {};

propertyGroups.forEach((group) => {
    group.types.forEach((type) => {
        componentPropertiesMap[type] = group.properties;
    });
});
