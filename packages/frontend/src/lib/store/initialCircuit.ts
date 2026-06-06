import type { Subcircuit, CircuitComponent } from "../types";

export function createTwoAndGateCircuit(): Map<number, Subcircuit> {
  const circuitKey = window.api.core.createCircuit("Test");

  const and1 = window.api.core.addComponent(
    circuitKey,
    "AND",
    1,
    2,
    2,
    "AND1",
    10,
    10,
    2
  );

  const and2 = window.api.core.addComponent(
    circuitKey,
    "AND",
    1,
    2,
    2,
    "AND2",
    20,
    10,
    2
  );
  console.log(window.api.core.getTransientState(BigInt(circuitKey)));


  const subcircuit: Subcircuit = {
    frontendId: 0,
    backendKey: circuitKey.toString(),
    name: "A AND B",
    components: new Map<number, CircuitComponent>([
      [
        1,
        {
          frontendId: 1,
          backendKey: and1.toString(),
          type: "and",
          label: "AND1",
          bitsize: 1,
          inputs: 2,
          ports: [],
          bounds: [
            { x: 10, y: 10 },
            { x: 13, y: 12 },
          ],
          orientation: 2,
          handedness: -1,
          labelOrientation: 2,
          x: 10,
          y: 10,
        },
      ],
      [
        2,
        {
          frontendId: 2,
          backendKey: and2.toString(),
          type: "and",
          label: "AND2",
          bitsize: 1,
          inputs: 2,
          ports: [],
          bounds: [
            { x: 20, y: 10 },
            { x: 23, y: 12 },
          ],
          orientation: 2,
          handedness: -1,
          labelOrientation: 2,
          x: 20,
          y: 10,
        },
      ],
    ]),
    wires: [],
  };

  return new Map<number, Subcircuit>([[0, subcircuit]]);
}