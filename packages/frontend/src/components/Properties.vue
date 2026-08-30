<script setup lang="ts">
import { Handedness, Orientation } from "circuitsim-glue";
import { computed, onMounted, ref, watchEffect } from "vue";
import { toast } from "vue-sonner";

import { circuits, currentSubcircuit, updateComponents } from "@/lib/store/circuit";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import { settings } from "@/lib/store/settings";
import {
    componentSelection,
    placingComponent,
    placingHandedness,
    placingOrientation,
} from "@/lib/store/view";
import { CircuitComponent, componentPropertiesMap } from "@/lib/types";

const nameReset = ref(0);
const subcircuitName = computed({
    get() {
        return currentSubcircuit.value.name;
    },
    set(name) {
        if (name === "") {
            toast.error("Subcircuit name is required!");
            nameReset.value++;
            return;
        } else if (Array.from(circuits.value.values()).some(([subcircuit]) => subcircuit.name === name)) {
            toast.error("A subcircuit with this name already exists!");
            nameReset.value++;
            return;
        }
        currentSubcircuit.value.name = name;
    },
});

// FIXME: These two consts is redundant and should be removed.
const orientations = [
    { label: "N", value: "North" },
    { label: "S", value: "South" },
    { label: "E", value: "East" },
    { label: "W", value: "West" },
] as const satisfies { label: string; value: Orientation }[];
const handednesses = [
    { label: "TOP/LEFT", value: "TopLeft" },
    { label: "BOTTOM/RIGHT", value: "DownRight" },
] as const satisfies { label: string; value: Handedness }[];

const selectedComponents = computed<CircuitComponent[]>(() =>
    [...componentSelection.value].map((id) => currentSubcircuit.value.components.get(id)!),
);
// Get properties defined by all selected components.
const selectedProperties = computed(() => {
    if (selectedComponents.value.length == 0) return new Set();

    return selectedComponents.value
        .map((c) => new Set(getComponentProps(c)))
        .reduce((acc, cv) => acc.intersection(cv));
});

const sections = ["global", "circuit", "component"] as const;

/**
 * Gets a value from all selected components, returning it if all components have the same value.
 * Otherwise, this returns the fallback value.
 */
function getFromSelected<T>(query: (c: CircuitComponent) => T, fallback: T) {
    let queries = selectedComponents.value.map((c) => query(c));
    return queries.every((t) => queries[0] == t) ? queries[0] : fallback;
}
/** Updates all selected components with the specified updates. */
function updateAllSelected(updates: Partial<CircuitComponent>) {
    updateComponents(
        selectedComponents.value.map((c) => c.frontendId),
        updates,
    );
}
/** Gets all defined properties for a given component. */
function getComponentProps(component: CircuitComponent) {
    // FIXME: Please use an enum here instead of string[]
    return componentPropertiesMap[component.type.toLowerCase()];
}

// TODO: Allow null or "invalid state" cases
//    Should show up when not all inputs are the same
const labelInput = computed({
    get: () => getFromSelected((c) => c.label, ""),
    set: (label) => updateAllSelected({ label }),
});
const bitsizeInput = ref(1);
const selsizeInput = ref(1);
const nInputsInput = ref(1);
watchEffect(() => {
    bitsizeInput.value = getFromSelected((c) => c.bitsize, 1);
    selsizeInput.value = getFromSelected((c) => c.selsize, 1);
    nInputsInput.value = getFromSelected((c) => c.inputs, 1);
});

const constantError = ref("");
const constantValue = {
    dec: computed({
        get: () => {
            let bin = getFromSelected((c) => c.constantValue, "0");
            return /^[01]+$/.test(bin) ? parseInt(bin, 2) : "?";
        },
        set: (value) => {
            constantError.value = "";
            const n = +value;
            if (!Number.isInteger(n) || n < 0 || isNaN(n)) {
                constantError.value = "Enter a non-negative integer";
                return;
            }

            let bin = n.toString(2).padStart(bitsizeInput.value, "0").slice(-bitsizeInput.value);
            updateAllSelected({ constantValue: bin });
        },
    }),
    bin: computed({
        get: () => getFromSelected((c) => c.constantValue, "0"),
        set: (value) => {
            constantError.value = "";
            if (!/^[01]+$/.test(value)) {
                constantError.value = "Binary digits only (0 and 1)";
                return;
            }

            let bin = value.slice(-bitsizeInput.value);
            updateAllSelected({ constantValue: bin });
        },
    }),
};

onMounted(() =>
    document.addEventListener("keydown", (e) => {
        if (placingComponent.value) {
            if (e.key == "ArrowUp") {
                placingOrientation.value = "North";
            }
            if (e.key == "ArrowLeft") {
                placingOrientation.value = "West";
            }
            if (e.key == "ArrowRight") {
                placingOrientation.value = "East";
            }
            if (e.key == "ArrowDown") {
                placingOrientation.value = "South";
            }
            if (e.key.toLowerCase() == "h") {
                placingHandedness.value =
                    placingHandedness.value == "TopLeft" ? "DownRight" : "TopLeft";
            }
        } else {
            if (selectedProperties.value.has("orientation")) {
                if (e.key == "ArrowUp") {
                    updateAllSelected({ orientation: "North" });
                }
                if (e.key == "ArrowLeft") {
                    updateAllSelected({ orientation: "West" });
                }
                if (e.key == "ArrowRight") {
                    updateAllSelected({ orientation: "East" });
                }
                if (e.key == "ArrowDown") {
                    updateAllSelected({ orientation: "South" });
                }
            }
            if (selectedProperties.value.has("handedness")) {
                if (e.key.toLowerCase() == "h") {
                    if (selectedComponents.value[0].handedness == "TopLeft") {
                        updateAllSelected({ handedness: "DownRight" });
                    } else {
                        updateAllSelected({ handedness: "TopLeft" });
                    }
                }
            }
        }
    }),
);
</script>

<template>
    <h2 class="border-b bg-panel-light px-4 py-3 text-sm font-semibold text-foreground-highlight">
        Properties
    </h2>

    <AccordionRoot :default-value="sections.slice()">
        <AccordionItem value="global">
            <AccordionHeader> Global </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Global Bitsize</span>
                        <span>{{ settings.globalBitsize }}</span>
                    </span>
                    <input v-model="settings.globalBitsize" type="range" min="1" step="1" max="16"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500" />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem value="circuit">
            <AccordionHeader> {{ currentSubcircuit.name }} </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between font-medium"> Name </span>
                    <input :key="nameReset" v-model.lazy.trim="subcircuitName" type="text" min="1" step="1" max="16"
                        @keydown.stop
                        class="mt-1 block w-full appearance-none border bg-panel-light px-1 py-1 accent-blue-500" />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem v-if="selectedComponents.length != 0 && selectedComponents[0] != undefined" value="component">
            <AccordionHeader>
                {{
                    selectedComponents.length > 1
                        ? "Component Group"
                        : selectedComponents[0].label != ""
                            ? selectedComponents[0].label
                            : selectedComponents[0].type.toUpperCase()
                }}
            </AccordionHeader>

            <!-- LABEL -->
            <AccordionContent v-if="
                selectedComponents.length == 1 &&
                getComponentProps(selectedComponents[0]).includes('label')
            " class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between font-medium">Label</span>
                    <input v-model.lazy.trim="labelInput" type="text" placeholder="Enter label..." @keydown.stop />

                    <h2 class="font-medium">Label Orientation</h2>

                    <!-- FIXME: Missing accessibility labels -->
                    <div class="mt-2 flex overflow-hidden rounded border">
                        <button v-for="option in orientations" :key="option.value" type="button"
                            class="flex-1 px-3 py-2 transition-colors" :class="selectedComponents[0].labelOrientation === option.value
                                ? 'bg-blue-500 text-white'
                                : 'bg-panel-light hover:bg-panel-dark'
                                " @click="updateAllSelected({ labelOrientation: option.value })">
                            {{ option.label }}
                        </button>
                    </div>
                </label>
            </AccordionContent>

            <!-- BITSIZE -->
            <AccordionContent v-if="selectedProperties.has('bitsize')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Bitsize</span>
                        <span>{{ bitsizeInput }}</span>
                    </span>
                    <input v-model.number="bitsizeInput" type="range" :min="1" :step="1" :max="64"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                        @change="updateAllSelected({ bitsize: bitsizeInput })" @keydown.stop />

                </label>
            </AccordionContent>

            <!-- SELSIZE -->
            <AccordionContent v-if="selectedProperties.has('selsize')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium">Selector Bits</span>
                        <span>{{ selsizeInput }}</span>
                    </span>
                    <input v-model.number="selsizeInput" type="range" :min="1" :step="1" :max="6"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                        @change="updateAllSelected({ selsize: selsizeInput })" @keydown.stop />
                </label>
            </AccordionContent>
            <!-- INPUTS -->
            <AccordionContent v-if="selectedProperties.has('inputs')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Num Inputs</span>
                        <span>{{ nInputsInput }}</span>
                    </span>
                    <input v-model.number="nInputsInput" type="range" :min="1" :step="1" :max="8"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                        @change="updateAllSelected({ inputs: nInputsInput })" @keydown.stop />
                </label>
            </AccordionContent>
            <!-- ORIENTATION -->
            <AccordionContent v-if="selectedProperties.has('orientation')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Orientation</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                        <!-- FIXME: Missing accessibility labels -->
                        <button v-for="option in orientations" :key="option.value" type="button"
                            class="flex-1 px-3 py-2 transition-colors" :class="selectedComponents[0].orientation === option.value
                                ? 'bg-blue-500 text-white'
                                : 'bg-panel-light hover:bg-panel-dark'
                                " @click="updateAllSelected({ orientation: option.value })">
                            {{ option.label }}
                        </button>
                    </div>
                </label>
            </AccordionContent>

            <!-- HANDEDNESS -->
            <AccordionContent v-if="selectedProperties.has('handedness')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Handedness</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                        <!-- FIXME: Missing accessibility labels -->
                        <button v-for="option in handednesses" :key="option.value" type="button"
                            class="flex-1 px-3 py-2 transition-colors" :class="selectedComponents[0].handedness === option.value
                                ? 'bg-blue-500 text-white'
                                : 'bg-panel-light hover:bg-panel-dark'
                                " @click="updateAllSelected({ handedness: option.value })">
                            {{ option.label }}
                        </button>
                    </div>
                </label>
            </AccordionContent>
            <!-- CONSTANT VALUE -->
            <AccordionContent v-if="selectedProperties.has('constantValue')" class="px-4 py-3 text-xs">
                <label class="block space-y-3">
                    <span class="font-medium">Value</span>

                    <div>
                        <span class="flex justify-between font-medium">Decimal</span>
                        <input v-model.lazy.trim="constantValue.dec.value" type="text" class="font-mono"
                            @keydown.stop />
                    </div>
                    <div>
                        <span class="flex justify-between font-medium">Binary</span>
                        <input v-model.lazy.trim="constantValue.bin.value" type="text" class="font-mono"
                            @keydown.stop />
                    </div>
                    <span v-if="constantError" class="text-xs text-red-500">{{
                        constantError
                    }}</span>
                </label>
            </AccordionContent>
        </AccordionItem>
    </AccordionRoot>
</template>
