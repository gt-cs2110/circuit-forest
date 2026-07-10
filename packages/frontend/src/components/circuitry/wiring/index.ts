import { ComponentMap } from "@/lib/types";
import { constant } from "./constant";
import { probe } from "./probe";

export const wiring = {
    constant,probe
} satisfies Partial<ComponentMap>;
