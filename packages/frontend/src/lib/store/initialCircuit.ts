import type { Subcircuit, CircuitComponent } from "../types";

export function createTwoAndGateCircuit(): Map<number, Subcircuit> {
  const circuitKey = window.api.core.createCircuit("Test");

  const subcircuit: Subcircuit = {
    frontendId: 0,
    backendKey: circuitKey.toString(),
    name: "A AND B",
    components: new Map<number, CircuitComponent>(),
    wires: [],
  };

  return new Map<number, Subcircuit>([[0, subcircuit]]);
}