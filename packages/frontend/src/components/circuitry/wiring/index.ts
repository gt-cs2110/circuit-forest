import { ComponentMap } from "@/lib/types";
import { constant } from "./constant";
import { probe } from "./probe";
import { tunnel } from "./tunnel";

export const wiring = {
    constant,
    probe,
    tunnel,
} satisfies Partial<ComponentMap>;
