type LegacyCircuitFile = {
    /**
     * CircuitSim version.
     * 
     * Currently accepts:
     * - 1.x.x
     * - 1.x.x 2110 version
     * - 1.10.0-CE
     * - 1.11.0-CE to 1.11.2-CE
     */
    version: string,

    /**
     * Global bit size (1-32), int
     */
    globalBitSize: number,

    /**
     * Clock speed, int
     */
    clockSpeed: number,

    /**
     * All defined circuits in this file.
     */
    circuits: CircuitInfo[],

    /**
     * A hash that keeps track of all updates to the file.
     * 
     * This is used to detect if there was any copied data between files.
     */
    revisionSignatures: string[]
}

type CircuitInfo = {
    /**
     * Name of the circuit
     */
    name: string,

    /**
     * Components in circuit.
     */
    components: ComponentInfo[],

    /**
     * Wires in circuit.
     */
    wires: WireInfo[],
}

type ComponentInfo = {
    /**
     * Component type.
     */
    name: string,

    /**
     * X position of component, int
     */
    x: number,

    /**
     * Y position of component, int
     */
    y: number,

    /**
     * Properties of component.
     */
    properties: Record<string, string>
}

type WireInfo = {
    /**
     * X position of wire, int
     */
    x: number,

    /**
     * Y position of wire, int
     */
    y: number,

    /**
     * Length of wire, int
     */
    length: number,

    /**
     * Whether the wire is horizontal or vertical.
     */
    isHorizontal: boolean
}