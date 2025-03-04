<script setup lang="ts">
import { ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Bars3Icon, ChevronLeftIcon } from '@heroicons/vue/24/outline'
import Sidebar from './Sidebar.vue'

const route = useRoute()
const router = useRouter()
const sidebarOpen = ref(false)
</script> 

<template>
  <div class="min-h-screen bg-gray-100">
    <!-- Sidebar -->
    <Sidebar :is-open="sidebarOpen" @close="sidebarOpen = false" />

    <!-- Header -->
    <header class="bg-white shadow-sm">
      <div class="mx-auto px-4 sm:px-6 lg:px-8">
        <div class="flex h-16 items-center justify-between">
          <!-- Left side with menu button or back button -->
          <div class="flex items-center">
            <template v-if="route.name === 'device-detail'">
              <button
                type="button"
                class="text-gray-500 hover:text-gray-600 focus:outline-none flex items-center"
                @click="router.push('/')"
              >
                <ChevronLeftIcon class="h-6 w-6" />
                <span class="ml-2 text-lg font-medium">Thông tin thiết bị</span>
              </button>
            </template>
            <template v-else>
              <button
                type="button"
                class="text-gray-500 hover:text-gray-600 focus:outline-none"
                @click="sidebarOpen = true"
              >
                <span class="sr-only">Open sidebar</span>
                <Bars3Icon class="h-6 w-6" aria-hidden="true" />
              </button>
            </template>
          </div>

          <!-- Right side -->
          <div class="flex items-center space-x-4">
            <!-- Profile dropdown -->
            <div class="relative">
              <button
                type="button"
                class="flex rounded-full bg-white text-sm focus:outline-none"
              >
                <span class="sr-only">Open user menu</span>
                <img
                  class="h-8 w-8 rounded-full"
                  src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
                  alt=""
                />
              </button>
            </div>
          </div>
        </div>
      </div>
    </header>

    <!-- Main content -->
    <main>
      <div class="mx-auto max-w-7xl py-6 sm:px-6 lg:px-8">
        <router-view></router-view>
      </div>
    </main>
  </div>
</template>

