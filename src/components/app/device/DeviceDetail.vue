<script setup lang="ts">
import { useVirtualKeyboardDetection } from '@/composables';
import { deviceService, type DeviceDetail, type DeviceInventory } from '@/lib/db';
import { DeviceStatus } from '@/types/db/generated';
import { ChevronDownIcon, ChevronUpIcon } from 'lucide-vue-next';
import { onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';

const showMore = ref(false)
const router = useRouter()
const route = useRoute()
const deviceDetail = ref<DeviceDetail | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)
const retrying = ref(false)
const inventory = ref<DeviceInventory[]>([])
const loadingInventory = ref(true)

const handleVirtualKeyboardDetection = async (input: string, type?: 'userId' | 'device') => {
  if (type === 'device') {
    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1];
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1];

    if (deviceKindId && deviceId) {
      if (deviceId !== route.params.id || deviceKindId !== route.query.deviceKindId) {
        router.push({
          name: 'device-detail',
          params: { id: deviceId },
          query: { deviceKindId }
        });
      }
    }
  }
};

useVirtualKeyboardDetection(handleVirtualKeyboardDetection, {
  device: { pattern: /^https?:\/\/[^/]+\/devices\/[a-fA-F0-9]{8}\?id=[a-fA-F0-9]+$/ },
  scannerThresholdMs: 100,
  maxInputTimeMs: 1000,
});

const getStatusColor = (status: DeviceStatus | null) => {
  if (!status) return 'text-gray-500';

  switch (status) {
    case DeviceStatus.HEALTHY:
      return 'text-green-600';
    case DeviceStatus.BROKEN:
      return 'text-red-600';
    case DeviceStatus.DISCARDED:
      return 'text-gray-600';
    case DeviceStatus.ASSESSING:
      return 'text-yellow-600';
    case DeviceStatus.MAINTAINING:
      return 'text-blue-600';
    case DeviceStatus.SHIPPING:
      return 'text-purple-600';
    case DeviceStatus.BORROWING:
      return 'text-orange-600';
    case DeviceStatus.LOST:
      return 'text-red-800';
    default:
      return 'text-gray-500';
  }
};

const getStatusBgColor = (status: DeviceStatus | null) => {
  if (!status) return 'bg-gray-100';

  switch (status) {
    case DeviceStatus.HEALTHY:
      return 'bg-green-50';
    case DeviceStatus.BROKEN:
      return 'bg-red-50';
    case DeviceStatus.DISCARDED:
      return 'bg-gray-50';
    case DeviceStatus.ASSESSING:
      return 'bg-yellow-50';
    case DeviceStatus.MAINTAINING:
      return 'bg-blue-50';
    case DeviceStatus.SHIPPING:
      return 'bg-purple-50';
    case DeviceStatus.BORROWING:
      return 'bg-orange-50';
    case DeviceStatus.LOST:
      return 'bg-red-100';
    default:
      return 'bg-gray-100';
  }
};

const getStatusText = (status: DeviceStatus | null) => {
  if (!status) return 'UNKNOWN';

  switch (status) {
    case DeviceStatus.HEALTHY:
      return 'SẴN SÀNG';
    case DeviceStatus.BROKEN:
      return 'HƯ HỎNG';
    case DeviceStatus.DISCARDED:
      return 'ĐÃ LOẠI BỎ';
    case DeviceStatus.ASSESSING:
      return 'ĐANG ĐÁNH GIÁ';
    case DeviceStatus.MAINTAINING:
      return 'ĐANG BẢO TRÌ';
    case DeviceStatus.SHIPPING:
      return 'ĐANG VẬN CHUYỂN';
    case DeviceStatus.BORROWING:
      return 'ĐANG CHO MƯỢN';
    case DeviceStatus.LOST:
      return 'ĐÃ MẤT';
    default:
      return 'UNKNOWN';
  }
};

async function loadDeviceDetails() {
  loading.value = true
  error.value = null
  try {
    const id = route.params.id as string
    deviceDetail.value = await deviceService.getDeviceReceiptById(id)
    if (!deviceDetail.value) {
      error.value = 'Device not found'
    } else {
      const kindId = route.query.deviceKindId as string || deviceDetail.value.kind
      await loadInventoryData(kindId)
    }
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load device details'
  } finally {
    loading.value = false
    retrying.value = false
  }
}

async function loadInventoryData(kindId: string) {
  loadingInventory.value = true
  try {
    inventory.value = await deviceService.getDeviceInventoryByKindId(kindId)
  } catch (e) {
    throw e
  } finally {
    loadingInventory.value = false
  }
}

onMounted(() => {
  loadDeviceDetails()
})

function retryLoading() {
  retrying.value = true
  loadDeviceDetails()
}

watch(
  () => [route.params.id, route.query.deviceKindId],
  () => {
    loadDeviceDetails();
  }
);
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
        <button @click="retryLoading"
          class="mt-3 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
          :disabled="retrying">
          <span v-if="retrying">Retrying...</span>
          <span v-else>Retry</span>
        </button>
      </div>

      <div v-else-if="deviceDetail" class="bg-white rounded-lg shadow">
        <div class="p-6">
          <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="md:col-span-1">
              <img :src="deviceDetail.image?.mainImage || '/device-image.svg'"
                :alt="deviceDetail.deviceName || 'Device Image'" class="w-full rounded-lg" />
            </div>

            <div class="md:col-span-2">
              <div class="space-y-4">
                <div>
                  <div class="text-sm text-gray-500">MÃ: {{ deviceDetail.fullId }}</div>
                  <h1 class="text-2xl font-bold text-gray-900 mt-1">
                    {{ deviceDetail.deviceName }}
                  </h1>
                </div>

                <dl class="grid grid-cols-1 gap-4">
                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Tình trạng</dt>
                    <dd class="text-sm font-medium col-span-3">
                      <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium capitalize"
                        :class="[getStatusColor(deviceDetail.status), getStatusBgColor(deviceDetail.status)]">
                        {{ getStatusText(deviceDetail.status) }}
                      </span>
                    </dd>
                  </div>

                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Hoạt động</dt>
                    <dd class="text-sm text-gray-900 col-span-3">
                      {{ deviceDetail.status === DeviceStatus.HEALTHY ? 'Mượn trả - Sẵn sàng' : deviceDetail.status ===
                        DeviceStatus.BORROWING ? 'Mượn trả - Đang mượn' : 'Không khả dụng' }}
                    </dd>
                  </div>

                  <div class="grid grid-cols-4">
                    <dt class="text-sm font-medium text-gray-500">Nơi chứa</dt>
                    <dd class="text-sm text-gray-900 col-span-3">
                      {{ deviceDetail.labRoom?.split('-')[1] + ' ' + deviceDetail.labRoom?.split('-')[0] + ', ' +
                        deviceDetail.labBranch }}
                    </dd>
                  </div>

                  <Transition as="template" :show="showMore">
                    <div class="contents">
                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Quyền mượn</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.allowedBorrowRoles?.join(', ') || 'Tất cả' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Phân loại</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.categoryName || 'N/A' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Thương hiệu</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.brand || 'N/A' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Nhà sản xuất</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.manufacturer || 'N/A' }}
                        </dd>
                      </div>

                      <div class="grid grid-cols-4">
                        <dt class="text-sm font-medium text-gray-500">Mô tả</dt>
                        <dd class="text-sm text-gray-900 col-span-3">
                          {{ deviceDetail.description || 'N/A' }}
                        </dd>
                      </div>
                    </div>
                  </Transition>
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

          <div class="mt-4 grid grid-cols-1 md:grid-cols-3 gap-6">
            <div class="bg-white rounded-lg border p-4">
              <h3 class="text-lg font-semibold text-gray-900">Mượn trả</h3>
              <div v-if="deviceDetail.status === DeviceStatus.BORROWING" class="space-y-4">
                <p class="text-base text-gray-500">{{ '#BR_' + deviceDetail.receiptId }}</p>
                <hr />
                <div>
                  <div class="mt-2 flex items-center">
                    <img :src="deviceDetail.borrower?.image || '/default-avatar.png'" alt="Borrower avatar"
                      class="h-12 w-12 rounded-full" />
                    <div class="ml-2">
                      <h4 class="text-base font-medium text-gray-500">Người mượn</h4>
                      <span class="text-sm text-gray-900">[{{ deviceDetail.borrower?.id }}] {{
                        deviceDetail.borrower?.name }}</span>
                    </div>
                  </div>
                </div>

                <div class="grid grid-cols-1 gap-4">
                  <div class="grid grid-cols-2">
                    <dt class="text-sm font-medium text-gray-500">Ngày mượn</dt>
                    <dd class="mt-1 text-sm text-gray-900">{{ deviceDetail.borrowedAt ? new
                      Date(deviceDetail.borrowedAt).toLocaleDateString('vi-VN') : '---' }}</dd>
                  </div>

                  <div class="grid grid-cols-2">
                    <dt class="text-sm font-medium text-gray-500">Ngày hẹn trả</dt>
                    <dd class="mt-1 text-sm text-gray-900">{{ deviceDetail.expectedReturnAt ? new
                      Date(deviceDetail.expectedReturnAt).toLocaleDateString('vi-VN') : '---' }} <span
                        class="text-gray-500">(Dự kiến)</span>
                    </dd>
                  </div>

                  <div class="grid grid-cols-2">
                    <dt class="text-sm font-medium text-gray-500">Nơi mượn</dt>
                    <dd class="mt-1 text-sm text-gray-900">{{ deviceDetail.borrowedLab || '---' }}</dd>
                  </div>

                  <div class="grid grid-cols-2">
                    <dt class="text-sm font-medium text-gray-500">Nơi hẹn trả</dt>
                    <dd class="mt-1 text-sm text-gray-900">{{ deviceDetail.expectedReturnLab || '---' }}</dd>
                  </div>
                </div>

                <button @click="router.push(`/device/${route.params.id}/return`)"
                  class="mt-4 w-full rounded-md py-2 px-4 bg-blue-600 text-white hover:bg-blue-700">
                  Trả thiết bị
                </button>
              </div>

              <div v-else>
                <p class="mt-2 text-sm text-gray-600">
                  {{ deviceDetail.status === DeviceStatus.HEALTHY
                    ? 'Thiết bị đang sẵn sàng để được mượn.'
                    : 'Thiết bị hiện không khả dụng.' }}
                </p>
                <button @click="router.push(`/device/${route.params.id}/borrow`)"
                  :hidden="deviceDetail.status !== DeviceStatus.HEALTHY" :class="[
                    'mt-4 w-full rounded-md py-2 px-4',
                    deviceDetail.status === DeviceStatus.HEALTHY
                      ? 'bg-blue-600 text-white hover:bg-blue-700'
                      : 'bg-gray-300 text-gray-500 cursor-not-allowed'
                  ]">
                  Mượn thiết bị
                </button>
              </div>
            </div>

            <div class="md:col-span-2 bg-white rounded-lg border p-6">
              <h3 class="text-lg font-semibold text-gray-900">Tồn kho thiết bị</h3>
              <div class="mt-4">
                <div v-if="loadingInventory" class="text-center py-4">
                  <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900"></div>
                  <p class="mt-2 text-sm text-gray-600">Loading inventory data...</p>
                </div>
                <table v-else class="min-w-full">
                  <thead>
                    <tr>
                      <th class="text-left text-sm font-medium text-gray-500">Nơi chứa</th>
                      <th class="text-right text-sm font-medium text-gray-500">Sẵn sàng</th>
                      <th class="text-right text-sm font-medium text-gray-500">Đang mượn</th>
                    </tr>
                  </thead>
                  <tbody class="divide-y divide-gray-200 border-t border-gray-200">
                    <tr v-for="item in inventory" :key="item.room + item.branch" class="border-b border-gray-200">
                      <td class="py-4 text-sm text-gray-900">{{ item.room?.split('-')[1] + ' ' +
                        item.room?.split('-')[0] + ', ' + item.branch }}</td>
                      <td class="py-4 text-right text-sm text-gray-900">{{ item.availableQuantity }}</td>
                      <td class="py-4 text-right text-sm text-gray-900">{{ item.borrowingQuantity }}</td>
                    </tr>
                    <tr v-if="inventory.length === 0">
                      <td colspan="3" class="py-4 text-sm text-gray-500 text-center">No inventory data available</td>
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