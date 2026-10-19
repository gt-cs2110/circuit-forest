<script setup lang="ts">
import type { Handedness, Orientation } from "circuitsim-glue";
import { computed, onMounted, onUnmounted, ref, useId, watch } from "vue";
import { toast } from "vue-sonner";

import { circuits, currentSubcircuit, updateComponents } from "@/lib/store/circuit";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import CommitInput from "./ui/CommitInput.vue";
import CommitSlider from "./ui/CommitSlider.vue";
import SegmentedControl from "./ui/SegmentedControl.vue";
import { isEditableTarget } from "@/lib/keyboard";
import { settings } from "@/lib/store/settings";
import {
    componentSelection,
    placingComponent,
    placingHandedness,
    placingOrientation,
} from "@/lib/store/view";
import type { CircuitComponent, Subcircuit } from "@/lib/types";
import { componentPropertiesMap } from "@/lib/types";
import { componentMap } from "./circuitry";
import { gateKinds, type GateKind } from "./circuitry/gates/gate";

function renameSubcircuit(name: string, subcircuit: Subcircuit) {
    if (name === subcircuit.name) return;

    if (name === "") {
        toast.error("Subcircuit name is required!");
        return;
    } else if (
        [...circuits.value.values()].some(
            (other) => other.frontendId !== subcircuit.frontendId && other.name === name,
        )
    ) {
        toast.error("A subcircuit with this name already exists!");
        return;
    }
    subcircuit.name = name;
}

// FIXME: These two consts is redundant and should be removed.
const orientations = [
    { label: "N", name: "North", value: "North" },
    { label: "E", name: "East", value: "East" },
    { label: "S", name: "South", value: "South" },
    { label: "W", name: "West", value: "West" },
] as const satisfies { label: string; name: string; value: Orientation }[];
const handednesses = [
    { label: "TOP/LEFT", name: "Top/Left", value: "TopLeft" },
    { label: "BOTTOM/RIGHT", name: "Bottom/Right", value: "DownRight" },
] as const satisfies { label: string; name: string; value: Handedness }[];

const gateFunctions = gateKinds.map((kind) => ({
    label: componentMap[kind].displayName,
    name: componentMap[kind].displayName,
    value: kind,
}));

const functionId = useId();
const labelOrientationId = useId();
const orientationId = useId();
const handednessId = useId();

const selectedComponents = computed<CircuitComponent[]>(() =>
    [...componentSelection.value].map((id) => currentSubcircuit.value.components.get(id)!),
);
// Get properties defined by all selected components.
const selectedProperties = computed(() => {
    if (selectedComponents.value.length == 0) return new Set<string>();
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

/**
 * Keys allow us to update values even when the input element is unmounted because of some selection.
 * There used to be the issue where if you updated a text value like a label and then clicked on another component or unfocused on the canvas,
 * the value wouldn't update, because the input would unmount or correspond to different component(s).
 * The key makes it so that the inputs unmount and re-mount and forces the CommitInputs to emit a `commit` event.
 * It's a relatively contrived solution but it does resolve the problem.
 */
type EditTarget = { key: string; subcircuit: Subcircuit; componentIds: number[] };
const editTarget = computed<EditTarget>(() => {
    const subcircuit = currentSubcircuit.value;
    const componentIds = [...componentSelection.value];
    return { key: `${subcircuit.frontendId}:${componentIds.join()}`, subcircuit, componentIds };
});
function updateTarget(target: EditTarget, updates: Partial<CircuitComponent>) {
    updateComponents(target.componentIds, updates, target.subcircuit);
}

/** Gets all defined properties for a given component. */
function getComponentProps(component: CircuitComponent) {
    // FIXME: Please use an enum here instead of string[]
    return componentPropertiesMap[component.type.toLowerCase()] ?? [];
}

// TODO: Allow null or "invalid state" cases
//    Should show up when not all inputs are the same
const labelValue = computed(() => getFromSelected((c) => c.label, ""));
const bitsizeValue = computed(() => getFromSelected((c) => c.bitsize, 1));
const selsizeValue = computed(() => getFromSelected((c) => c.selsize, 1));
const inputsValue = computed(() => getFromSelected((c) => c.inputs, 1));

// Mixed values across the selection show no active option.
const labelOrientationValue = computed(() =>
    getFromSelected<Orientation | undefined>((c) => c.labelOrientation, undefined),
);
const orientationValue = computed(() =>
    getFromSelected<Orientation | undefined>((c) => c.orientation, undefined),
);
const handednessValue = computed(() =>
    getFromSelected<Handedness | undefined>((c) => c.handedness, undefined),
);
const functionValue = computed(() =>
    getFromSelected<GateKind | undefined>((c) => c.type.toLowerCase() as GateKind, undefined),
);

const constantError = ref("");
watch(
    () => [...componentSelection.value].join(),
    () => (constantError.value = ""),
);
const constantValue = computed(() => getFromSelected((c) => c.constantValue, "0"));
// BigInt keeps values above 53 bits exact.
const constantDecimal = computed(() =>
    /^[01]+$/.test(constantValue.value) ? BigInt("0b" + constantValue.value).toString() : "?",
);
function setConstantValue(target: EditTarget, bin: string | null, error: string) {
    if (target.key === editTarget.value.key) constantError.value = bin === null ? error : "";
    if (bin !== null) updateTarget(target, { constantValue: bin });
}
function setConstantDecimal(value: string, target: EditTarget) {
    const bin = /^\d+$/.test(value) ? BigInt(value).toString(2) : null;
    setConstantValue(target, bin, "Enter a non-negative integer");
}
function setConstantBinary(value: string, target: EditTarget) {
    setConstantValue(target, /^[01]+$/.test(value) ? value : null, "Binary digits only (0 and 1)");
}

const arrowOrientations: Record<string, Orientation> = {
    ArrowUp: "North",
    ArrowLeft: "West",
    ArrowRight: "East",
    ArrowDown: "South",
};

function handleKeyDown(e: KeyboardEvent) {
    if (isEditableTarget(e.target) || e.metaKey || e.ctrlKey || e.altKey) return;

    const orientation = arrowOrientations[e.key];
    const isHandednessKey = e.key.toLowerCase() == "h";

    if (placingComponent.value) {
        if (orientation) {
            e.preventDefault();
            placingOrientation.value = orientation;
        }
        if (isHandednessKey) {
            placingHandedness.value =
                placingHandedness.value == "TopLeft" ? "DownRight" : "TopLeft";
        }
    } else {
        if (orientation && selectedProperties.value.has("orientation")) {
            e.preventDefault();
            updateAllSelected({ orientation });
        }
        if (isHandednessKey && selectedProperties.value.has("handedness")) {
            if (selectedComponents.value[0].handedness == "TopLeft") {
                updateAllSelected({ handedness: "DownRight" });
            } else {
                updateAllSelected({ handedness: "TopLeft" });
            }
        }
    }
}

onMounted(() => document.addEventListener("keydown", handleKeyDown));
onUnmounted(() => document.removeEventListener("keydown", handleKeyDown));
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
                    <input
                        v-model.number="settings.globalBitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-primary"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem value="circuit">
            <AccordionHeader> {{ currentSubcircuit.name }} </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Name</span>
                    <CommitInput
                        :key="currentSubcircuit.frontendId"
                        :value="currentSubcircuit.name"
                        :target="currentSubcircuit"
                        class="mt-1 block w-full border bg-panel-light px-1 py-1"
                        @commit="renameSubcircuit"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem
            v-if="selectedComponents.length != 0 && selectedComponents[0] != undefined"
            value="component"
        >
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
            <AccordionContent
                v-if="
                    selectedComponents.length == 1 &&
                    getComponentProps(selectedComponents[0]).includes('label')
                "
                class="space-y-3 px-4 py-3 text-xs"
            >
                <label class="block">
                    <span class="font-medium">Label</span>
                    <CommitInput
                        :key="editTarget.key"
                        :value="labelValue"
                        :target="editTarget"
                        placeholder="Enter label..."
                        class="mt-1 block w-full border bg-panel-light px-1 py-1 placeholder:text-foreground-muted"
                        @commit="(label, target) => updateTarget(target, { label })"
                    />
                </label>

                <div>
                    <span :id="labelOrientationId" class="font-medium">Label Orientation</span>
                    <SegmentedControl
                        :options="orientations"
                        :value="labelOrientationValue"
                        :labelledby="labelOrientationId"
                        @update:value="updateAllSelected({ labelOrientation: $event })"
                    />
                </div>
            </AccordionContent>

            <!-- GATE FUNCTION -->
            <AccordionContent v-if="selectedProperties.has('function')" class="px-4 py-3 text-xs">
                <span :id="functionId" class="font-medium">Function</span>
                <SegmentedControl
                    :options="gateFunctions"
                    :value="functionValue"
                    :labelledby="functionId"
                    :columns="3"
                    @update:value="updateAllSelected({ type: $event })"
                />
            </AccordionContent>

            <!-- BITSIZE -->
            <AccordionContent v-if="selectedProperties.has('bitsize')" class="px-4 py-3 text-xs">
                <CommitSlider
                    label="Bitsize"
                    :value="bitsizeValue"
                    :min="1"
                    :max="64"
                    @commit="updateAllSelected({ bitsize: $event })"
                />
            </AccordionContent>

            <!-- SELSIZE -->
            <AccordionContent v-if="selectedProperties.has('selsize')" class="px-4 py-3 text-xs">
                <CommitSlider
                    label="Selector Bits"
                    :value="selsizeValue"
                    :min="1"
                    :max="6"
                    @commit="updateAllSelected({ selsize: $event })"
                />
            </AccordionContent>
            <!-- INPUTS -->
            <AccordionContent v-if="selectedProperties.has('inputs')" class="px-4 py-3 text-xs">
                <CommitSlider
                    label="Num Inputs"
                    :value="inputsValue"
                    :min="1"
                    :max="8"
                    @commit="updateAllSelected({ inputs: $event })"
                />
            </AccordionContent>
            <!-- ORIENTATION -->
            <AccordionContent
                v-if="selectedProperties.has('orientation')"
                class="px-4 py-3 text-xs"
            >
                <span :id="orientationId" class="font-medium">Orientation</span>
                <SegmentedControl
                    :options="orientations"
                    :value="orientationValue"
                    :labelledby="orientationId"
                    @update:value="updateAllSelected({ orientation: $event })"
                />
            </AccordionContent>

            <!-- HANDEDNESS -->
            <AccordionContent v-if="selectedProperties.has('handedness')" class="px-4 py-3 text-xs">
                <span :id="handednessId" class="font-medium">Handedness</span>
                <SegmentedControl
                    :options="handednesses"
                    :value="handednessValue"
                    :labelledby="handednessId"
                    @update:value="updateAllSelected({ handedness: $event })"
                />
            </AccordionContent>
            <!-- CONSTANT VALUE -->
            <AccordionContent
                v-if="selectedProperties.has('constantValue')"
                class="px-4 py-3 text-xs"
            >
                <fieldset class="space-y-3">
                    <legend class="font-medium">Value</legend>

                    <label class="block">
                        <span class="font-medium">Decimal</span>
                        <CommitInput
                            :key="editTarget.key"
                            :value="constantDecimal"
                            :target="editTarget"
                            inputmode="numeric"
                            class="mt-1 block w-full border bg-panel-light px-1 py-1 font-mono"
                            @commit="setConstantDecimal"
                        />
                    </label>
                    <label class="block">
                        <span class="font-medium">Binary</span>
                        <CommitInput
                            :key="editTarget.key"
                            :value="constantValue"
                            :target="editTarget"
                            class="mt-1 block w-full border bg-panel-light px-1 py-1 font-mono"
                            @commit="setConstantBinary"
                        />
                    </label>
                    <p v-if="constantError" class="text-xs text-error">
                        {{ constantError }}
                    </p>
                </fieldset>
            </AccordionContent>
        </AccordionItem>
    </AccordionRoot>
</template>
