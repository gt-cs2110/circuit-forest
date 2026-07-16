import { ComponentMap } from "@/lib/types";
import { constant } from "./constant";
import { probe } from "./probe";
import { tunnel } from "./tunnel";
import { splitter } from "./splitter";

export const wiring = {
    constant,
    probe,
    tunnel,
    splitter,
} satisfies Partial<ComponentMap>;
