<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { ChevronDown, Trash, Box, User, Calendar, MapPin, PackageCheck } from 'lucide-vue-next'
import BorrowReturnLayout from '@/layouts/BorrowReturnLayout.vue'
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection'
import { toast } from '@/components/ui/toast'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { deviceService, userService, DeviceStatus } from '@/lib/db'
import { statusMap, statusColorMap, type UserInfo, type Device, type DeviceItem } from '@/types/status'

const route = useRoute()
const userInfo = ref<UserInfo | null>(null)
const devices = ref<Device[]>([])
const notes = ref<string>("")

const returnDetails = ref({
  location: '601 H6, Dĩ An',
  borrowDate: '12/03/2025',
  returnDate: '16/03/2025',
  actualReturnDate: new Date().toLocaleDateString('vi-VN'),
  returnProgress: 'Trễ hạn'
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

  const { deviceId, deviceName, deviceImage, deviceStatus, deviceKindId, deviceUnit } = route.query
  if (deviceId) {
    const initialItem: DeviceItem = {
      id: deviceId as string,
      status: deviceStatus as DeviceStatus,
      returnCondition: DeviceStatus.HEALTHY
    }
    devices.value.push({
      code: deviceKindId as string,
      name: deviceName as string,
      image: deviceImage as string,
      quantity: 1,
      unit: deviceUnit as string,
      expanded: true,
      items: [initialItem]
    })
  }
})

const handleDeviceScan = async (input: string) => {
  try {
    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1]
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1]

    if (!deviceId) {
      toast({ title: 'Lỗi', description: 'Không thể trích xuất ID thiết bị từ mã QR', variant: 'destructive' })
      return
    }

    const isAlreadyAdded = devices.value.some(device =>
      device.items.some(item => item.id === deviceId)
    );

    if (isAlreadyAdded) {
      toast({ title: 'Lỗi', description: 'Thiết bị này đã được thêm vào danh sách.', variant: 'destructive' });
      return;
    }

    const deviceInfo = await deviceService.getDeviceStatusById(deviceId)

    if (!deviceInfo || deviceInfo.status !== 'borrowing') {
      toast({
        title: 'Lỗi',
        description: `Thiết bị không ở trạng thái đang mượn (ID: ${deviceId})`,
        variant: 'destructive'
      })
      return
    }

    const deviceDetails = await deviceService.getDeviceById(deviceId)
    if (!deviceDetails) {
      toast({ title: 'Lỗi', description: 'Không thể lấy thông tin thiết bị', variant: 'destructive' })
      return
    }

    const newItem: DeviceItem = {
      id: deviceId,
      status: deviceInfo.status,
      returnCondition: DeviceStatus.HEALTHY
    }

    const existingDevice = devices.value.find(d => d.code === deviceKindId)
    if (existingDevice) {
      existingDevice.items.push(newItem)
      existingDevice.quantity = existingDevice.items.length
    } else {
      devices.value.push({
        code: deviceKindId!,
        name: deviceDetails.deviceName,
        image: deviceDetails.image,
        quantity: 1,
        unit: deviceDetails.unit,
        expanded: true,
        items: [newItem]
      })
    }

    toast({ title: 'Thành công', description: 'Đã thêm thiết bị vào danh sách trả' })
  } catch (error) {
    toast({
      title: 'Lỗi',
      description: 'Không thể xử lý thiết bị',
      variant: 'destructive'
    })
  }
}

const handleUserScan = async (userId: string) => {
  try {
    const fetchedUserInfo = await userService.getUserById(userId);
    if (fetchedUserInfo) {
      userInfo.value = {
        id: fetchedUserInfo.id,
        name: fetchedUserInfo.name,
        avatar: fetchedUserInfo.avatar,
        roles: fetchedUserInfo.roles,
      };
      toast({ title: 'Thành công', description: 'Đã quét thông tin người mượn.' });
    } else {
      toast({ title: 'Lỗi', description: 'Không tìm thấy người dùng.', variant: 'destructive' });
    }
  } catch (error) {
    toast({ title: 'Lỗi', description: 'Không thể lấy thông tin người dùng.', variant: 'destructive' });
  }
}

const toggleDevice = (device: Device) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded
  }
}

const removeDeviceItem = (device: Device, itemId: string) => {
  device.items = device.items.filter((item: DeviceItem) => item.id !== itemId)
  device.quantity = device.items.length

  if (device.items.length === 0) {
    devices.value = devices.value.filter(d => d.code !== device.code)
  }
}

const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0)
})

useVirtualKeyboardDetection((input: string, type?: 'userId' | 'device' | undefined) => {
  if (type === 'userId') {
    handleUserScan(input);
  } else if (type === 'device') {
    handleDeviceScan(input);
  }
}, {
  userId: { length: 7 },
  device: { pattern: /^https?:\/\/[^/]+\/devices\/[a-fA-F0-9]{8}\?id=[a-fA-F0-9]+$/ },
  scannerThresholdMs: 100,
  maxInputTimeMs: 1000,
})
</script>

<template>
  <BorrowReturnLayout title="GHI NHẬN TRẢ" left-column-title="DANH SÁCH TRẢ" right-column-title="THÔNG TIN TRẢ"
    :left-icon="PackageCheck" :right-icon="User">
    <template #left-column>
      <div class="divide-y divide-gray-200">
        <div v-for="device in devices" :key="device.code" class="divide-y divide-gray-100">
          <div class="p-4 hover:bg-gray-50 cursor-pointer"
            :class="{ 'cursor-pointer': device.items.length > 0, 'opacity-50': device.items.length === 0 }"
            @click="toggleDevice(device)">
            <div class="grid grid-cols-10 items-center">
              <div class="col-span-7 flex items-center gap-3">
                <img :src="device.image" alt="Device image" class="h-12 w-12 rounded-full object-cover" />
                <div>
                  <h3 class="font-medium text-gray-900 text-sm">Mã loại: <span class="font-bold text-base">{{
                    device.code }}</span></h3>
                  <p class="text-base text-gray-900 font-medium">{{ device.name }}</p>
                </div>
              </div>
              <span class="col-span-2 text-base text-gray-900 font-medium mr-4">
                {{ device.quantity }} {{ device.unit }}
              </span>
              <ChevronDown class="h-5 w-5 text-gray-400 transition-transform justify-self-end"
                :class="{ 'rotate-180': device.expanded }" />
            </div>
          </div>

          <div v-if="device.expanded && device.items.length > 0" class="bg-gray-50">
            <div class="grid grid-cols-10 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200">
              <div class="col-span-3">THIẾT BỊ GHI NHẬN</div>
              <div class="col-span-3">TIẾN ĐỘ TRẢ</div>
              <div class="col-span-3 text-center">TÌNH TRẠNG</div>
              <div class="col-span-1"></div>
            </div>
            <div v-for="item in device.items" :key="item.id"
              class="grid grid-cols-10 items-center px-4 py-3 border-b border-gray-100 last:border-b-0">
              <div class="col-span-3 text-sm font-medium text-gray-900">
                {{ device.code }}/{{ item.id }}
              </div>
              <div class="col-span-3 text-sm text-gray-600">
                Đúng hạn (còn 2 ngày)
              </div>
              <div class="col-span-3">
                <div class="flex items-center gap-1">
                  <span :class="statusColorMap[item.status]" class="text-sm font-medium w-fit text-right flex-shrink-0">
                    {{ statusMap[item.status] }}
                  </span>
                  <span class="text-gray-400 mx-1">→</span>
                  <Select v-model="item.returnCondition" class="flex-grow">
                    <SelectTrigger class="h-9 text-sm bg-white font-medium"
                      :class="item.returnCondition ? statusColorMap[item.returnCondition] : 'text-gray-900'">
                      <SelectValue placeholder="Chọn tình trạng" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem v-for="(label, status) in statusMap" :key="status" :value="status">
                        <span :class="statusColorMap[status]">{{ label }}</span>
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
              <div class="col-span-1 flex justify-end">
                <button @click.stop="removeDeviceItem(device, item.id)"
                  class="text-gray-400 hover:text-red-500 transition-colors p-1 rounded-full hover:bg-gray-100"
                  aria-label="Remove device">
                  <Trash class="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
        <div v-if="devices.length === 0" class="p-6 text-center text-gray-500">
          Chưa có thiết bị nào được quét.
        </div>
      </div>
    </template>

    <template #right-column>
      <div class="space-y-6">
        <div v-if="userInfo" class="flex flex-col items-center py-1 bg-gray-50 rounded-lg">
          <div class="flex items-center">
            <img :src="userInfo.avatar" alt="User avatar" class="h-12 w-12 rounded-full object-cover" />
            <div class="ml-3">
              <h4 class="text-sm font-medium text-gray-500">{{ userInfo.id }}</h4>
              <p class="text-base font-semibold text-gray-900">
                {{ userInfo.name }}
                <span class="text-sm text-gray-500 italic font-normal">
                  ({{userInfo.roles?.map(r => r.name).join(', ') || 'Không có vai trò'}})
                </span>
              </p>
            </div>
          </div>
        </div>
        <div v-else
          class="m-1 border border-dashed border-gray-300 rounded-lg py-1 flex flex-col items-center justify-center">
          <div class="bg-gray-100 rounded-full p-3 mb-4">
            <User class="h-6 w-6 text-gray-400" />
          </div>
          <h3 class="text-lg font-medium">Quét mã QR người trả</h3>
        </div>

        <div class="space-y-4 p-4 border-t border-gray-200">
          <div class="flex items-center gap-3">
            <div class="rounded-full bg-blue-50 p-2">
              <Box class="h-4 w-4 text-blue-600" />
            </div>
            <div class="grid grid-cols-2 w-full">
              <p class="text-sm text-gray-500">Tổng thiết bị</p>
              <p class="font-medium text-gray-800">{{ totalDevices }} cái</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-amber-50 p-2">
              <MapPin class="h-4 w-4 text-amber-600" />
            </div>
            <div class="grid grid-cols-2 w-full">
              <p class="text-sm text-gray-500">Nơi mượn/trả</p>
              <p class="font-medium text-gray-800">{{ returnDetails.location }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-indigo-50 p-2">
              <Calendar class="h-4 w-4 text-indigo-600" />
            </div>
            <div class="grid grid-cols-2 w-full">
              <p class="text-sm text-gray-500">Ngày trả</p>
              <p class="font-medium text-gray-800">{{ returnDetails.actualReturnDate }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-red-50 p-2">
              <Calendar class="h-4 w-4 text-red-600" />
            </div>
            <div class="grid grid-cols-2 w-full">
              <p class="text-sm text-gray-500">Tiến độ trả</p>
              <p class="font-medium text-red-600">{{ returnDetails.returnProgress }}</p>
            </div>
          </div>

          <div>
            <label for="notes" class="block text-sm font-medium text-gray-700 mb-1">Ghi chú</label>
            <Textarea id="notes" v-model="notes"
              placeholder="Thêm ghi chú về tình trạng thiết bị hoặc lý do trả trễ (nếu có)..." class="min-h-[80px]" />
          </div>

          <button :disabled="!userInfo || devices.length === 0"
            class="w-full mt-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed">
            <PackageCheck class="h-5 w-5" />
            Xác nhận trả
          </button>
        </div>
      </div>
    </template>
  </BorrowReturnLayout>
</template>