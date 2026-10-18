import type { Location } from "circuitsim-glue";
import type { Wire } from "./types";

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
 * Returns points with 3+ endpoints to render dots that indicate intersection.
 */
export function getJunctions(wires: readonly Wire[]) {
    const junctions = new Map<string, { pos: Location; value: string; count: number }>();
    for (const wire of wires) {
        for (const pos of wire.endpoints) {
            const key = `${pos.x},${pos.y}`;
            const junction = junctions.get(key);
            if (junction) junction.count++;
            else junctions.set(key, { pos, value: wire.value, count: 1 });
        }
    }
    return [...junctions].filter(([, { count }]) => count >= 3);
}
