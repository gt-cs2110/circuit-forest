import { ComponentMap } from "@/lib/types";

import { gates } from "./gates";
import { wiring } from "./wiring";
import { selectors } from "./selectors";
import { arithmetic } from "./arithmetic";

export const componentMap: ComponentMap = {
    ...gates,
    ...wiring,
    ...selectors,
    ...arithmetic,
};
