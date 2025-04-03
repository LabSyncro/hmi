<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { ChevronDown, Trash, Box, User, Calendar, MapPin, PackageCheck } from 'lucide-vue-next'
import BorrowReturnLayout from '@/layouts/BorrowReturnLayout.vue'
import { deviceService } from '@/lib/db'
import { useRoute } from 'vue-router'
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection'
import { toast } from '@/components/ui/toast'

type UserInfo = {
  id: string
  name: string
  avatar: string
  roles: { name: string; key: string }[]
}

type DeviceItem = {
  id: string
  status: string
}

type Device = {
  code: string
  name: string,
  image: string,
  quantity: number
  expanded: boolean
  items: DeviceItem[]
}

const route = useRoute()
const userInfo = ref<UserInfo | null>(null)
const devices = ref<Device[]>([])

const borrowDetails = ref({
  location: '601 H6, Dĩ An',
  borrowDate: '12/03/2025',
  returnDate: '16/03/2025',
})

onMounted(async () => {
  const { userId, userName, userAvatar, userRoles } = route.query
  if (userId) {
    userInfo.value = {
      id: userId as string,
      name: userName as string,
      avatar: userAvatar as string,
      roles: JSON.parse(userRoles as string) as { name: string; key: string }[]
    }
  }

  const { deviceId, deviceName, deviceImage, deviceStatus } = route.query
  if (deviceId) {
    devices.value.push({
      code: deviceId as string,
      name: deviceName as string,
      image: deviceImage as string,
      quantity: 1,
      expanded: true,
      items: [{ id: deviceId as string, status: deviceStatus as string }]
    })
  }
})

const handleDeviceScan = async (input: string) => {
  try {
    //const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1]
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1]

    if (!deviceId) {
      toast({ title: 'Lỗi', description: 'Không thể trích xuất ID thiết bị từ mã QR', variant: 'destructive' })
      return
    }

    const deviceStatus = await deviceService.getDeviceStatusById(deviceId)

    if (!deviceStatus || deviceStatus.status !== 'healthy') {
      toast({
        title: 'Lỗi',
        description: `Thiết bị không khả dụng để mượn (ID: ${deviceId})`,
        variant: 'destructive'
      })
      return
    }

    const deviceDetails = await deviceService.getDeviceById(deviceId)
    if (!deviceDetails) {
      toast({ title: 'Lỗi', description: 'Không thể lấy thông tin thiết bị', variant: 'destructive' })
      return
    }

    const existingDevice = devices.value.find(d => d.code === deviceDetails.kind)
    if (existingDevice) {
      existingDevice.items.push({
        id: deviceId,
        status: 'Tốt'
      })
      existingDevice.quantity = existingDevice.items.length
    } else {
      devices.value.push({
        code: deviceDetails.kind,
        name: deviceDetails.deviceName,
        image: deviceDetails.image,
        quantity: 1,
        expanded: true,
        items: [{
          id: deviceId,
          status: deviceStatus.status
        }]
      })
    }

    toast({ title: 'Thành công', description: 'Đã thêm thiết bị vào danh sách mượn' })
  } catch (error) {
    toast({
      title: 'Lỗi',
      description: 'Không thể xử lý thiết bị',
      variant: 'destructive'
    })
  }
}

const toggleDevice = (device: any) => {
  device.expanded = !device.expanded
}

const removeDeviceItem = (device: any, itemId: string) => {
  device.items = device.items.filter((item: any) => item.id !== itemId)
  device.quantity = device.items.length

  if (device.items.length === 0) {
    devices.value = devices.value.filter(d => d.code !== device.code)
  }
}

const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0)
})

useVirtualKeyboardDetection((input: string) => handleDeviceScan(input), {
  device: { pattern: /^https?:\/\/[^/]+\/devices\/[a-fA-F0-9]{8}\?id=[a-fA-F0-9]+$/ },
  scannerThresholdMs: 100,
  maxInputTimeMs: 1000,
})
</script>

<template>
  <BorrowReturnLayout title="GHI NHẬN MƯỢN" left-column-title="DANH SÁCH MƯỢN" right-column-title="THÔNG TIN MƯỢN"
    :left-icon="PackageCheck" :right-icon="User">
    <template #left-column>
      <div class="divide-y divide-gray-200">
        <div v-for="device in devices" :key="device.code" class="divide-y divide-gray-100">
          <div class="p-4 hover:bg-gray-50 cursor-pointer" @click="toggleDevice(device)">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <img :src="device.image" alt="Device image" class="h-12 w-12 rounded-full object-cover" />
                <div>
                  <h3 class="font-medium text-gray-900">MÃ LOẠI: {{ device.code }}</h3>
                  <p class="text-sm text-gray-900 font-medium">{{ device.name }}</p>
                </div>
              </div>
              <div class="flex items-center gap-4">
                <span class="text-base text-gray-900 font-medium mr-32">
                  {{ device.quantity }} cái
                </span>
                <ChevronDown class="h-5 w-5 text-gray-400 transition-transform"
                  :class="{ 'rotate-180': device.expanded }" />
              </div>
            </div>
          </div>

          <div v-if="device.expanded" class="bg-gray-50">
            <div class="p-4">
              <div class="flex justify-between mb-2">
                <h4 class="text-sm font-medium text-gray-500">THIẾT BỊ GHI NHẬN</h4>
                <h4 class="text-sm font-medium text-gray-500 mr-32">TÌNH TRẠNG</h4>
              </div>
              <div class="space-y-3">
                <div v-for="item in device.items" :key="item.id" class="flex items-center justify-between">
                  <div class="text-sm font-medium text-gray-900">
                    {{ device.code }}/{{ item.id }}
                  </div>
                  <div class="flex items-center gap-3">
                    <span class="text-sm text-gray-900 font-medium mr-32">{{ item.status }}</span>
                    <button @click.stop="removeDeviceItem(device, item.id)"
                      class="text-gray-400 hover:text-red-500 transition-colors p-1 rounded-full hover:bg-gray-100"
                      aria-label="Remove device">
                      <Trash class="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template #right-column>
      <div class="p-4 space-y-6">
        <div v-if="userInfo" class="flex flex-col items-center p-4 bg-gray-50 rounded-lg">
          <div class="mt-2 flex items-center">
            <img :src="userInfo.avatar || 'default-avatar.png'" alt="User avatar"
              class="h-12 w-12 rounded-full object-cover" />
            <div class="ml-3">
              <h4 class="text-sm font-medium text-gray-500">{{ userInfo.id }}</h4>
              <p class="text-base font-semibold text-gray-900">{{ userInfo.name }}</p>
              <p class="text-sm text-gray-600">
                {{userInfo.roles?.map(r => r.name).join(', ') || 'Không có vai trò'}}
              </p>
            </div>
          </div>
        </div>
        <div v-else class="text-center text-gray-500 py-4">
          Đang tải thông tin người dùng...
        </div>

        <div class="space-y-4">
          <div class="flex items-center gap-3">
            <div class="rounded-full bg-blue-50 p-2">
              <Box class="h-4 w-4 text-blue-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Tổng thiết bị</p>
              <p class="font-medium text-gray-800">{{ totalDevices }} cái</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-amber-50 p-2">
              <MapPin class="h-4 w-4 text-amber-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Nơi mượn/trả</p>
              <p class="font-medium text-gray-800">{{ borrowDetails.location }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-green-50 p-2">
              <Calendar class="h-4 w-4 text-green-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Ngày mượn</p>
              <p class="font-medium text-gray-800">{{ borrowDetails.borrowDate }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-purple-50 p-2">
              <Calendar class="h-4 w-4 text-purple-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Ngày hẹn trả</p>
              <p class="font-medium text-gray-800">{{ borrowDetails.returnDate }}</p>
            </div>
          </div>
        </div>

        <button :disabled="!userInfo"
          class="w-full mt-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed">
          <PackageCheck class="h-5 w-5" />
          Xác nhận mượn
        </button>
      </div>
    </template>
  </BorrowReturnLayout>
</template>