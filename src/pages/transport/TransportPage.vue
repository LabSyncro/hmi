<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { ChevronDown, Trash, Box, User, Calendar, MapPin, Truck, Package } from 'lucide-vue-next'
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection'
import { deviceService, userService, type DeviceStatus } from '@/lib/db'
import { toast } from '@/components/ui/toast'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { statusMap, statusColorMap, type UserInfo, type Device, type TransportDeviceItem } from '@/types/status'

const mode = ref<'idle' | 'transport'>('idle')

const userInfo = ref<UserInfo | null>(null)

const devices = ref<(Device & { items: TransportDeviceItem[] })[]>([])

const notes = ref<string>("")

const transportDetails = ref({
    sourceLocation: '601 H6, Dĩ An',
    destinationLocation: '602 H6, Dĩ An',
    transportDate: new Date().toLocaleDateString('vi-VN'),
    status: 'pending',
    deviceCount: 0
})

const route = useRoute()

const totalDevices = computed(() => {
    return devices.value.reduce((total, device) => total + device.quantity, 0)
})

const pageTitle = computed(() => "GHI NHẬN VẬN CHUYỂN")

const leftColumnTitle = computed(() => "DANH SÁCH GHI NHẬN")

const rightColumnTitle = computed(() => "NGƯỜI VẬN CHUYỂN")

onMounted(async () => {
    const { userId, userName, userAvatar, userRoles, deviceId, deviceName, deviceImage, deviceStatus, deviceKindId, deviceUnit } = route.query

    if (userId) {
        userInfo.value = {
            id: userId as string,
            name: userName as string,
            avatar: userAvatar as string,
            roles: JSON.parse(userRoles as string || '[]') as { name: string; key: string }[]
        }
    }

    if (deviceId) {
        mode.value = 'transport'

        const initialItem: TransportDeviceItem = {
            id: deviceId as string,
            status: deviceStatus as DeviceStatus,
            transportDestination: transportDetails.value.destinationLocation
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

        updateDeviceCount()
    }
})

async function handleUserCodeChange(userId: string) {
    const isValidUserCode = /^\d{7}$/.test(userId)

    if (!isValidUserCode) {
        toast({ title: 'Lỗi', description: 'Mã người dùng không hợp lệ', variant: 'destructive' })
        userInfo.value = null
        return
    }

    try {
        const userMeta = await userService.getUserById(userId)
        if (!userMeta) {
            toast({ title: 'Lỗi', description: 'Không tìm thấy người dùng', variant: 'destructive' })
            userInfo.value = null
            return
        }

        userInfo.value = {
            id: userMeta.id,
            name: userMeta.name,
            avatar: userMeta.avatar,
            roles: userMeta.roles,
        }

        toast({ title: 'Thành công', description: `Đã nhận diện: ${userMeta.name}` })

    } catch (error) {
        toast({
            title: 'Lỗi',
            description: 'Không thể tìm thấy thông tin người dùng',
            variant: 'destructive'
        })
        userInfo.value = null
    }
}

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
        )

        if (isAlreadyAdded) {
            toast({ title: 'Lỗi', description: 'Thiết bị này đã được thêm vào danh sách.', variant: 'destructive' })
            return
        }

        const deviceDetails = await deviceService.getDeviceById(deviceId)
        if (!deviceDetails || !deviceDetails.status) {
            toast({ title: 'Lỗi', description: 'Không thể lấy thông tin thiết bị', variant: 'destructive' })
            return
        }

        if (mode.value === 'idle') {
            mode.value = 'transport'
        }

        const newItem: TransportDeviceItem = {
            id: deviceId,
            status: deviceDetails.status,
            transportDestination: transportDetails.value.destinationLocation
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

        updateDeviceCount()

        toast({ title: 'Thành công', description: 'Đã thêm thiết bị vào danh sách vận chuyển' })
    } catch (error) {
        toast({
            title: 'Lỗi',
            description: 'Không thể xử lý thiết bị',
            variant: 'destructive'
        })
    }
}

const updateDeviceCount = () => {
    transportDetails.value.deviceCount = totalDevices.value
}

const handleVirtualKeyboardDetection = async (input: string, type?: 'userId' | 'device') => {
    if (type === 'userId') {
        await handleUserCodeChange(input)
    } else if (type === 'device') {
        await handleDeviceScan(input)
    }
}

const toggleDevice = (device: Device & { items: TransportDeviceItem[] }) => {
    if (device.items.length > 0) {
        device.expanded = !device.expanded
    }
}

const removeDeviceItem = (device: Device & { items: TransportDeviceItem[] }, itemId: string) => {
    device.items = device.items.filter((item) => item.id !== itemId) as TransportDeviceItem[]
    device.quantity = device.items.length

    if (device.items.length === 0) {
        devices.value = devices.value.filter(d => d.code !== device.code)
    }

    if (devices.value.length === 0) {
        mode.value = 'idle'
    }

    updateDeviceCount()
}

const resetForm = () => {
    mode.value = 'idle'
    devices.value = []
    notes.value = ""
    transportDetails.value.deviceCount = 0
}

const completeTransport = async () => {
    if (!userInfo.value) {
        toast({ title: 'Lỗi', description: 'Vui lòng quét mã người dùng', variant: 'destructive' })
        return
    }

    if (devices.value.length === 0) {
        toast({ title: 'Lỗi', description: 'Vui lòng quét ít nhất một thiết bị', variant: 'destructive' })
        return
    }

    try {
        toast({ title: 'Thành công', description: 'Đã hoàn tất ghi nhận vận chuyển thiết bị' })
        resetForm()
    } catch (error) {
        toast({ title: 'Lỗi', description: 'Không thể lưu dữ liệu vận chuyển', variant: 'destructive' })
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
    <div>
        <h1 class="text-2xl font-bold text-center">{{ pageTitle }}</h1>
        <p class="text-center text-gray-500 mb-6">
            Sử dụng máy scan quét mã QR thiết bị/người dùng để ghi nhận vận chuyển
        </p>

        <div class="grid grid-cols-3 gap-6">
            <div class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200">
                <div class="p-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold flex items-center gap-2">
                        <Truck class="h-5 w-5" />
                        {{ leftColumnTitle }}
                    </h2>
                </div>

                <div class="p-4">
                    <div v-if="devices.length === 0"
                        class="flex flex-col items-center justify-center py-20 text-center">
                        <div class="rounded-full bg-gray-100 p-3 mb-4">
                            <Package class="h-8 w-8 text-gray-400" />
                        </div>
                        <h3 class="text-lg font-medium mb-1">CHƯA GHI NHẬN</h3>
                        <p class="text-sm text-gray-500 max-w-xs">
                            Quét mã QR thiết bị để ghi nhận
                            <br>
                            vận chuyển thiết bị
                        </p>
                    </div>

                    <div v-else class="divide-y divide-gray-200">
                        <div v-for="device in devices" :key="device.code" class="divide-y divide-gray-100">
                            <div class="p-4 hover:bg-gray-50 cursor-pointer" @click="toggleDevice(device)">
                                <div class="flex items-center justify-between">
                                    <div class="flex items-center gap-3">
                                        <img :src="device.image.mainImage" alt="Device image"
                                            class="h-12 w-12 rounded-full object-cover" />
                                        <div>
                                            <h3 class="font-medium text-gray-900 text-sm">Mã loại: <span
                                                    class="font-bold text-base">{{
                                                        device.code }}</span></h3>
                                            <p class="text-base text-gray-900 font-medium">{{ device.name }}</p>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-4">
                                        <span class="text-base text-gray-900 font-medium mr-32">
                                            {{ device.quantity }} {{ device.unit }}
                                        </span>
                                        <ChevronDown class="h-5 w-5 text-gray-400 transition-transform"
                                            :class="{ 'rotate-180': device.expanded }" />
                                    </div>
                                </div>
                            </div>

                            <div v-if="device.expanded" class="bg-gray-50">
                                <div
                                    class="grid grid-cols-10 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200">
                                    <div class="col-span-4">THIẾT BỊ GHI NHẬN</div>
                                    <div class="col-span-2">ĐỊA ĐIỂM GỐC</div>
                                    <div class="col-span-3">TÌNH TRẠNG</div>
                                    <div class="col-span-1"></div>
                                </div>
                                <div v-for="item in device.items" :key="item.id"
                                    class="grid grid-cols-10 items-center px-4 py-3 border-b border-gray-100 last:border-b-0">
                                    <div class="col-span-4 text-sm font-medium text-gray-900">
                                        {{ device.code }}/{{ item.id }}
                                    </div>
                                    <div class="col-span-2 text-sm text-gray-600">
                                        {{ transportDetails.sourceLocation }}
                                    </div>
                                    <div class="col-span-3">
                                        <Badge :class="statusColorMap[item.status]" class="text-base font-semibold"
                                            variant="outline">
                                            {{ statusMap[item.status] }}
                                        </Badge>
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
                    </div>
                </div>
            </div>

            <div class="flex-1 bg-white rounded-lg shadow-sm border border-gray-200 h-fit">
                <div class="p-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold flex items-center gap-2">
                        <User class="h-5 w-5" />
                        {{ rightColumnTitle }}
                    </h2>
                </div>

                <div>
                    <div class="space-y-4 bg-gray-50 rounded-lg p-2">
                        <div v-if="!userInfo"
                            class="border border-dashed border-gray-300 rounded-lg p-1 flex flex-col items-center justify-center">
                            <div class="bg-gray-100 rounded-full p-3">
                                <User class="h-6 w-6 text-gray-400" />
                            </div>
                            <h3 class="text-lg font-medium">Chưa ghi nhận</h3>
                            <p class="text-sm text-gray-500 text-center">
                                Quét QR định danh người dùng
                            </p>
                        </div>

                        <div v-else class="rounded-lg px-4 py-1">
                            <div class="flex items-center">
                                <img :src="userInfo.avatar" alt="User avatar"
                                    class="h-12 w-12 rounded-full object-cover" />
                                <div class="ml-3">
                                    <h4 class="text-sm font-medium text-gray-500">Mã số:
                                        <span class="text-gray-500 font-semibold">{{ userInfo.id }}</span>
                                        <span class="text-sm text-gray-500 italic font-semibold">
                                            ({{userInfo.roles?.map(r => r.name).join(', ') || 'Không có vai trò'}})
                                        </span>
                                    </h4>
                                    <p class="text-base font-semibold text-gray-900">{{ userInfo.name }}</p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-if="mode === 'transport' && devices.length > 0"
                        class="space-y-4 p-4 border-t border-gray-200">
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
                                <p class="text-sm text-gray-500">Địa điểm gốc</p>
                                <p class="font-medium text-gray-800">{{ transportDetails.sourceLocation }}</p>
                            </div>
                        </div>

                        <div class="flex items-center gap-3">
                            <div class="rounded-full bg-purple-50 p-2">
                                <MapPin class="h-4 w-4 text-purple-600" />
                            </div>
                            <div class="grid grid-cols-2 w-full">
                                <p class="text-sm text-gray-500">Địa điểm đích</p>
                                <p class="font-medium text-gray-800">{{ transportDetails.destinationLocation }}</p>
                            </div>
                        </div>

                        <div class="flex items-center gap-3">
                            <div class="rounded-full bg-green-50 p-2">
                                <Calendar class="h-4 w-4 text-green-600" />
                            </div>
                            <div class="grid grid-cols-2 w-full">
                                <p class="text-sm text-gray-500">Ngày chuyển</p>
                                <p class="font-medium text-gray-800">{{ transportDetails.transportDate }}</p>
                            </div>
                        </div>

                        <div>
                            <label for="notes" class="block text-sm font-medium text-gray-700 mb-1">Ghi chú</label>
                            <Textarea id="notes" v-model="notes"
                                placeholder="Thêm ghi chú về lý do vận chuyển hoặc các yêu cầu đặc biệt..."
                                class="min-h-[80px]" />
                        </div>

                        <button :disabled="!userInfo || devices.length === 0"
                            class="w-full mt-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            @click="completeTransport">
                            <Truck class="h-5 w-5" />
                            Xác nhận chuyển đi
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>