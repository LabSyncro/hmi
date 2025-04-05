<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useRoute } from 'vue-router'
import { PackageCheck, ClipboardCheck, Wrench, Truck, ChevronDown, Search } from 'lucide-vue-next'

interface NavChild {
  name: string
  route: string
  active?: boolean
}

interface NavItem {
  name: string
  route?: string
  icon?: any
  active?: boolean
  children?: NavChild[]
}

const route = useRoute()
const openDropdown = ref<string | null>(null)

const navigationItems: NavItem[] = [
  {
    name: 'Mượn Trả',
    route: '/',
    icon: PackageCheck,
    children: [
      { name: 'Ghi nhận', route: '/' },
      { name: 'Tổng quan', route: '/borrow-return' }
    ]
  },
  {
    name: 'Kiểm Đếm',
    route: '/audit',
    icon: ClipboardCheck
  },
  {
    name: 'Sửa Chữa',
    route: '/maintenance',
    icon: Wrench
  },
  {
    name: 'Vận Chuyển',
    route: '/transport',
    icon: Truck
  },
  {
    name: 'Tra cứu',
    route: '/search',
    icon: Search
  }
]

const activeNavItems = computed<NavItem[]>(() => {
  return navigationItems.map(item => ({
    ...item,
    active: (route.path.startsWith(item.route!) && item.route !== '/') ||
      (item.route === '/' && (route.path === '/' || route.path === '/borrow-return')),
    children: item.children?.map(child => ({
      ...child,
      active: (route.path.startsWith(child.route) && child.route !== '/') ||
        (child.route === '/' && route.path === '/')
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
          <router-link v-if="!item.children" :to="item.route!"
            class="px-4 py-2 text-sm font-semibold transition-colors rounded-md flex items-center gap-2"
            :class="item.active ? 'bg-blue-50 text-blue-600' : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'"
            :aria-current="item.active ? 'page' : undefined">
            <component :is="item.icon" class="h-4 w-4" />
            {{ item.name }}
          </router-link>

          <div v-else class="relative" @click.stop>
            <button @click="toggleDropdown(item.name)"
              class="px-4 py-2 text-sm font-semibold transition-colors rounded-md flex items-center gap-2 w-full text-left"
              :class="item.active ? 'bg-blue-50 text-blue-600' : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'">
              <component :is="item.icon" class="h-4 w-4" />
              {{ item.name }}
              <ChevronDown class="h-4 w-4 ml-1 transition-transform"
                :class="{ 'rotate-180': openDropdown === item.name }" />
            </button>
            <transition enter-active-class="transition ease-out duration-100"
              enter-from-class="transform opacity-0 scale-95" enter-to-class="transform opacity-100 scale-100"
              leave-active-class="transition ease-in duration-75" leave-from-class="transform opacity-100 scale-100"
              leave-to-class="transform opacity-0 scale-95">
              <div v-if="openDropdown === item.name && item.children"
                class="absolute z-10 mt-2 w-48 origin-top-left rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none left-0">
                <router-link v-for="child in item.children" :key="child.route" :to="child.route"
                  @click="closeAllDropdowns" class="block px-4 py-2 text-sm w-full text-left"
                  :class="child.active ? 'bg-blue-50 text-blue-600' : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'">
                  {{ child.name }}
                </router-link>
              </div>
            </transition>
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