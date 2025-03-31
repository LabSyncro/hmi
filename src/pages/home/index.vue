<script setup lang="ts">
import { ref } from 'vue'
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection'
import { Package, User, Smartphone, Scan } from 'lucide-vue-next'

const devices = ref<Array<{
  id: string;
  name: string;
  type: string;
  status: 'borrowed' | 'returned';
  timestamp: string;
}>>([])

const scannedUser = ref<string | null>(null)
const scanMode = ref<'borrowed' | 'returned'>('borrowed')

const handleVirtualKeyboardDetection = async (input: string, type?: 'userId' | 'device') => {
  if (type === 'userId') {
    console.log('User scanned:', input)
    alert(`User ${input} (ID: ${input})`)
    scannedUser.value = `User ${input} (ID: ${input})`
  } else if (type === 'device') {
    if (!scannedUser.value) return

    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1]
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1]

    if (deviceKindId && deviceId) {
      const newDevice = {
        id: deviceId,
        name: `Device ${deviceId.substring(0, 4)}`,
        type: 'Hardware',
        status: scanMode.value,
        timestamp: new Date().toLocaleString()
      }

      devices.value = [newDevice, ...devices.value]
    }
  }
}

useVirtualKeyboardDetection(handleVirtualKeyboardDetection, {
  userId: { length: 7 },
  device: { pattern: /^https?:\/\/[^/]+\/devices\/[a-fA-F0-9]{8}\?id=[a-fA-F0-9]+$/ },
  scannerThresholdMs: 100,
  maxInputTimeMs: 1000,
})
</script>

<template>
  <div class="py-6">
    <h1 class="text-2xl font-bold text-center mb-6">GHI NHẬN MƯỢN/TRẢ</h1>

    <div class="flex gap-6">
      <div class="flex-1 bg-white rounded-lg shadow-sm border border-gray-200">
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center justify-center gap-2">
            <Package class="h-5 w-5" />
            DANH SÁCH GHI NHẬN
          </h2>
        </div>

        <div class="p-4">
          <div v-if="devices.length === 0" class="flex flex-col items-center justify-center py-12 text-center">
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <Package class="h-8 w-8 text-gray-400" />
            </div>
            <h3 class="text-lg font-medium mb-1">Không có thiết bị nào được ghi nhận</h3>
          </div>

          <div v-else class="space-y-4">
            <div v-for="device in devices" :key="device.id"
              class="flex items-center justify-between p-4 border rounded-lg">
              <div class="flex items-center gap-3">
                <div class="rounded-full bg-gray-100 p-2">
                  <Smartphone class="h-5 w-5" />
                </div>
                <div>
                  <p class="font-medium">{{ device.name }}</p>
                  <p class="text-sm text-gray-500">{{ device.type }}</p>
                </div>
              </div>
              <div class="flex flex-col items-end">
                <span :class="[
                  'text-xs px-2 py-1 rounded-full',
                  device.status === 'borrowed'
                    ? 'bg-blue-100 text-blue-800'
                    : 'bg-green-100 text-green-800'
                ]">
                  {{ device.status === 'borrowed' ? 'Mượn' : 'Trả' }}
                </span>
                <span class="text-xs text-gray-500 mt-1">
                  {{ device.timestamp }}
                </span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="flex-1 bg-white rounded-lg shadow-sm border border-gray-200 h-fit">
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center justify-center gap-2">
            <Scan class="h-5 w-5" />
            QUÉT MÃ QR
          </h2>
        </div>

        <div class="p-4 space-y-6">
          <div class="space-y-4">
            <div v-if="!scannedUser"
              class="border border-dashed border-gray-300 rounded-lg p-6 flex flex-col items-center justify-center">
              <div class="bg-gray-100 rounded-full p-3 mb-4">
                <User class="h-6 w-6 text-gray-400" />
              </div>
              <h3 class="text-lg font-medium mb-2">Quét mã QR người dùng</h3>
              <p class="text-sm text-gray-500 text-center mb-4">
                Vui lòng quét mã QR của người dùng để xác định ai đang mượn hoặc trả thiết bị.
              </p>
            </div>

            <div v-else class="bg-gray-50 rounded-lg p-4">
              <div class="flex items-center gap-3">
                <div class="bg-white rounded-full p-2 border border-gray-200">
                  <User class="h-5 w-5" />
                </div>
                <div>
                  <p class="font-medium">{{ scannedUser }}</p>
                  <p class="text-sm text-gray-500">Đã xác định người dùng</p>
                </div>
              </div>
            </div>
          </div>

          <template v-if="scannedUser">
            <hr class="border-gray-200" />
          </template>
        </div>
      </div>
    </div>
  </div>
</template>