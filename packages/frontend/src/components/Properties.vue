<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { circuits, currentSubcircuit, updateComponent } from "@/lib/store/circuit";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import { toast } from "vue-sonner";
import { settings } from "@/lib/store/settings";
import { selection } from "@/lib/store/view";

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
        } else if ([...circuits.values()].some((subcircuit) => subcircuit.name === name)) {
            toast.error("A subcircuit with this name already exists!");
            nameReset.value++;
            return;
        }
        currentSubcircuit.value.name = name;
    },
});


const orientations = [
  { label: "N", value: 0 },
  { label: "S", value: 1 },
  { label: "E", value: 2 },
  { label: "W", value: 3 },
] as const;
const handidnesses = [
  { label: "TOP/LEFT", value: 0 },
  { label: "BOTTOM/RIGHT", value: 1 },
 
] as const;

const selectedComponent = computed(() =>
    selection.value.values().next().value?currentSubcircuit.value.components.get(selection.value.values().next().value||-1):null
);

const sections = ["global", "circuit", "component"] as const;


//CONSTANT INPUT LOGIC
const constantError = ref("");
// const constantValue = ref({binaryValue:selectedComponent.value!.constantValue.padStart(selectedComponent.value!.bitsize,"0"),decimalValue:parseInt(selectedComponent.value!.constantValue,2).toString()})
const constantValue = ref({binaryValue:"0", decimalValue:"0"})
watch(()=>selectedComponent, (comp)=>{
    constantValue.value = {binaryValue:selectedComponent.value!.constantValue.padStart(selectedComponent.value!.bitsize,"0"),decimalValue:parseInt(selectedComponent.value!.constantValue,2).toString()}
    // constantValue.value.binaryValue = constantValue.value!.binaryValue.padStart(newBitsize,"0")
    // updateComponent(selectedComponent.value!.frontendId, { componentValue: constantValue.value.binaryValue });
})
watch(()=>selectedComponent.value?.bitsize, (newBitsize)=>
{
   constantValue.value.binaryValue = constantValue.value!.binaryValue.slice(-newBitsize!).padStart(newBitsize!,"0")
    updateComponent(selectedComponent.value!.frontendId, { componentValue: constantValue.value.binaryValue }); 
})

function onDecimalInput(e: Event) {
    const val = (e.target as HTMLInputElement).value.trim();
    constantError.value = ""
    if (!val) {
        constantValue.value.binaryValue="";
        constantValue.value.decimalValue=""
        return;
    }
    const n = Number(val);
    if (!Number.isInteger(n) || n < 0 || isNaN(n)) {
        constantError.value = "Enter a non-negative integer";
        return;
    }
    if (n.toString(2).length > selectedComponent.value!.bitsize){//exceeds max bit size
        constantError.value = "Value exceeds current bitsize"
        
        return;
    }
        console.log(val)

    constantValue.value={ binaryValue:n.toString(2).padStart(selectedComponent.value!.bitsize,"0"), decimalValue:val}
    updateComponent(selectedComponent.value!.frontendId, { componentValue: constantValue.value.binaryValue });
}
function onBinaryInput(e: Event) {
    const val = (e.target as HTMLInputElement).value.trim();
        constantError.value = ""
    if (!val) {
       constantValue.value.binaryValue="";
        constantValue.value.decimalValue=""
        return;
    }
    if (!/^[01]+$/.test(val) ) {
        constantError.value = "Binary digits only (0 and 1)";
        
        return;
    }
     if (val.length > selectedComponent.value!.bitsize){//exceeds max bit size
        constantError.value = "Value exceeds current bitsize"
        return;
    }
    const n = parseInt(val, 2);
    constantValue.value={ binaryValue:val.padStart(selectedComponent.value!.bitsize,"0"), decimalValue:String(n)}
    console.log(val)

    updateComponent(selectedComponent.value!.frontendId, { componentValue: constantValue.value.binaryValue });
}


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
                        v-model="settings.globalBitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

        <AccordionItem value="circuit">
            <AccordionHeader> {{ currentSubcircuit.name }} </AccordionHeader>

            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between font-medium"> Name </span>
                    <input
                        :key="nameReset"
                        v-model.lazy.trim="subcircuitName"
                        type="text"
                        min="1"
                        step="1"
                        max="16"
                        class="mt-1 block w-full appearance-none border bg-panel-light px-1 py-1 accent-blue-500"
                    />
                </label>
            </AccordionContent>
        </AccordionItem>

         <AccordionItem v-if="selectedComponent !== null && selectedComponent !=undefined" value="component">
            <AccordionHeader>
                {{ selectedComponent.label !=""?selectedComponent.label:selectedComponent.type.toUpperCase()}}
            </AccordionHeader>

            <!-- LABEL -->
             <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between font-medium"> Label </span>
                    <input
                        v-model.lazy.trim="selectedComponent.label"
                        type="text"
                       placeholder="Enter label..."
                       @keydown.stop
                        @change="updateComponent(selectedComponent.frontendId, { label: selectedComponent.label })"
                        
                    />
                </label>
            </AccordionContent>
<!-- LABEL ORIENTATION -->
            <AccordionContent v-if="selectedComponent.label!=''"class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Label Orientation</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in orientations"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponent.orientation === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="
                        updateComponent(selectedComponent.frontendId, {
                            labelOrientation: option.value,
                        })
                        "
                    >
                        {{ option.label }}
                    </button>
                    </div>
                </label>
            </AccordionContent>
            <!-- BITSIZE -->
            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Bitsize</span>
                        <span>{{ selectedComponent.bitsize }}</span>
                    </span>
                    <input
                        v-model.number="selectedComponent.bitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        @change = "updateComponent(selectedComponent.frontendId, { bitsize: selectedComponent.bitsize })"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <!--SELSIZE-->
            <AccordionContent v-if= "selectedComponent.type == 'MUX'||selectedComponent.type == 'DEMUX'" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> SelectorBits</span>
                        <span>{{ selectedComponent.selsize }}</span>
                    </span>
                    <input
                        v-model.number="selectedComponent.selsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        @change = "updateComponent(selectedComponent.frontendId, { selsize: selectedComponent.selsize })"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <!-- INPUTS -->
            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Num Inputs</span>
                        <span>{{ selectedComponent.inputs }}</span>
                    </span>
                    <input
                        v-model.number="selectedComponent.inputs"
                        type="range"
                        min="1"
                        step="1"
                        max="8"
                        @change = "updateComponent(selectedComponent.frontendId, { inputs: selectedComponent.inputs })"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <!-- ORIENTATION -->
            <AccordionContent class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Orientation</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in orientations"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponent.orientation === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="
                        updateComponent(selectedComponent.frontendId, {
                            orientation: option.value,
                        })
                        "
                    >
                        {{ option.label }}
                    </button>
                    </div>
                </label>
            </AccordionContent>
            
            <!-- HANDIDNESS -->
            <AccordionContent v-if = "selectedComponent.type.toUpperCase()=='BUFFER'" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Handedness</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in handidnesses"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponent.handedness === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="
                        updateComponent(selectedComponent.frontendId, {
                            handedness: option.value,
                        })
                        "
                    >
                        {{ option.label }}
                    </button>
                    </div>
                </label>
            </AccordionContent>
            <!-- CONSTANT VALUE -->
            <AccordionContent v-if="selectedComponent.type.toUpperCase() == 'CONSTANT'" class="px-4 py-3 text-xs">
                <label class="block space-y-3">
                    <span class="font-medium">Value</span>

                    <div>
                        <span class="flex justify-between font-medium">Decimal</span>
                        <input
                            :value="constantValue.decimalValue"
                            type="text"
                            :placeholder="constantValue.decimalValue"
                            class="font-mono"
                            @keydown.stop
                            @input="onDecimalInput"
                        />
                    </div>
                    <div>
                        <span class="flex justify-between font-medium">Binary</span>
                        <input
                            :value="constantValue.binaryValue"
                            type="text"
                            :placeholder="constantValue.binaryValue"
                            class="font-mono"
                            @keydown.stop
                            @change="onBinaryInput"
                        />
                    </div>
                                                                <span v-if="constantError" class="text-xs text-red-500">{{ constantError }}</span>

                </label>
            </AccordionContent>
        </AccordionItem>
        
    </AccordionRoot>
</template>
