import { Component } from "vue";

export type ComponentType = "and" | "or" | "constant";

export type Dimensions = { width: number; height: number };

export type ComponentMetadata = {
    component: Component;
    getDimensions: (component: CircuitComponent) => Dimensions;
};

export type ComponentMap = Record<ComponentType, ComponentMetadata>;

export type CircuitComponent = {
    id: number;
    type: ComponentType;
    name: string;
    bitsize: number;
    x: number;
    y: number;
};
