<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { TransitionRoot } from '@headlessui/vue'
import { ChevronDownIcon, ChevronUpIcon } from '@heroicons/vue/24/outline'
import { useRouter, useRoute } from 'vue-router'
import { getDeviceById } from '@/lib/db/device'
import type { DeviceDetail } from '@/lib/db/device'

const showMore = ref(false)
const router = useRouter()
const route = useRoute()
const deviceDetail = ref<DeviceDetail | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const retrying = ref(false)

async function loadDeviceDetails() {
  loading.value = true
  error.value = null
  
  try {
    const id = '1003' // Hardcoded ID for testing
    deviceDetail.value = await getDeviceById(id)
    if (!deviceDetail.value) {
      error.value = 'Device not found'
    }
  } catch (e) {
    console.error('Failed to load device details:', e)
    if (e instanceof Error && e.message.includes('state not managed')) {
      error.value = `Database connection issue: The Tauri backend needs to be updated to properly manage state.
      
In the Rust backend (src-tauri/src/main.rs), you need to ensure the AppState is properly managed. 

Look for the tauri::Builder::default() section and make sure it includes:
.manage(state)
before the .invoke_handler() call.

This error occurs because the 'state' field is not managed for the 'query_table' command.`
    } else {
      error.value = e instanceof Error ? e.message : 'Failed to load device details'
    }
  } finally {
    loading.value = false
    retrying.value = false
  }
}

onMounted(() => {
  loadDeviceDetails()
})

function retryLoading() {
  retrying.value = true
  loadDeviceDetails()
}
</script>

<template>
  <div class="bg-gray-50 py-6 sm:px-6 lg:px-8">
    <div class="mx-auto max-w-7xl">
      <div v-if="loading" class="text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"></div>
        <p class="mt-2 text-sm text-gray-600">Loading device details...</p>
      </div>

      <div v-else-if="error" class="bg-red-50 p-4 rounded-md">
        <p class="text-red-700">{{ error }}</p>
        <button 
          @click="retryLoading" 
          class="mt-3 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          :disabled="retrying"
        >
          <span v-if="retrying">Retrying...</span>
          <span v-else>Retry</span>
        </button>
      </div>

      <div v-else-if="deviceDetail" class="bg-white rounded-lg shadow">
        <div class="p-6">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="md:col-span-1">
              <img :src="deviceDetail.kind.image?.url || '/device-image.svg'" 
                   :alt="deviceDetail.kind.name || 'Device Image'" 
                   class="w-full rounded-lg" />
            </div>

            <div class="md:col-span-2">
              <div class="space-y-4">
                <div>
                  <div class="text-sm text-gray-500">MÃ: {{ deviceDetail.device.fullId }}</div>
                  <h1 class="text-2xl font-bold text-gray-900 mt-1">
                    {{ deviceDetail.kind.name }}
                  </h1>
                </div>

                <dl class="grid grid-cols-1 gap-4">
                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Tình trạng</dt>
                    <dd class="text-sm text-green-600 font-medium col-span-3">
                      {{ deviceDetail.device.status }}
                    </dd>
                  </div>

                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Nơi chứa</dt>
                    <dd class="text-sm text-gray-900 col-span-3">
                      {{ deviceDetail.lab?.name || 'N/A' }}
                    </dd>
                  </div>

                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Quyền mượn</dt>
                    <dd class="text-sm text-gray-900 col-span-3">
                      {{ deviceDetail.kind.allowedBorrowRoles || 'Tất cả' }}
                    </dd>
                  </div>

                  <TransitionRoot as="template" :show="showMore">
                    <div class="contents">
                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Phân loại</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.kind.meta?.category || 'N/A' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Thương hiệu</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.kind.brand || 'N/A' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Nhà sản xuất</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.kind.manufacturer || 'N/A' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Mô tả</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.kind.description || 'N/A' }}
                        </dd>
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
                {{ deviceDetail.device.status === 'AVAILABLE' 
                  ? 'Thiết bị đang sẵn sàng để được mượn.'
                  : 'Thiết bị hiện không khả dụng.' }}
              </p>
              <button 
                @click="router.push(`/device/${route.params.id}/borrow`)"
                :disabled="deviceDetail.device.status !== 'AVAILABLE'"
                :class="[
                  'mt-4 w-full rounded-md py-2 px-4',
                  deviceDetail.device.status === 'AVAILABLE'
                    ? 'bg-blue-600 text-white hover:bg-blue-700'
                    : 'bg-gray-300 text-gray-500 cursor-not-allowed'
                ]">
                Mượn thiết bị
              </button>
              <button
                @click="router.push(`/device/${route.params.id}/return`)"
                :disabled="deviceDetail.device.status !== 'BORROWED'"
                :class="[
                  'mt-4 w-full rounded-md py-2 px-4',
                  deviceDetail.device.status === 'BORROWED'
                    ? 'bg-blue-600 text-white hover:bg-blue-700'
                    : 'bg-gray-300 text-gray-500 cursor-not-allowed'
                ]">
                Trả thiết bị
              </button>
            </div>

            <div class="md:col-span-2 bg-white rounded-lg border p-6">
              <h3 class="text-lg font-semibold text-gray-900">Thông tin thêm</h3>
              <div class="mt-4 prose prose-sm max-w-none">
                <p>{{ deviceDetail.kind.meta?.description }}</p>
                <p v-if="deviceDetail.kind.datasheet">
                  <a :href="deviceDetail.kind.datasheet" 
                     target="_blank"
                     class="text-blue-600 hover:text-blue-500">
                    Xem datasheet
                  </a>
                </p>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>