<script setup lang="ts">
import { useAuth } from "@/composables";

const { labInfo } = useAuth();
</script>

<template>
  <div class="min-h-screen bg-[#060b28] text-white flex flex-col">
    <header class="p-4 flex justify-between items-center">
      <img class="h-8 w-auto" src="/full-logo.svg" alt="Lab Syncro" />

      <div class="flex items-center gap-2">
        <span>Địa điểm:</span>
        <div
          class="w-fit bg-white text-black border border-gray-200 shadow-sm rounded-md px-3 py-2"
        >
          {{ labInfo?.name || "---" }}
        </div>
      </div>
    </header>

    <main class="flex-1 flex flex-col">
      <slot />
    </main>
  </div>
</template>
