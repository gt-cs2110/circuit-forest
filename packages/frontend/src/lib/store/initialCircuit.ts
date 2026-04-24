// testing layout file. to be removed

import { Subcircuit } from "../types";

export const initialCircuit: Map<string, Subcircuit> = new Map([
    [
        "circuit1",
        {
            id: "circuit1",
            name: "Main Circuit",
            components: new Map([
                [
                    1,
                    {
                        id: 1,
                        type: "nand",
                        x: 1,
                        y: 1,
                        label: "Component A",
                        bitsize: 1,
                    },
                ],
                [
                    2,
                    {
                        id: 2,
                        type: "constant",
                        x: 6,
                        y: 7,
                        label: "Component B",
                        bitsize: 1,
                    },
                ],
                [
                    3,
                    {
                        id: 3,
                        type: "or",
                        x: 17,
                        y: 9,
                        label: "Component C",
                        bitsize: 1,
                    },
                ],
            ]),
            wires: [{ x: 5, y: 3, direction: "H", length: 5 }],
        },
    ],
    [
        "circuit2",
        {
            id: "circuit2",
            name: "Second Circuit",
            components: new Map([
                [
                    1,
                    {
                        id: 1,
                        type: "and",
                        x: 1,
                        y: 10,
                        label: "Component A",
                        bitsize: 1,
                    },
                ],
                [
                    2,
                    {
                        id: 2,
                        type: "or",
                        x: 7,
                        y: 6,
                        label: "Component B",
                        bitsize: 1,
                    },
                ],
                [
                    3,
                    {
                        id: 3,
                        type: "constant",
                        x: 9,
                        y: 13,
                        label: "Component C",
                        bitsize: 1,
                    },
                ],
            ]),
            wires: [],
        },
    ],
]);
