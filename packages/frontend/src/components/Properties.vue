<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { circuits, currentSubcircuit, updateComponent } from "@/lib/store/circuit";
import { AccordionContent, AccordionHeader, AccordionItem, AccordionRoot } from "./ui/accordion";
import { toast } from "vue-sonner";
import { settings } from "@/lib/store/settings";
import { selection } from "@/lib/store/view";
import { componentPropertiesMap } from "@/lib/types";

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

const selectedComponents = computed(() =>
    [...selection.value].map((id)=>currentSubcircuit.value.components.get(id))
);

const sections = ["global", "circuit", "component"] as const;


//CONSTANT INPUT LOGIC
const constantError = ref("");
const constantValue = ref({binaryValue:"0", decimalValue:"0"})

//Once selectedComponents renders in, we'll set the default binary and decimal values in the constant accordion selector to be the constant value associated with the first constant
watch(()=>selectedComponents, (_)=>{
    if (!selectedComponents.value || selectedComponents.value.length ==0){
        return;
    }
    constantValue.value = {binaryValue:selectedComponents.value[0]!.constantValue.padStart(selectedComponents.value[0]!.bitsize,"0"),decimalValue:parseInt(selectedComponents.value[0]!.constantValue,2).toString()}
    
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
    if (n.toString(2).length > selectedComponents.value[0]!.bitsize){//exceeds max bit size
        constantError.value = "Value exceeds current bitsize"
        
        return;
    }
        console.log(val)

    constantValue.value={ binaryValue:n.toString(2).padStart(selectedComponents.value[0]!.bitsize,"0"), decimalValue:val}

 selectedComponents.value.forEach(component=>{
    if(!component)return;
    updateComponent(component.frontendId, {componentValue:constantValue.value.binaryValue})
   });
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
     if (val.length > selectedComponents.value[0]!.bitsize){//exceeds max bit size
        constantError.value = "Value exceeds current bitsize"
        return;
    }
    const n = parseInt(val, 2);
    constantValue.value={ binaryValue:val.padStart(selectedComponents.value[0]!.bitsize,"0"), decimalValue:String(n)}
    console.log(val)

     selectedComponents.value.forEach(component=>{
    if(!component)return;
    updateComponent(component.frontendId, {componentValue:constantValue.value.binaryValue})
   });
}

const label = computed(()=>selectedComponents.value.length == 0?"":selectedComponents.value[0]?.label)

//If we have more than one componet selected, we will choose to display the property modifier which al=pply to all of them

const properties = computed(() => {
    if (!selectedComponents.value[0]) return [];
    let props = componentPropertiesMap[selectedComponents.value[0].type.toLowerCase()];
    selectedComponents.value.forEach(comp => {
        if (!comp) return;
        props = props.filter(prop => componentPropertiesMap[comp.type.toLowerCase()].includes(prop));
    });
    return props;
});
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

         <AccordionItem v-if="selectedComponents.length !=0 &&selectedComponents[0]!=undefined" value="component">
            <AccordionHeader>

                {{ selectedComponents.length>1?"Component Group":(selectedComponents[0].label !=""?selectedComponents[0].label:selectedComponents[0].type.toUpperCase())}}
            </AccordionHeader>

            <!-- LABEL -->
             <AccordionContent v-if="selectedComponents.length==1 && componentPropertiesMap[selectedComponents[0].type.toLowerCase()].includes('label')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between font-medium"> Label </span>
                    <input
                        
                       :value="0"
                        type="text"
                       placeholder="Enter label..."
                       @keydown.stop
                        @change="updateComponent(selectedComponents[0].frontendId, { label: label })"
                        
                    />
                    
                    <h2 class="font-medium">Label Orientation</h2>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in orientations"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponents[0].labelOrientation === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="
                        updateComponent(selectedComponents[0].frontendId, {
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
            <AccordionContent v-if="properties.includes('bitsize')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Bitsize</span>
                        <span>{{ selectedComponents[0].bitsize }}</span>
                    </span>
                    <input
                        :value="selectedComponents[0].bitsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        @change = "(e) => {
                            let bitsize =  Number((e.target as HTMLInputElement).value)
                            
                        constantValue.binaryValue = constantValue.binaryValue.slice(-bitsize!).padStart(bitsize!,'0');
                        selectedComponents.forEach(comp=>{
                            if(!comp)return;
                                updateComponent(comp.frontendId, {componentValue:constantValue.binaryValue, bitsize: Number((e.target as HTMLInputElement).value)})
                        });
                        }"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>

            <!-- SELSIZE -->
            <AccordionContent v-if= "properties.includes('selsize')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> SelectorBits</span>
                        <span>{{ selectedComponents[0].selsize }}</span>
                    </span>
                    <input
                        :value="selectedComponents[0].selsize"
                        type="range"
                        min="1"
                        step="1"
                        max="16"
                        @change = " selectedComponents.forEach(comp=>{
                            if(!comp)return;
                             updateComponent(comp!.frontendId, { selsize: Number(($event.target as HTMLInputElement).value) })
                        });
                        "

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <!-- INPUTS -->
            <AccordionContent v-if = "properties.includes('inputs')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="flex justify-between">
                        <span class="font-medium"> Num Inputs</span>
                        <span>{{ selectedComponents[0].inputs }}</span>
                    </span>
                    <input
                        :value="selectedComponents[0].inputs"
                        type="range"
                        min="1"
                        step="1"
                        max="8"
                        @change = "selectedComponents.forEach(comp=>{if(!comp){return;}updateComponent(comp.frontendId, { inputs: Number(($event.target as HTMLInputElement).value) })})"

                        class="mt-3 mb-1 block h-1 w-full appearance-none rounded border bg-panel-light accent-blue-500"
                    />
                </label>
            </AccordionContent>
            <!-- ORIENTATION -->
            <AccordionContent v-if = "properties.includes('orientation')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Orientation</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in orientations"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponents[0].orientation === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="selectedComponents.forEach(comp=>{if(!comp){return;}updateComponent(comp.frontendId, { orientation: Number(option.value) })})"
                    >
                        {{ option.label }}
                    </button>
                    </div>
                </label>
            </AccordionContent>
            
            <!-- HANDIDNESS -->
            <AccordionContent v-if = "properties.includes('buffer')" class="px-4 py-3 text-xs">
                <label class="block">
                    <span class="font-medium">Handedness</span>

                    <div class="mt-2 flex overflow-hidden rounded border">
                    <button
                        v-for="option in handidnesses"
                        :key="option.value"
                        type="button"
                        class="flex-1 px-3 py-2 transition-colors"
                        :class="
                        selectedComponents[0].handedness === option.value
                            ? 'bg-blue-500 text-white'
                            : 'bg-panel-light hover:bg-panel-dark'
                        "
                        @click="selectedComponents.forEach(comp=>{if(!comp){return;}updateComponent(comp.frontendId, { handedness: Number(($event.target as HTMLInputElement).value) })})"
                    >
                        {{ option.label }}
                    </button>
                    </div>
                </label>
            </AccordionContent>
            <!-- CONSTANT VALUE -->
            <AccordionContent v-if="properties.includes('constantValue')" class="px-4 py-3 text-xs">
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
