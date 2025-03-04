<script setup lang="ts">
import { ref } from 'vue'
import { TransitionRoot } from '@headlessui/vue'
import { ChevronDownIcon, ChevronUpIcon } from '@heroicons/vue/24/outline'
import { useRouter, useRoute } from 'vue-router'

const showMore = ref(false)
const router = useRouter()
const route = useRoute()
</script>

<template>
  <div class="bg-gray-50 py-6 sm:px-6 lg:px-8">
    <div class="mx-auto max-w-7xl">
      <div class="bg-white rounded-lg shadow">
        <div class="p-6">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="md:col-span-1">
              <img src="/device-image.svg" alt="Raspberry Pi 3 GPIO-232" class="w-full rounded-lg" />
            </div>

            <div class="md:col-span-2">
              <div class="space-y-4">
                <div>
                  <div class="text-sm text-gray-500">MÃ: 123-123/123-126</div>
                  <h1 class="text-2xl font-bold text-gray-900 mt-1">
                    Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng
                  </h1>
                </div>

                <dl class="grid grid-cols-1 gap-4">
                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Tình trạng</dt>
                    <dd class="text-sm text-green-600 font-medium col-span-3">Tốt</dd>
                  </div>

                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Nơi chứa</dt>
                    <dd class="text-sm text-gray-900 col-span-3">601 H6, Dĩ An</dd>
                  </div>

                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Quyền mượn</dt>
                    <dd class="text-sm text-gray-900 col-span-3">Sinh viên, Giảng viên</dd>
                  </div>

                  <TransitionRoot as="template" :show="showMore">
                    <div class="contents">
                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Phân loại</dt>
                        <dd class="text-sm text-gray-900 col-span-3">Máy hàn, khò</dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Thương hiệu</dt>
                        <dd class="text-sm text-gray-900 col-span-3">OEM</dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Môn đang học</dt>
                        <dd class="text-sm text-gray-900 col-span-3">Đồ án Đa ngành</dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Môn đã học</dt>
                        <dd class="text-sm text-gray-900 col-span-3">N/A</dd>
                      </div>
                    </div>
                  </TransitionRoot>
                </dl>

                <div class="flex justify-start mt-4">
                  <button type="button"
                    class="inline-flex gap-1 items-center text-sm font-medium text-blue-600 hover:text-blue-500"
                    @click="showMore = !showMore">
                    {{ showMore ? 'Ẩn bớt' : 'Xem thêm' }}
                    <ChevronDownIcon v-if="!showMore" class="h-4 w-4 mr-1" aria-hidden="true" />
                    <ChevronUpIcon v-else class="h-4 w-4 mr-1" aria-hidden="true" />
                  </button>
                </div>
              </div>
            </div>
          </div>

          <div class="mt-8 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="bg-white rounded-lg border p-6">
              <h3 class="text-lg font-semibold text-gray-900">Mượn trả</h3>
              <p class="mt-2 text-sm text-gray-600">
                Thiết bị đang sẵn sàng để được mượn.
              </p>
              <button 
                @click="router.push(`/device/${route.params.id}/borrow`)"
                class="mt-4 w-full bg-blue-600 text-white rounded-md py-2 px-4 hover:bg-blue-700">
                Mượn thiết bị
              </button>
            </div>

            <div class="md:col-span-2 bg-white rounded-lg border p-6">
              <h3 class="text-lg font-semibold text-gray-900">Tồn kho thiết bị</h3>
              <div class="mt-4">
                <table class="min-w-full">
                  <thead>
                    <tr>
                      <th class="text-left text-sm font-medium text-gray-500">Nơi chứa</th>
                      <th class="text-right text-sm font-medium text-gray-500">Sẵn sàng</th>
                      <th class="text-right text-sm font-medium text-gray-500">Đang mượn</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-gray-200">
                    <tr>
                      <td class="py-4 text-sm text-gray-900">601 H6, Dĩ An</td>
                      <td class="py-4 text-right text-sm text-gray-900">10</td>
                      <td class="py-4 text-right text-sm text-gray-900">10</td>
                    </tr>
                    <tr>
                      <td class="py-4 text-sm text-gray-900">605 H6, Dĩ An</td>
                      <td class="py-4 text-right text-sm text-gray-900">1</td>
                      <td class="py-4 text-right text-sm text-gray-900">1</td>
                    </tr>
                  </tbody>
                </table>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>