<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'
import { ChevronDown } from 'lucide-vue-next'

interface NavChild {
  name: string;
  route: string;
  active?: boolean;
}

interface NavItem {
  name: string;
  route: string;
  active: boolean;
  children?: NavChild[];
}

const route = useRoute()

const openDropdown = ref<string | null>(null)

const navigationItems: Omit<NavItem, 'active'>[] = [
  {
    name: 'Mượn trả', route: '/',
    children: [
      { name: 'Ghi nhận', route: '/' },
      { name: 'Tổng quan', route: '/borrow-return' }
    ]
  },
  { name: 'Kiểm đếm', route: '/inventory' },
  { name: 'Sửa chữa', route: '/repair' },
  { name: 'Vận chuyển', route: '/transport' },
  { name: 'Tra cứu', route: '/device/:id' },
]

const activeNavItems = computed<NavItem[]>(() => {
  return navigationItems.map(item => ({
    ...item,
    active: (route.path.startsWith(item.route) && item.route !== '/') || (item.route === '/' && (route.path === '/' || route.path === '/borrow-return' || route.path === '/borrow-record' || route.path === '/return-record')),
    children: item.children?.map(child => ({
      ...child,
      active: (route.path.startsWith(child.route) && child.route !== '/') || (child.route === '/' && (route.path === '/' || route.path === '/borrow-record' || route.path === '/return-record'))
    }))
  }))
})

const userProfile = {
  avatar: "https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80"
}

const toggleDropdown = (name: string) => {
  openDropdown.value = openDropdown.value === name ? null : name
}

const closeAllDropdowns = () => {
  openDropdown.value = null
}

onMounted(() => {
  document.addEventListener('click', closeAllDropdowns)
})

onBeforeUnmount(() => {
  document.removeEventListener('click', closeAllDropdowns)
})
</script>

<template>
  <div class="w-full">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <img class="h-8 w-auto" src="/logo.svg" alt="Lab Syncro" />
        <span class="text-sm font-bold">Lab Syncro</span>
      </div>
      <div class="flex items-center space-x-1">
        <template v-for="(item, index) in activeNavItems" :key="index">
          <router-link v-if="!item.children" :to="item.route"
            class="px-4 py-2 text-sm font-semibold transition-colors rounded-md"
            :class="item.active ? 'bg-blue-50 text-blue-600' : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'"
            :aria-current="item.active ? 'page' : undefined">
            {{ item.name }}
          </router-link>

          <div v-else class="relative">
            <button @click.stop="toggleDropdown(item.name)"
              class="px-4 py-2 text-sm font-semibold transition-colors rounded-md flex items-center"
              :class="item.active ? 'bg-blue-50 text-blue-600' : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'">
              {{ item.name }}
              <ChevronDown class="ml-1 h-4 w-4 transition-transform duration-200"
                :class="openDropdown === item.name ? 'rotate-180' : ''" />
            </button>

            <div v-if="openDropdown === item.name"
              class="absolute left-0 mt-1 w-56 rounded-md bg-white border border-gray-200 shadow-lg z-10" @click.stop>
              <div class="py-1">
                <router-link v-for="child in item.children" :key="child.name" :to="child.route"
                  :class="(child.active ? 'bg-blue-50 text-blue-600' : 'text-gray-700 hover:bg-gray-100') + ' block px-4 py-2 text-sm'"
                  @click="openDropdown = null">
                  {{ child.name }}
                </router-link>
              </div>
            </div>
          </div>
        </template>
      </div>

      <div class="flex items-center">
        <div
          class="flex items-center px-3 py-1.5 rounded-full border border-gray-200 hover:bg-gray-50 transition-colors cursor-pointer">
          <img :src="userProfile.avatar" alt="User Avatar" class="h-8 w-8 rounded-full object-cover" />
        </div>
      </div>
    </div>
  </div>
</template>