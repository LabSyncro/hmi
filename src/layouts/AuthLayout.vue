<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { AcceptableValue } from "reka-ui";
import { ref } from "vue";

const selectedLocation = ref("601 H6, Dĩ An");
const locations = ["601 H6, Dĩ An", "602 H6, Dĩ An", "603 H6, Dĩ An"];

const handleLocationChange = (value: AcceptableValue) => {
  selectedLocation.value = value as string;
};
</script>

<template>
  <div class="min-h-screen bg-[#060b28] text-white flex flex-col">
    <header class="p-4 flex justify-between items-center">
      <img class="h-8 w-auto" src="/full-logo.svg" alt="Lab Syncro" />

      <div class="flex items-center gap-2">
        <span>Địa điểm:</span>
        <Select
          :model-value="selectedLocation"
          @update:model-value="handleLocationChange"
        >
          <SelectTrigger
            class="w-[180px] bg-white text-black border border-gray-200 shadow-sm"
          >
            <SelectValue :placeholder="selectedLocation" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="location in locations"
              :key="location"
              :value="location"
            >
              {{ location }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
    </header>

    <main class="flex-1 flex flex-col">
      <slot />
    </main>
  </div>
</template>
