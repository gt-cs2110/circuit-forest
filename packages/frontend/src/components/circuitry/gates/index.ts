import { ComponentMap } from "@/lib/types";
import { and } from "./and";
import { nand } from "./nand";
import { or } from "./or";
import { nor } from "./nor";
import { xor } from "./xor";
import { xnor } from "./xnor";
import { not } from "./not";
import { buffer } from "./buffer";

export const gates = {
    and,
    nand,
    or,
    nor,
    xor,
    xnor,
    not,
    buffer,
} satisfies Partial<ComponentMap>;
