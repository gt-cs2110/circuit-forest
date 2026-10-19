import type { Location } from "circuitsim-glue";
import type { CircuitComponent, Wire } from "./types";

/** Gets the wire color from the bit array. */
export function wireColor(value: string) {
    // FIXME: Add colors for different bitsizes
    if (value.length == 0) return "rgb(127, 127, 127)";
    if (value.includes("X")) return "rgb(255,0,0)";
    if (value.includes("1")) return "rgb(0,255,0)";
    if (value.includes("0")) return "#006400";
    return "rgb(0,0,255)";
}

/**
 * Wires can't be identified by their `backendKey` (shared by every wire in the same net)
 * or by index (changes whenever the backend state is refreshed), so a wire's ID is based on position.
 */
export function wireId([a, b]: readonly [Location, Location]) {
    return `${a.x},${a.y}:${b.x},${b.y}`;
}

/**
 * Returns points with 3+ wire or component endpoints to render dots that indicate intersection.
 */
export function getJunctions(wires: readonly Wire[], components: Iterable<CircuitComponent>) {
    const junctions = new Map<string, { pos: Location; value: string; count: number }>();
    function add(pos: Location, value: string) {
        const key = `${pos.x},${pos.y}`;
        const junction = junctions.get(key);
        if (junction) junction.count++;
        else junctions.set(key, { pos, value, count: 1 });
    }

    for (const wire of wires) {
        for (const pos of wire.endpoints) {
            add(pos, wire.value);
        }
    }
    for (const component of components) {
        for (const port of component.ports) {
            add(port.pos, port.value);
        }
    }
    return [...junctions].filter(([, { count }]) => count >= 3);
}
