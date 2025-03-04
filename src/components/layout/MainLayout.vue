<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Bars3Icon, ChevronLeftIcon } from '@heroicons/vue/24/outline'
import Sidebar from './Sidebar.vue'

const route = useRoute()
const router = useRouter()
const sidebarOpen = ref(false)

const showBackButton = computed(() => {
  return route.name === 'device-detail' || route.name === 'device-borrow'
})

const headerTitle = computed(() => {
  if (route.name === 'device-detail') {
    return 'Thông tin thiết bị'
  } else if (route.name === 'device-borrow') {
    return 'Ghi nhận mượn'
  }
  return ''
})

const handleBack = () => {
  if (route.name === 'device-borrow') {
    router.push(`/device/${route.params.id}`)
  } else {
    router.push('/')
  }
}
</script>

<template>
  <div class="min-h-screen bg-gray-100">
    <!-- Sidebar -->
    <Sidebar :is-open="sidebarOpen" @close="sidebarOpen = false" />

    <!-- Header -->
    <header class="bg-white shadow-sm">
      <div class="flex h-14 items-center justify-between px-4 border-b">
        <div class="flex items-center">
          <button v-if="showBackButton" @click="handleBack" class="flex items-center text-gray-900 hover:text-gray-600">
            <ChevronLeftIcon class="h-6 w-6" />
            <span class="ml-2 text-lg">{{ headerTitle }}</span>
          </button>
          <button v-else @click="sidebarOpen = true" class="text-gray-500 hover:text-gray-600">
            <Bars3Icon class="h-6 w-6" />
          </button>
        </div>

        <!-- Profile dropdown -->
        <div class="flex items-center">
          <button class="flex items-center">
            <img class="h-8 w-8 rounded-full"
              src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
              alt="" />
          </button>
        </div>
      </div>
    </header>

    <!-- Main content -->
    <main>
      <div class="mx-auto max-w-7xl">
        <router-view></router-view>
      </div>
    </main>
  </div>
</template>
