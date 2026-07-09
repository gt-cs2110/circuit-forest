import { TransientComponentState, TransientWireState, Location, Key } from "circuitsim-glue";
import { Component } from "vue";

export const componentCategories = {
    wiring: ["constant"],
    gates: ["and", "nand", "or", "nor", "xor", "xnor", "not", "buffer"],
    plexers: ["mux", "demux", "decoder"],
} as const;
export type ComponentType = (typeof componentCategories)[keyof typeof componentCategories][number];
export const componentTypes = [
    ...componentCategories.wiring,
    ...componentCategories.gates,
    ...componentCategories.plexers,
] as const satisfies readonly ComponentType[];

export type Dimensions = { width: number; height: number };
export const Orientation = {
    NORTH: 0,

    SOUTH: 1,
    EAST: 2,
    WEST: 3,
};
export const Handedness = {
    "N/A": 0,
    TOPLEFT: 0,
    BOTTOMRIGHT: 1,
};

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
    getDimensions: (component: CircuitComponent) => Dimensions;
    getDefaultPorts: () => Port[]; //Default Ports are only used for preview visualization before a component is created
    getOriginToFixedPortOffset: (component: CircuitComponent) => { x: number; y: number }; //All components have a fixed port that they rotate around, generally the output port but can vary; we need the offset from the origin of the svg drawing to the output port to shift everything properly in Cricuit COmponet rendering
};

export type ComponentMap = Record<string, ComponentMetadata>;

export type CircuitComponent = Location &
    TransientComponentState & {
        frontendId: number;
        type: string;
        label: string;
        bitsize: number;
        selsize: number;
        isInput: boolean; //For Pins
        textContent: string; //For textboxes
        constantValue: string;

        inputs: number;
        orientation: number;
        handedness: number;
        labelOrientation: number;
    };

export type WireDirection = "H" | "V";
export type Wire = TransientWireState;

export type Subcircuit = {
    frontendId: number;
    backendKey: Key;
    name: string;
    components: Map<number, CircuitComponent>;
    wires: Wire[];
};

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
        types: ["not", "pin", "probe", "tunnel"],
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
        types: ["ground", "power", "subcircuit"],
        properties: ["label", "label_orientation", "orientation"],
    },
    {
        types: ["text"],
        properties: [],
    },
];

//Build a lookup map
export const componentPropertiesMap: Record<string, string[]> = {};

propertyGroups.forEach((group) => {
    group.types.forEach((type) => {
        componentPropertiesMap[type] = group.properties;
    });
});
