<template>
  <TransitionRoot as="template" :show="isOpen">
    <Dialog as="div" class="relative z-50" @close="closeDrawer">
      <TransitionChild
        as="template"
        enter="transition-opacity ease-linear duration-300"
        enter-from="opacity-0"
        enter-to="opacity-100"
        leave="transition-opacity ease-linear duration-300"
        leave-from="opacity-100"
        leave-to="opacity-0"
      >
        <div class="fixed inset-0 bg-gray-900/80" />
      </TransitionChild>

      <div class="fixed inset-0 flex">
        <TransitionChild
          as="template"
          enter="transition ease-in-out duration-300 transform"
          enter-from="-translate-x-full"
          enter-to="translate-x-0"
          leave="transition ease-in-out duration-300 transform"
          leave-from="translate-x-0"
          leave-to="-translate-x-full"
        >
          <DialogPanel class="relative mr-16 flex w-full max-w-xs flex-1">
            <div class="absolute left-full top-0 flex w-16 justify-center pt-5">
              <button type="button" class="-m-2.5 p-2.5" @click="closeDrawer">
                <span class="sr-only">Close sidebar</span>
                <XMarkIcon class="h-6 w-6 text-white" aria-hidden="true" />
              </button>
            </div>

            <div class="flex grow flex-col gap-y-5 overflow-y-auto bg-white px-6 pb-4">
              <div class="flex h-16 shrink-0 items-center">
                <img class="h-8 w-auto" src="/logo.svg" alt="Lab Syncro" />
              </div>
              
              <!-- Navigation sections -->
              <nav class="flex flex-1 flex-col">
                <ul role="list" class="flex flex-1 flex-col gap-y-7">
                  <!-- Primary navigation -->
                  <li>
                    <div class="text-xs font-semibold leading-6 text-gray-400">TỔNG QUAN</div>
                    <ul role="list" class="-mx-2 mt-2 space-y-1">
                      <li>
                        <a
                          href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                        >
                          <ChartPieIcon class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Dashboard
                        </a>
                      </li>
                    </ul>
                  </li>

                  <!-- Secondary navigation -->
                  <li>
                    <div class="text-xs font-semibold leading-6 text-gray-400">VẬN HÀNH</div>
                    <ul role="list" class="-mx-2 mt-2 space-y-1">
                      <li>
                        <a
                          href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-blue-600 bg-gray-50"
                        >
                          <HandRaisedIcon class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Mượn trả
                        </a>
                      </li>
                      <li>
                        <a
                          href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                        >
                          <CalculatorIcon class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Kiểm đếm
                        </a>
                      </li>
                      <li>
                        <a
                          href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                        >
                          <TruckIcon class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Vận chuyển
                        </a>
                      </li>
                      <li>
                        <a
                          href="#"
                          class="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-blue-600 hover:bg-gray-50"
                        >
                          <WrenchScrewdriverIcon class="h-6 w-6 shrink-0" aria-hidden="true" />
                          Sửa chữa
                        </a>
                      </li>
                    </ul>
                  </li>
                </ul>
              </nav>
            </div>
          </DialogPanel>
        </TransitionChild>
      </div>
    </Dialog>
  </TransitionRoot>
</template>

<script setup lang="ts">
import { Dialog, DialogPanel, TransitionChild, TransitionRoot } from '@headlessui/vue'
import {
  XMarkIcon,
  ChartPieIcon,
  HandRaisedIcon,
  CalculatorIcon,
  TruckIcon,
  WrenchScrewdriverIcon,
} from '@heroicons/vue/24/outline'

defineProps<{
  isOpen: boolean
}>()

const emit = defineEmits<{
  (e: 'close'): void
}>()

const closeDrawer = () => {
  emit('close')
}
</script> 