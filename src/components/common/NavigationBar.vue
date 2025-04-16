<script setup lang="ts">
import { useAuth } from "@/composables";
import {
  ChevronDown,
  ClipboardCheck,
  LogOut,
  PackageCheck,
  Search,
  Truck,
  User,
  Wrench,
} from "lucide-vue-next";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useRoute } from "vue-router";

interface NavChild {
  name: string;
  route: string;
  active?: boolean;
}

interface NavItem {
  name: string;
  route?: string;
  icon?: any;
  active?: boolean;
  children?: NavChild[];
}

const route = useRoute();
const openDropdown = ref<string | null>(null);
const userDropdownOpen = ref(false);

const { getUserInfo, logout, labInfo } = useAuth();

const navigationItems: NavItem[] = [
  {
    name: "Mượn Trả",
    route: "/",
    icon: PackageCheck,
    children: [
      { name: "Ghi nhận", route: "/" },
      { name: "Tổng quan", route: "/borrow-return" },
    ],
  },
  {
    name: "Kiểm Đếm",
    route: "/audit",
    icon: ClipboardCheck,
  },
  {
    name: "Sửa Chữa",
    route: "/maintenance",
    icon: Wrench,
  },
  {
    name: "Vận Chuyển",
    route: "/transport",
    icon: Truck,
  },
  {
    name: "Tra cứu",
    route: "/search",
    icon: Search,
  },
];

const activeNavItems = computed<NavItem[]>(() => {
  return navigationItems.map((item) => ({
    ...item,
    active:
      (route.path.startsWith(item.route!) && item.route !== "/") ||
      (item.route === "/" &&
        (route.path === "/" || route.path === "/borrow-return")),
    children: item.children?.map((child) => ({
      ...child,
      active:
        (route.path.startsWith(child.route) && child.route !== "/") ||
        (child.route === "/" && route.path === "/"),
    })),
  }));
});

const userProfile = computed(() => {
  const userInfo = getUserInfo();
  return {
    name: userInfo?.name || "Người dùng",
    email: userInfo?.email || "",
    avatar: userInfo?.avatar || "",
    lab: userInfo?.lab || labInfo.value,
  };
});

const toggleDropdown = (name: string) => {
  openDropdown.value = openDropdown.value === name ? null : name;
};

const toggleUserDropdown = (event: Event) => {
  event.stopPropagation();
  userDropdownOpen.value = !userDropdownOpen.value;
};

const closeAllDropdowns = () => {
  openDropdown.value = null;
  userDropdownOpen.value = false;
};

const handleLogout = (event: Event) => {
  event.preventDefault();
  closeAllDropdowns();
  logout();
};

onMounted(() => {
  document.addEventListener("click", closeAllDropdowns);
});

onBeforeUnmount(() => {
  document.removeEventListener("click", closeAllDropdowns);
});
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
          <router-link
            v-if="!item.children"
            :to="item.route!"
            class="px-4 py-2 text-sm font-semibold transition-colors rounded-md flex items-center gap-2"
            :class="
              item.active
                ? 'bg-blue-50 text-blue-600'
                : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'
            "
            :aria-current="item.active ? 'page' : undefined"
          >
            <component :is="item.icon" class="h-4 w-4" />
            {{ item.name }}
          </router-link>

          <div v-else class="relative" @click.stop>
            <button
              @click="toggleDropdown(item.name)"
              class="px-4 py-2 text-sm font-semibold transition-colors rounded-md flex items-center gap-2 w-full text-left"
              :class="
                item.active
                  ? 'bg-blue-50 text-blue-600'
                  : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'
              "
            >
              <component :is="item.icon" class="h-4 w-4" />
              {{ item.name }}
              <ChevronDown
                class="h-4 w-4 ml-1 transition-transform"
                :class="{ 'rotate-180': openDropdown === item.name }"
              />
            </button>
            <transition
              enter-active-class="transition ease-out duration-100"
              enter-from-class="transform opacity-0 scale-95"
              enter-to-class="transform opacity-100 scale-100"
              leave-active-class="transition ease-in duration-75"
              leave-from-class="transform opacity-100 scale-100"
              leave-to-class="transform opacity-0 scale-95"
            >
              <div
                v-if="openDropdown === item.name && item.children"
                class="absolute z-10 mt-2 w-48 origin-top-left rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none left-0"
              >
                <router-link
                  v-for="child in item.children"
                  :key="child.route"
                  :to="child.route"
                  @click="closeAllDropdowns"
                  class="block px-4 py-2 text-sm w-full text-left"
                  :class="
                    child.active
                      ? 'bg-blue-50 text-blue-600'
                      : 'text-gray-600 hover:text-blue-600 hover:bg-gray-50'
                  "
                >
                  {{ child.name }}
                </router-link>
              </div>
            </transition>
          </div>
        </template>
      </div>

      <div class="flex items-center">
        <div class="relative" @click.stop>
          <div
            @click="toggleUserDropdown"
            class="flex items-center px-3 py-1.5 rounded-full border border-gray-200 hover:bg-gray-50 transition-colors cursor-pointer"
          >
            <img
              :src="userProfile.avatar"
              alt="User Avatar"
              class="h-8 w-8 rounded-full object-cover"
            />
            <ChevronDown
              class="h-4 w-4 ml-2 transition-transform"
              :class="{ 'rotate-180': userDropdownOpen }"
            />
          </div>

          <transition
            enter-active-class="transition ease-out duration-100"
            enter-from-class="transform opacity-0 scale-95"
            enter-to-class="transform opacity-100 scale-100"
            leave-active-class="transition ease-in duration-75"
            leave-from-class="transform opacity-100 scale-100"
            leave-to-class="transform opacity-0 scale-95"
          >
            <div
              v-if="userDropdownOpen"
              class="absolute z-10 mt-2 w-56 origin-top-right right-0 rounded-md bg-white py-1 shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none"
            >
              <div class="px-4 py-3 border-b border-gray-100">
                <p class="text-sm font-medium text-gray-900">
                  {{ userProfile.name }}
                </p>
                <p class="text-xs text-gray-500 truncate">
                  {{ userProfile.email }}
                </p>
                <p
                  v-if="userProfile.lab"
                  class="text-xs text-gray-500 mt-1 truncate"
                >
                  {{ userProfile.lab.name }}
                </p>
              </div>

              <a
                href="#"
                class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-50 flex items-center gap-2"
              >
                <User class="h-4 w-4" />
                Thông tin cá nhân
              </a>
              <a
                href="#"
                class="block px-4 py-2 text-sm text-red-600 hover:bg-gray-50 flex items-center gap-2"
                @click="handleLogout"
              >
                <LogOut class="h-4 w-4" />
                Đăng xuất
              </a>
            </div>
          </transition>
        </div>
      </div>
    </div>
  </div>
</template>
