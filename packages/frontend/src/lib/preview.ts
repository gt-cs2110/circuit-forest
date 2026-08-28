import { Handedness, Location, Orientation } from "circuitsim-glue";
import { ComponentType } from "./types";
import { componentMap } from "@/components/circuitry";

//mirrors backend logic for rotating points aroudn origin 0,0
function orientDelta(
    point: Location,
    orientation: Orientation,
    handedness: Handedness,
    fromHandedness: Handedness = "DownRight",
): Location {
    const needsInitialFlip = fromHandedness !== "DownRight";

    const normalizedPoint = needsInitialFlip ? { x: point.x, y: -point.y } : point;

    const y =
        (orientation === "North" || orientation === "East") === (handedness === "DownRight")
            ? normalizedPoint.y
            : -normalizedPoint.y;

    if (orientation === "North") return { x: y, y: -normalizedPoint.x };
    if (orientation === "South") return { x: -y, y: normalizedPoint.x };
    if (orientation === "West") return { x: -normalizedPoint.x, y: -y };
    return { x: normalizedPoint.x, y };
}
export function getPreviewGeometry(
    type: ComponentType,
    orientation?: Orientation,
    handedness?: Handedness,
) {
    const metadata = componentMap[type];
    const dims = metadata.getDefaultDimensions();
    const defaultPorts = metadata.getDefaultPorts();

    const anchor = defaultPorts[defaultPorts.length - 1];

    const corners = [
        { x: 0, y: 0 },
        { x: dims.width, y: 0 },
        { x: 0, y: dims.height },
        { x: dims.width, y: dims.height },
    ];

    //computes corners coords relative to the anchor at 0,0 and then applies the appropriate transform
    const orientedCorners = corners.map((corner) =>
        orientDelta(
            {
                x: corner.x - anchor.x,
                y: corner.y - anchor.y,
            },
            orientation || "East",
            handedness || "DownRight",
            "DownRight",
        ),
    );

    const minX = Math.min(...orientedCorners.map((p) => p.x));
    const minY = Math.min(...orientedCorners.map((p) => p.y));
    const maxX = Math.max(...orientedCorners.map((p) => p.x));
    const maxY = Math.max(...orientedCorners.map((p) => p.y));

    const ports = defaultPorts.map((port) => {
        //convert ports to anchor relative coords and rotate them
        const oriented = orientDelta(
            {
                x: port.x - anchor.x,
                y: port.y - anchor.y,
            },
            orientation || "East",
            handedness || "DownRight",
            "DownRight",
        );
        //conver ports baxk to top left relative coords
        return {
            ...port,
            pos: {
                x: oriented.x - minX,
                y: oriented.y - minY,
            },
        };
    });

    return {
        bounds: [
            { x: 0, y: 0 },
            { x: maxX - minX, y: maxY - minY },
        ],
        ports,
        dimensions: {
            width: maxX - minX,
            height: maxY - minY,
        },
        anchorOffset: {
            x: -minX,
            y: -minY,
        },
    };
}
