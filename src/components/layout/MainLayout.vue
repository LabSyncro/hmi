<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { Menu, ChevronLeft } from 'lucide-vue-next'
import Sidebar from './Sidebar.vue'

const route = useRoute()
const router = useRouter()
const sidebarOpen = ref(false)

const showBackButton = computed(() => {
  const routesWithBack = [
    'device-detail',
    'device-borrow',
    'confirm-borrow',
    'borrow-invoice',
    'device-return',
    'confirm-return',
    'return-invoice'
  ]
  return routesWithBack.includes(route.name as string)
})

const headerTitle = computed(() => {
  const titles: Record<string, string> = {
    'device-detail': 'Thông tin thiết bị',
    'device-borrow': 'Ghi nhận mượn',
    'confirm-borrow': 'Xác nhận mượn',
    'borrow-invoice': 'Thông tin đơn mượn',
    'device-return': 'Ghi nhận trả',
    'confirm-return': 'Xác nhận trả',
    'return-invoice': 'Thông tin đơn trả',
    'borrow-return': 'Mượn trả thiết bị'
  }
  return titles[route.name as string] || ''
})

const handleBack = () => {
  const routeMap: Record<string, string> = {
    'device-borrow': `/device/${route.params.id}`,
    'confirm-borrow': `/device/${route.params.id}/borrow`,
    'borrow-invoice': '/',
    'device-return': `/device/${route.params.id}`,
    'confirm-return': `/device/${route.params.id}/return`,
    'return-invoice': '/'
  }

  router.push(routeMap[route.name as string] || '/')
}

watch(route, () => {
  sidebarOpen.value = false
})
</script>

<template>
  <div class="min-h-screen bg-gray-100">
    <Sidebar :is-open="sidebarOpen" @close="() => sidebarOpen = false" />

    <header class="sticky top-0 z-10 bg-white shadow-sm">
      <div class="flex h-14 items-center justify-between px-4 border-b">
        <div class="flex items-center">
          <button v-if="showBackButton" @click="handleBack"
            class="flex items-center text-gray-900 hover:text-gray-600 transition-colors">
            <ChevronLeft class="h-6 w-6" />
            <span class="ml-2 text-lg font-medium">{{ headerTitle }}</span>
          </button>
          <button v-else @click="() => sidebarOpen = true" class="text-gray-500 hover:text-gray-600 transition-colors">
            <Menu class="h-6 w-6" />
          </button>
        </div>

        <div class="flex items-center">
          <button class="flex items-center rounded-full overflow-hidden hover:opacity-80 transition-opacity">
            <img class="h-8 w-8 rounded-full object-cover"
              src="https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
              alt="User avatar" />
          </button>
        </div>
      </div>
    </header>

    <main>
      <div class="mx-auto max-w-7xl p-4">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
