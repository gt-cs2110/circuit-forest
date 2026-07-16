/** Gets the wire color from the bit array. */
export function wireColor(value: string) {
    // FIXME: Add colors for different bitsizes
    if (value.length == 0) return "rgb(127, 127, 127)";
    if (value.includes("X")) return "rgb(255,0,0)";
    if (value.includes("1")) return "rgb(0,255,0)";
    if (value.includes("0")) return "#006400";
    return "rgb(0,0,255)";
}
