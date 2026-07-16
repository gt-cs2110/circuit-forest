import { ComponentMap } from "@/lib/types";

import { gates } from "./gates";
import { wiring } from "./wiring";
import { selectors } from "./selectors";

export const componentMap: ComponentMap = {
    ...gates,
    ...wiring,
    ...selectors,
};
