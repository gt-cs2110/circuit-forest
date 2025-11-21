import { Component } from "vue";

export type ComponentType = "and" | "or" | "constant";

export type Dimensions = { width: number; height: number };
export type Location = { x: number; y: number };
export type Port = Location & {
    label?: string;
};

export type ComponentMetadata = {
    component: Component;
    getDimensions: (component: CircuitComponent) => Dimensions;
    getPorts: (component: CircuitComponent) => Port[];
};

export type ComponentMap = Record<ComponentType, ComponentMetadata>;

export type CircuitComponent = Location & {
    id: number;
    type: ComponentType;
    name: string;
    bitsize: number;
};

export type WireDirection = "H" | "V";
export type Wire = Location & {
    direction: WireDirection;
    length: number;
};

export type Subcircuit = {
    name: string;
    components: Map<number, CircuitComponent>;
    wires: Wire[];
};
