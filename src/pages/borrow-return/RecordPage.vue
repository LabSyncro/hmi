<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { ChevronDown, Trash, Box, User, Calendar, MapPin, PackageCheck, Package } from 'lucide-vue-next'
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection'
import { deviceService, userService, type DeviceStatus, type DeviceQuality } from '@/lib/db'
import { toast } from '@/components/ui/toast'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Textarea } from '@/components/ui/textarea'
import { Badge } from '@/components/ui/badge'
import { statusMap, statusColorMap, qualityMap, qualityColorMap, type UserInfo, type Device, type ReturnDeviceItem } from '@/types/status'

const mode = ref<'idle' | 'borrow' | 'return'>('idle')

const userInfo = ref<UserInfo | null>(null)

const devices = ref<(Device & { items: ReturnDeviceItem[] })[]>([])

const notes = ref<string>("")

const borrowDetails = ref({
    location: '601 H6, Dĩ An',
    borrowDate: new Date().toLocaleDateString('vi-VN'),
    returnDate: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000).toLocaleDateString('vi-VN'),
})

const returnDetails = ref({
    location: '601 H6, Dĩ An',
    borrowDate: '12/03/2025',
    returnDate: '16/03/2025',
    actualReturnDate: new Date().toLocaleDateString('vi-VN'),
    returnProgress: 'Trễ hạn'
})

const route = useRoute()

const totalDevices = computed(() => {
    return devices.value.reduce((total, device) => total + device.quantity, 0)
})

const pageTitle = computed(() => {
    if (mode.value === 'borrow') return 'GHI NHẬN MƯỢN'
    if (mode.value === 'return') return 'GHI NHẬN TRẢ'
    return 'GHI NHẬN MƯỢN TRẢ'
})

const leftColumnTitle = computed(() => {
    if (mode.value === 'borrow') return 'DANH SÁCH MƯỢN'
    if (mode.value === 'return') return 'DANH SÁCH TRẢ'
    return 'DANH SÁCH GHI NHẬN'
})

const rightColumnTitle = computed(() => {
    if (mode.value === 'borrow') return 'NGƯỜI MƯỢN'
    if (mode.value === 'return') return 'NGƯỜI TRẢ'
    return 'THÔNG TIN NGƯỜI MƯỢN/TRẢ'
})

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
        if (deviceStatus === 'borrowing') {
            mode.value = 'return'

            const initialItem: ReturnDeviceItem = {
                id: deviceId as string,
                status: deviceStatus as DeviceStatus,
                returnCondition: 'healthy' as DeviceStatus,
                prevQuality: 'healthy' as DeviceQuality
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
        } else {
            mode.value = 'borrow'

            const initialItem: ReturnDeviceItem = {
                id: deviceId as string,
                status: deviceStatus as DeviceStatus,
                returnCondition: 'healthy' as DeviceStatus,
                prevQuality: 'healthy' as DeviceQuality
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
            if (deviceDetails.status === 'healthy' || deviceDetails.status === 'broken') {
                mode.value = 'borrow'
            } else if (deviceDetails.status === 'borrowing') {
                mode.value = 'return'
            } else {
                toast({ title: 'Thông báo', description: `Thiết bị đang ở trạng thái '${statusMap[deviceDetails.status]}', không thể mượn/trả.` })
                return
            }
        }

        if (mode.value === 'borrow' && (deviceDetails.status !== 'healthy' && deviceDetails.status !== 'broken')) {
            toast({
                title: 'Lỗi',
                description: `Thiết bị không khả dụng để mượn (ID: ${deviceId})`,
                variant: 'destructive'
            })
            return
        }

        if (mode.value === 'return' && deviceDetails.status !== 'borrowing') {
            toast({
                title: 'Lỗi',
                description: `Thiết bị không ở trạng thái đang mượn (ID: ${deviceId})`,
                variant: 'destructive'
            })
            return
        }

        if (mode.value === 'borrow') {
            const existingDevice = devices.value.find(d => d.code === deviceKindId)
            if (existingDevice) {
                existingDevice.items.push({
                    id: deviceId,
                    status: deviceDetails.status,
                    returnCondition: 'healthy' as DeviceStatus,
                    prevQuality: 'healthy' as DeviceQuality
                })
                existingDevice.quantity = existingDevice.items.length
            } else {
                devices.value.push({
                    code: deviceKindId!,
                    name: deviceDetails.deviceName,
                    image: deviceDetails.image,
                    quantity: 1,
                    unit: deviceDetails.unit,
                    expanded: true,
                    items: [{
                        id: deviceId,
                        status: deviceDetails.status,
                        returnCondition: 'healthy' as DeviceStatus,
                        prevQuality: 'healthy' as DeviceQuality
                    }]
                })
            }
            toast({ title: 'Thành công', description: 'Đã thêm thiết bị vào danh sách mượn' })
        } else if (mode.value === 'return') {
            const newItem: ReturnDeviceItem = {
                id: deviceId,
                status: deviceDetails.status,
                returnCondition: 'healthy' as DeviceStatus,
                prevQuality: 'healthy' as DeviceQuality
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
        }
    } catch (error) {
        toast({
            title: 'Lỗi',
            description: 'Không thể xử lý thiết bị',
            variant: 'destructive'
        })
    }
}

const handleVirtualKeyboardDetection = async (input: string, type?: 'userId' | 'device') => {
    if (type === 'userId') {
        await handleUserCodeChange(input)
    } else if (type === 'device') {
        await handleDeviceScan(input)
    }
}

const toggleDevice = (device: Device) => {
    if (device.items.length > 0) {
        device.expanded = !device.expanded
    }
}

const removeDeviceItem = (device: Device, itemId: string) => {
    device.items = device.items.filter((item) => item.id !== itemId)
    device.quantity = device.items.length

    if (device.items.length === 0) {
        devices.value = devices.value.filter(d => d.code !== device.code)
    }

    if (devices.value.length === 0) {
        mode.value = 'idle'
    }
}

const resetForm = () => {
    mode.value = 'idle'
    devices.value = []
    notes.value = ""
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
            Sử dụng máy scan quét mã QR thiết bị/người dùng để ghi nhận mượn trả
        </p>

        <div class="grid grid-cols-3 gap-6">
            <div class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200">
                <div class="p-4 border-b border-gray-200">
                    <h2 class="text-lg font-semibold flex items-center gap-2">
                        <PackageCheck class="h-5 w-5" />
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
                            mượn trả thiết bị
                        </p>
                    </div>

                    <div v-else-if="mode === 'borrow'" class="divide-y divide-gray-200">
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
                                <div class="p-4">
                                    <div class="flex justify-between mb-2">
                                        <h4 class="text-sm font-medium text-gray-500">THIẾT BỊ GHI NHẬN</h4>
                                        <h4 class="text-sm font-medium text-gray-500 mr-32">TÌNH TRẠNG</h4>
                                    </div>
                                    <div class="space-y-3">
                                        <div v-for="item in device.items" :key="item.id"
                                            class="flex items-center justify-between">
                                            <div class="text-sm font-medium text-gray-900">
                                                {{ device.code }}/{{ item.id }}
                                            </div>
                                            <div class="flex items-center gap-3">
                                                <Badge :class="statusColorMap[item.status]"
                                                    class="text-base font-semibold mr-32" variant="outline">
                                                    {{ statusMap[item.status] }}
                                                </Badge>
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

                    <div v-else-if="mode === 'return'" class="divide-y divide-gray-200">
                        <div v-for="device in devices" :key="device.code" class="divide-y divide-gray-100">
                            <div class="p-4 hover:bg-gray-50 cursor-pointer"
                                :class="{ 'cursor-pointer': device.items.length > 0, 'opacity-50': device.items.length === 0 }"
                                @click="toggleDevice(device)">
                                <div class="grid grid-cols-10 items-center">
                                    <div class="col-span-7 flex items-center gap-3">
                                        <img :src="device.image.mainImage" alt="Device image"
                                            class="h-12 w-12 rounded-full object-cover" />
                                        <div>
                                            <h3 class="font-medium text-gray-900 text-sm">Mã loại: <span
                                                    class="font-bold text-base">{{
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
                                <div
                                    class="grid grid-cols-10 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200">
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
                                            <span :class="qualityColorMap[item.prevQuality || 'healthy']"
                                                class="text-base font-semibold w-fit text-right flex-shrink-0 bg-transparent">
                                                {{ qualityMap[item.prevQuality || 'healthy'] }}
                                            </span>
                                            <span class="text-gray-400 mx-1">→</span>
                                            <Select v-model="item.returnCondition" class="flex-grow">
                                                <SelectTrigger
                                                    class="h-9 text-sm bg-white text-base font-semibold w-fit"
                                                    :class="item.returnCondition ? statusColorMap[item.returnCondition] : 'text-gray-900'">
                                                    <SelectValue placeholder="Chọn tình trạng" />
                                                </SelectTrigger>
                                                <SelectContent>
                                                    <SelectItem v-for="(label, status) in statusMap" :key="status"
                                                        :value="status">
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
                                    <p class="text-base font-semibold text-gray-900">{{ userInfo.name }}
                                    </p>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-if="mode === 'borrow' && devices.length > 0" class="space-y-4 p-4 border-t border-gray-200">
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
                                <p class="font-medium text-gray-800">{{ borrowDetails.location }}</p>
                            </div>
                        </div>

                        <div class="flex items-center gap-3">
                            <div class="rounded-full bg-green-50 p-2">
                                <Calendar class="h-4 w-4 text-green-600" />
                            </div>
                            <div class="grid grid-cols-2 w-full">
                                <p class="text-sm text-gray-500">Ngày mượn</p>
                                <p class="font-medium text-gray-800">{{ borrowDetails.borrowDate }}</p>
                            </div>
                        </div>

                        <div class="flex items-center gap-3">
                            <div class="rounded-full bg-purple-50 p-2">
                                <Calendar class="h-4 w-4 text-purple-600" />
                            </div>
                            <div class="grid grid-cols-2 w-full">
                                <p class="text-sm text-gray-500">Ngày hẹn trả</p>
                                <p class="font-medium text-gray-800">{{ borrowDetails.returnDate }}</p>
                            </div>
                        </div>

                        <button :disabled="!userInfo || devices.length === 0"
                            class="w-full mt-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            @click="resetForm">
                            <PackageCheck class="h-5 w-5" />
                            Xác nhận mượn
                        </button>
                    </div>

                    <div v-if="mode === 'return' && devices.length > 0" class="space-y-4 p-4 border-t border-gray-200">
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
                                placeholder="Thêm ghi chú về tình trạng thiết bị hoặc lý do trả trễ (nếu có)..."
                                class="min-h-[80px]" />
                        </div>

                        <button :disabled="!userInfo || devices.length === 0"
                            class="w-full mt-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                            @click="resetForm">
                            <PackageCheck class="h-5 w-5" />
                            Xác nhận trả
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>