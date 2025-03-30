<script setup lang="ts">
import { Dialog, DialogContent, DialogClose } from '@/components/ui/dialog'
import { useRoute } from 'vue-router'
import { X, PieChart, Hand, Calculator, Truck, Wrench } from 'lucide-vue-next'

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const route = useRoute()

const closeDrawer = () => {
  emit('close')
}
</script>

<template>
  <Dialog :open="isOpen" @update:open="closeDrawer" class="relative z-50">
    <div v-if="isOpen" class="fixed inset-0 overflow-hidden">
      <div class="absolute inset-0 overflow-hidden">
        <div class="pointer-events-none fixed inset-y-0 left-0 flex max-w-full">
          <DialogContent :class="[
            'pointer-events-auto w-screen max-w-xs transform transition-all duration-300 ease-in-out',
            isOpen ? 'translate-x-0' : '-translate-x-full'
          ]">
            <div class="absolute right-0 top-0 flex pt-4 pr-2">
              <DialogClose
                class="rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2">
                <X class="h-6 w-6 text-gray-600" />
                <span class="sr-only">Close</span>
              </DialogClose>
            </div>

            <div class="flex h-full flex-col overflow-y-auto bg-white py-6 shadow-xl">
              <div class="flex h-16 shrink-0 items-center px-6 bg-blue-500 -mt-6">
                <img class="h-8 w-auto" src="/logo.png" alt="Lab Syncro" />
              </div>

              <nav class="flex flex-1 flex-col px-6">
                <ul role="list" class="flex flex-1 flex-col gap-y-7">
                  <li>
                    <div class="text-xs font-semibold leading-6 text-gray-400">TỔNG QUAN</div>
                    <ul role="list" class="-mx-2 mt-2 space-y-1">
                      <li>
                        <router-link to="/"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                          :class="{ 'text-blue-600 bg-gray-50': route.name === 'home' }">
                          <PieChart class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Dashboard
                        </router-link>
                      </li>
                    </ul>
                  </li>

                  <li>
                    <div class="text-xs font-semibold leading-6 text-gray-400">VẬN HÀNH</div>
                    <ul role="list" class="-mx-2 mt-2 space-y-1">
                      <li>
                        <router-link to="/borrow-return"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                          :class="{ 'text-blue-600 bg-gray-50': route.name === 'borrow-return' }">
                          <Hand class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Mượn trả
                        </router-link>
                      </li>
                      <li>
                        <a href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50">
                          <Calculator class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Kiểm đếm
                        </a>
                      </li>
                      <li>
                        <a href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50">
                          <Truck class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Vận chuyển
                        </a>
                      </li>
                      <li>
                        <a href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50">
                          <Wrench class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Sửa chữa
                        </a>
                      </li>
                    </ul>
                  </li>
                </ul>
              </nav>
            </div>
          </DialogContent>
        </div>
      </div>
    </div>
  </Dialog>
</template>