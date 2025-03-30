<script setup lang="ts">
import { ref, Transition } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { TrashIcon, ChevronRightIcon, ChevronDownIcon, ChevronUpIcon } from 'lucide-vue-next'
import { DropdownMenu, DropdownMenuTrigger, DropdownMenuContent, DropdownMenuItem } from '@/components/ui/dropdown-menu'

type Device = {
    id: string
    code: string
    name: string
    image: string
    status: 'Tốt' | 'Hư'
    borrowId: string
    progress: 'Trễ hạn' | 'Đúng hạn'
}

const router = useRouter()
const route = useRoute()

const showMore = ref(false)

const devices: Device[] = [
    {
        id: '1',
        code: '123-123',
        borrowId: '123-446',
        name: 'Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng',
        image: '/device-image.svg',
        status: 'Tốt',
        progress: 'Trễ hạn'
    },
    {
        id: '2',
        code: '123-123',
        borrowId: '123-446',
        name: 'Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng',
        image: '/device-image.svg',
        status: 'Hư',
        progress: 'Trễ hạn'
    },
    {
        id: '3',
        code: '123-123',
        borrowId: '123-446',
        name: 'Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng',
        image: '/device-image.svg',
        status: 'Tốt',
        progress: 'Trễ hạn'
    },
    {
        id: '4',
        code: '123-123',
        borrowId: '123-446',
        name: 'Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng',
        image: '/device-image.svg',
        status: 'Tốt',
        progress: 'Đúng hạn'
    },
    {
        id: '5',
        code: '123-123',
        borrowId: '123-446',
        name: 'Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng',
        image: '/device-image.svg',
        status: 'Tốt',
        progress: 'Đúng hạn'
    }
]

const stats = {
    onTime: 17,
    late: 3,
    good: 15,
    damaged: 5
}

const getStatusColor = (status: string) => {
    return {
        'Tốt': 'text-green-600',
        'Hư': 'text-red-600'
    }[status] || ''
}

const getStatusBgColor = (status: string) => {
    return {
        'Tốt': 'bg-green-50 text-green-600 ring-1 ring-green-600/20',
        'Hư': 'bg-red-50 text-red-600 ring-1 ring-red-600/20'
    }[status] || ''
}

const getProgressColor = (progress: string) => {
    return progress === 'Trễ hạn' ? 'text-red-600' : 'text-gray-900'
}
</script>

<template>
    <div class="bg-gray-50">
        <div class="flex overflow-hidden">
            <div class="w-1/3 border-r bg-white p-6">
                <h2 class="text-2xl font-medium text-gray-900 mb-8">Chi tiết thiết bị</h2>

                <div class="space-y-6">
                    <div class="space-y-2">
                        <div class="flex items-start space-x-4">
                            <img src="/device-image.svg" alt="" class="w-24 h-24 rounded-lg border p-1" />
                            <div class="flex-1">
                                <div class="text-base text-gray-500">MÃ: 123-123</div>
                                <h3 class="text-xl font-medium text-gray-900 mt-1">
                                    Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng
                                </h3>
                            </div>
                        </div>
                    </div>

                    <div class="space-y-4">
                        <div class="grid grid-cols-3 gap-2 text-base mb-4">
                            <span class="text-gray-500">Quyền mượn</span>
                            <span class="text-gray-900 col-span-2">Sinh viên, Giảng viên</span>
                        </div>

                        <Transition as="template" :show="showMore">
                            <div class="contents text-base space-y-4">
                                <div class="grid grid-cols-3 items-center gap-2 text-sm">
                                    <dt class="text-gray-500 w-32">Phân loại</dt>
                                    <dd class="text-gray-900 col-span-2">Máy hàn, khò</dd>
                                </div>

                                <div class="grid grid-cols-3 items-center gap-2 text-sm">
                                    <dt class="font-medium text-gray-500">Thương hiệu</dt>
                                    <dd class="text-gray-900 col-span-2">OEM</dd>
                                </div>

                                <div class="grid grid-cols-3 items-center gap-2 text-sm">
                                    <dt class="font-medium text-gray-500">Môn đang học</dt>
                                    <dd class="text-gray-900 col-span-2">Đồ án Đa ngành</dd>
                                </div>

                                <div class="grid grid-cols-3 items-center gap-2 text-sm">
                                    <dt class="font-medium text-gray-500">Môn đã học</dt>
                                    <dd class="text-gray-900 col-span-2">N/A</dd>
                                </div>
                            </div>
                        </Transition>

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

                    <div class="border rounded-lg overflow-hidden">
                        <table class="w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th scope="col"
                                        class="px-10 py-2 text-left text-sm font-medium text-gray-500 w-[45%]">Mã</th>
                                    <th scope="col"
                                        class="px-3 py-2 text-left text-sm font-medium text-gray-500 w-[25%]">Tiến độ
                                    </th>
                                    <th scope="col"
                                        class="px-3 py-2 text-left text-sm font-medium text-gray-500 w-[30%]">Tình trạng
                                    </th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-200 bg-white">
                                <tr v-for="device in devices" :key="device.id">
                                    <td class="px-3 py-2 whitespace-nowrap text-sm">
                                        <div class="flex items-center">
                                            <TrashIcon class="h-4 w-4 text-red-500 mr-2 cursor-pointer shrink-0" />
                                            <span class="text-gray-900 truncate">{{ device.code }}/{{ device.borrowId
                                                }}</span>
                                        </div>
                                    </td>
                                    <td class="px-3 py-2 whitespace-nowrap text-sm"
                                        :class="getProgressColor(device.progress)">
                                        {{ device.progress }}
                                    </td>
                                    <td class="px-3 py-2 whitespace-nowrap text-sm">
                                        <div class="flex items-center gap-1">
                                            <span :class="getStatusColor(device.status)">{{ device.status }}</span>
                                            <ChevronRightIcon class="h-3 w-3 text-gray-400 shrink-0" />
                                            <DropdownMenu as="div" class="relative inline-block text-left">
                                                <div>
                                                    <DropdownMenuTrigger
                                                        class="inline-flex items-center rounded-md px-2 py-1 text-sm font-medium"
                                                        :class="getStatusBgColor(device.status)">
                                                        {{ device.status }}
                                                        <ChevronDownIcon class="ml-1 h-3 w-3" aria-hidden="true" />
                                                    </DropdownMenuTrigger>
                                                </div>
                                                <transition enter-active-class="transition ease-out duration-100"
                                                    enter-from-class="transform opacity-0 scale-95"
                                                    enter-to-class="transform opacity-100 scale-100"
                                                    leave-active-class="transition ease-in duration-75"
                                                    leave-from-class="transform opacity-100 scale-100"
                                                    leave-to-class="transform opacity-0 scale-95">
                                                    <DropdownMenuContent
                                                        class="absolute right-0 z-10 mt-1 w-24 origin-top-right rounded-md bg-white shadow-lg ring-1 ring-black ring-opacity-5 focus:outline-none">
                                                        <div class="py-1">
                                                            <DropdownMenuItem>
                                                                <button
                                                                    class="block w-full px-3 py-1 text-left text-sm text-gray-900">
                                                                    Tốt
                                                                </button>
                                                            </DropdownMenuItem>
                                                            <DropdownMenuItem>
                                                                <button
                                                                    class="block w-full px-3 py-1 text-left text-sm text-gray-900">
                                                                    Hư
                                                                </button>
                                                            </DropdownMenuItem>
                                                        </div>
                                                    </DropdownMenuContent>
                                                </transition>
                                            </DropdownMenu>
                                        </div>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div class="w-2/3 bg-gray-100 p-6">
                <h2 class="text-2xl font-medium text-gray-900 mb-2">Danh sách thiết bị</h2>

                <div class="grid grid-cols-2 gap-6 mb-6">
                    <div class="bg-white rounded-lg">
                        <h3 class="text-base font-medium text-gray-500 px-4 pt-4">Tiến độ</h3>
                        <div class="px-4 pb-4">
                            <div class="flex flex-col">
                                <div class="flex flex-col">
                                    <p class="text-lg text-gray-900">Đúng hạn: {{ stats.onTime }} cái</p>
                                </div>
                                <div class="flex flex-col">
                                    <p class="text-lg text-red-600">Trễ hạn: {{ stats.late }} cái</p>
                                </div>
                            </div>
                        </div>
                    </div>
                    <div class="bg-white rounded-lg">
                        <h3 class="text-base font-medium text-gray-500 px-4 pt-4">Tình trạng</h3>
                        <div class="px-4 pb-4">
                            <div class="flex flex-col">
                                <div class="flex flex-col">
                                    <p class="text-lg text-green-600">Tốt: {{ stats.good }} cái</p>
                                </div>
                                <div class="flex flex-col">
                                    <p class="text-lg text-red-600">Hư: {{ stats.damaged }} cái</p>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="overflow-hidden border rounded-lg bg-white">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-left text-sm font-medium text-gray-500">Mã</th>
                                <th scope="col" class="px-6 py-3 text-left text-sm font-medium text-gray-500">Tên thiết
                                    bị</th>
                                <th scope="col" class="px-6 py-3 text-right text-sm font-medium text-gray-500">SL</th>
                            </tr>
                        </thead>
                        <tbody class="bg-white divide-y divide-gray-200">
                            <tr v-for="device in devices" :key="device.id">
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{{ device.code }}</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" :src="device.image"
                                            :alt="device.name" />
                                        <span>{{ device.name }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">1</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <div class="fixed bottom-0 left-0 right-0 bg-tertiary-darker border-t">
            <div class="flex items-center justify-between px-6 py-4">
                <div class="flex flex-col">
                    <div class="flex items-center space-x-2">
                        <span class="text-sm font-medium">Nơi trả:</span>
                        <span class="text-sm font-bold">601 H6, Dĩ An</span>
                    </div>
                    <div class="flex items-center space-x-2">
                        <span class="text-sm font-medium">Tổng thiết bị:</span>
                        <span class="text-sm font-bold">20 Cái</span>
                    </div>
                </div>
                <button type="button"
                    class="bg-white text-tertiary-darker font-bold rounded-lg px-8 py-2 hover:bg-white/80"
                    @click="router.push(`/device/${route.params.id}/return/confirm`)">
                    Tiếp tục
                </button>
            </div>
        </div>
    </div>
</template>