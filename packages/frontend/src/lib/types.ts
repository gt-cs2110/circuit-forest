import { Component } from "vue";

export const gateTypes = ["and", "nand", "or"] as const;
export const wiringTypes = ["constant"] as const;

export const componentCategories = {
    gates: gateTypes,
    wiring: wiringTypes,
};

export const componentTypes = [...gateTypes, ...wiringTypes];
export type ComponentType = (typeof componentTypes)[number];

export type Dimensions = { width: number; height: number };
export type Location = { x: number; y: number };
export type Port = Location & {
    label?: string;
};

export type ComponentMetadata = {
    displayName: string;
    component: Component;
    getDimensions: (component?: CircuitComponent) => Dimensions;
    getPorts: (component?: CircuitComponent) => Port[];
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
