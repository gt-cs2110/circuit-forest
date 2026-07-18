import type { Dimensions } from "@/lib/types";
import type { Location, Orientation } from "circuitsim-glue";

/**
 * Gets width and height from bounds.
 */
export function getDimensions(bounds: [Location, Location]): Dimensions {
    return {
        width: bounds[1].x - bounds[0].x,
        height: bounds[1].y - bounds[0].y,
    };
}

export function isOrientationHoriz(orientation: Orientation): orientation is "East" | "West" {
    return orientation === "East" || orientation === "West";
}
export function invertOrientation(orientation: Orientation): Orientation {
    if (orientation === "East") return "West";
    if (orientation === "South") return "North";
    if (orientation === "West") return "East";
    if (orientation === "North") return "South";
    return "West";
}
export function intoDegrees(orientation: Orientation): number {
    if (orientation === "East") return 0;
    if (orientation === "South") return 90;
    if (orientation === "West") return 180;
    if (orientation === "North") return 270;
    return 0;
}
