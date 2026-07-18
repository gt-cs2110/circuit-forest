import type { Orientation } from "circuitsim-glue";
import type { CircuitComponent } from "./types";
import { GRID_SIZE } from "./consts";
import { getDimensions, intoDegrees, isOrientationHoriz } from "./bounds";

export function polyline(endpoints: ReadonlyArray<readonly [number, number]>): string {
    return [
        ...endpoints.map(([x, y], i) => `${i == 0 ? "M" : "L"} ${x * GRID_SIZE}, ${y * GRID_SIZE}`),
        "Z",
    ].join("\n");
}

export function rotateAround(orientation: Orientation, x: number, y: number) {
    return `rotate(${intoDegrees(orientation)} ${x} ${y})`;
}
export function rotateFromComponent(component?: CircuitComponent) {
    if (component) {
        const { orientation, bounds } = component;
        const dim = getDimensions(bounds);
        const mid_w = (dim.width * GRID_SIZE) / 2;
        const mid_h = (dim.height * GRID_SIZE) / 2;

        if (mid_w !== mid_h && !isOrientationHoriz(orientation)) {
            // For 90/270 deg rotations, the dimensions will not align, so we deal with it accordingly.
            return [
                `translate(${mid_w}, ${mid_h})`,
                rotateAround(orientation, 0, 0),
                `translate(${-mid_h}, ${-mid_w})`,
            ].join("\n");
        } else {
            return rotateAround(orientation, mid_w, mid_h);
        }
    }
}

export function orientedPath(
    pathGen: (main: number, cross: number) => ReadonlyArray<readonly [number, number]>,
    orientation: Orientation,
    width: number,
    height: number,
): ReadonlyArray<readonly [number, number]> {
    const horiz = isOrientationHoriz(orientation);
    const inverted = orientation === "North" || orientation === "West";

    let path = pathGen(...((horiz ? [width, height] : [height, width]) satisfies [number, number]));
    if (!horiz) {
        path = path.map<readonly [number, number]>(([m, c]) => [c, m]);
    }
    if (inverted) {
        path = path.map<readonly [number, number]>(([x, y]) => [width - x, height - y]);
    }
    return path;
}
export function trapezoid(orientation: Orientation, width: number, height: number) {
    return polyline(
        orientedPath(
            (main, cross) => [
                [0, 0],
                [main, 1],
                [main, cross - 1],
                [0, cross],
                [0, 0],
            ],
            orientation,
            width,
            height,
        ),
    );
}
