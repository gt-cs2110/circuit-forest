import { ComponentMap } from "@/lib/types";
import { and } from "./and";
import { nand } from "./nand";
import { or } from "./or";

export const gates = {
    and,
    nand,
    or,
} satisfies Partial<ComponentMap>;
