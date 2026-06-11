import { TransientComponentState } from "circuitsim-glue";
import { Component } from "vue";

export const gateTypes = ["and", "nand", "or", "nor", "xor", "xnor", "not", "buffer"] as const;
export const wiringTypes = ["constant"] as const;

export const componentCategories = {
    wiring: wiringTypes,
    gates: gateTypes,
};
export type Dimensions = { width: number; height: number };

export const componentTypes = [...gateTypes, ...wiringTypes];
export type ComponentType = (typeof componentTypes)[number];
export const Orientation = {
    "NORTH":0,
    
    "SOUTH":1,
    "EAST":2,
    "WEST":3
}
export const  Handedness = {
    "N/A":0,
    "TOPLEFT":0,
    "BOTTOMRIGHT":1
    
}

export type Location = { x: number; y: number };
export type Port = Location & {
    label?: string;
    value?: string;
};

export type ComponentMetadata = {
    displayName: string;
    component: Component;
    getDimensions: (component?: CircuitComponent) => Dimensions;
    getDefaultPorts: (component?: CircuitComponent) => Port[];//Default Ports are only used for preview visualization before a component is created

};

export type ComponentMap = Record<string, ComponentMetadata>;


export type CircuitComponent = Location & TransientComponentState &{
    frontendId:number;
    type: string;
    label: string;
    bitsize: number;
    selsize:number;
    isInput:boolean;//For Pins
    textContent:string;//For textboxes
    constantValue:string;
    

    inputs:number;
    orientation:number;
    handedness:number;
    labelOrientation:number;
   
};



export type WireDirection = "H" | "V";
export type Wire = Location & {
    direction: WireDirection;
    length: number;
};

export type Subcircuit = {
    frontendId: number;
    backendKey:string;
    name: string;
    components: Map<number, CircuitComponent>;
    wires: Wire[];
};
