import type { Dimensions } from "@/lib/types";
import type { Location } from "circuitsim-glue";

/**
 * Gets width and height from bounds.
 */
export const getDimensions = (bounds: [Location, Location]): Dimensions => ({
    width: bounds[1].x - bounds[0].x,
    height: bounds[1].y - bounds[0].y,
});
