import { ComponentMap } from "@/lib/types";
import { and } from "./and";
import { or } from "./or";

export const gates = {
    and,
    or,
} satisfies Partial<ComponentMap>;
