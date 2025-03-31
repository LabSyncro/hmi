<script setup lang="ts">
import { type Component } from 'vue'

interface Props {
  title: string
  leftColumnTitle?: string
  rightColumnTitle?: string
  leftIcon?: Component
  rightIcon?: Component
}

defineProps<Props>()
</script>

<template>
  <div class="min-h-screen bg-gray-50">
    <!-- Fixed width container -->
    <div class="w-[1280px] mx-auto py-6">
      <!-- Page Title -->
      <h1 class="text-2xl font-bold text-center mb-6">{{ title }}</h1>
      
      <!-- Two Column Layout -->
      <div class="flex gap-6" style="height: calc(100vh - 140px)">
        <!-- Left Column - Device List -->
        <div class="w-[800px] bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
          <div class="p-4 border-b border-gray-200 flex items-center gap-2">
            <component :is="leftIcon" v-if="leftIcon" class="h-5 w-5 text-gray-700" />
            <h2 class="text-lg font-semibold">{{ leftColumnTitle }}</h2>
          </div>
          
          <!-- Left Column Content -->
          <div class="overflow-auto" style="height: calc(100% - 60px)">
            <slot name="left-column" />
          </div>
        </div>
        
        <!-- Right Column - User Info & Actions -->
        <div class="w-[440px] flex flex-col gap-6">
          <!-- Right Column Content -->
          <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
            <div class="p-4 border-b border-gray-200 flex items-center gap-2">
              <component :is="rightIcon" v-if="rightIcon" class="h-5 w-5 text-gray-700" />
              <h2 class="text-lg font-semibold">{{ rightColumnTitle }}</h2>
            </div>
            
            <div class="overflow-auto" style="height: calc(100% - 60px)">
              <slot name="right-column" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>