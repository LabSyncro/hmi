<script setup lang="ts">
import Toaster from "@/components/ui/toast/Toaster.vue";
import MainLayout from "@/layouts/MainLayout.vue";
import { computed } from "vue";
import { useRoute } from "vue-router";

const route = useRoute();

const layout = computed(() => {
  return route.meta.layout || MainLayout;
});
</script>

<template>
  <Toaster />
  <component :is="layout">
    <router-view />
  </component>
</template>

<style>
:root {
  font-family:
    Aeonik Pro,
    sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }
}
</style>
