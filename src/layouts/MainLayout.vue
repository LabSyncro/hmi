<script setup lang="ts">
import { computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ChevronLeft } from 'lucide-vue-next'
import NavigationBar from '../components/common/NavigationBar.vue'

const route = useRoute()
const router = useRouter()

const showBackButton = computed(() => {
  const routesWithBack = [
    'device-detail',
    'device-borrow',
    'confirm-borrow',
    'borrow-invoice',
    'device-return',
    'confirm-return',
    'return-invoice',
    'borrow-record',
    'return-record'
  ]
  return routesWithBack.includes(route.name as string)
})

const handleBack = () => {
  const routeMap: Record<string, string> = {
    'device-borrow': `/device/${route.params.id}`,
    'confirm-borrow': `/device/${route.params.id}/borrow`,
    'borrow-invoice': '/',
    'device-return': `/device/${route.params.id}`,
    'confirm-return': `/device/${route.params.id}/return`,
    'return-invoice': '/',
    'borrow-record': '/',
    'return-record': '/'
  }

  router.push(routeMap[route.name as string] || '/')
}
</script>

<template>
  <div class="flex flex-col min-h-screen bg-gray-50">
    <header
      class="sticky top-0 z-40 w-full border-b border-gray-200/40 bg-white/95 backdrop-blur supports-[backdrop-filter]:bg-white/60 shadow-sm">
      <div class="mx-auto flex h-16 items-center justify-between px-4">
        <div class="flex items-center gap-2">
          <button v-if="showBackButton" @click="handleBack"
            class="inline-flex items-center justify-center rounded-md text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:opacity-50 disabled:pointer-events-none ring-offset-background h-10 w-10 mr-2 text-gray-700 hover:bg-gray-100"
            aria-label="Go back">
            <ChevronLeft class="h-5 w-5" />
            <span class="sr-only">Back</span>
          </button>
        </div>

        <NavigationBar />
      </div>
    </header>

    <main class="flex-1">
      <div class="mx-auto p-4">
        <router-view v-slot="{ Component }">
          <transition name="page" mode="out-in" enter-active-class="transition-all duration-300 ease-out"
            leave-active-class="transition-all duration-200 ease-in" enter-from-class="opacity-0 translate-y-4"
            enter-to-class="opacity-100 translate-y-0" leave-from-class="opacity-100 translate-y-0"
            leave-to-class="opacity-0 translate-y-4">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>
