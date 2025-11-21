import { ComponentMap } from "@/lib/types";

import { gates } from "./gates";
import { wiring } from "./wiring";

export const componentMap: ComponentMap = {
    ...gates,
    ...wiring,
};
