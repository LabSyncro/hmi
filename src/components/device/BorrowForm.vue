<script setup lang="ts">
import { ref } from 'vue'
import { TransitionRoot } from '@headlessui/vue'
import { useRouter, useRoute } from 'vue-router'
import {
    ChevronDownIcon,
    ChevronUpIcon,
    InformationCircleIcon,
    TrashIcon
} from '@heroicons/vue/24/outline'

const router = useRouter()
const route = useRoute()
const showMore = ref(false)
</script>

<template>
    <div class="bg-gray-50">
        <!-- Main Content -->
        <div class="flex overflow-hidden">
            <!-- Left Panel -->
            <div class="w-1/3 border-r bg-white p-6">
                <h2 class="text-2xl font-medium text-gray-900 mb-8">Chi tiết thiết bị</h2>
                <div class="space-y-6">
                    <!-- Device Header -->
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

                    <!-- Basic Info -->
                    <div class="space-y-4">
                        <div class="grid grid-cols-3 gap-2 text-base mb-4">
                            <span class="text-gray-500">Quyền mượn</span>
                            <span class="text-gray-900 col-span-2">Sinh viên, Giảng viên</span>
                        </div>

                        <TransitionRoot as="template" :show="showMore">
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
                        </TransitionRoot>

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

                    <!-- Device Status Table -->
                    <div class="border rounded-lg overflow-hidden">
                        <table class="min-w-full divide-y divide-gray-200">
                            <thead class="bg-gray-50">
                                <tr>
                                    <th scope="col" class="px-16 py-3 text-left text-sm font-medium text-gray-500">Mã
                                    </th>
                                    <th scope="col" class="px-6 py-3 text-left text-sm font-medium text-gray-500">Tình
                                        trạng</th>
                                </tr>
                            </thead>
                            <tbody class="bg-white divide-y divide-gray-200">
                                <tr>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                                        <div class="flex items-center">
                                            <TrashIcon class="h-5 w-5 text-red-500 mr-3" />
                                            <span class="text-gray-900">123-123/123-446</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <div class="flex items-center">
                                            <span class="px-2 py-1 text-xs rounded bg-red-50 text-red-700">Hư</span>
                                            <button class="ml-2">
                                                <InformationCircleIcon class="h-4 w-4 text-gray-400" />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                                        <div class="flex items-center">
                                            <TrashIcon class="h-5 w-5 text-red-500 mr-3" />
                                            <span class="text-gray-900">123-123/123-447</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <div class="flex items-center">
                                            <span class="px-2 py-1 text-xs rounded bg-red-50 text-red-700">Hư</span>
                                            <button class="ml-2">
                                                <InformationCircleIcon class="h-4 w-4 text-gray-400" />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                                        <div class="flex items-center">
                                            <TrashIcon class="h-5 w-5 text-red-500 mr-3" />
                                            <span class="text-gray-900">123-123/123-448</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <div class="flex items-center">
                                            <span class="px-2 py-1 text-xs rounded bg-red-50 text-red-700">Hư</span>
                                            <button class="ml-2">
                                                <InformationCircleIcon class="h-4 w-4 text-gray-400" />
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                                        <div class="flex items-center">
                                            <TrashIcon class="h-5 w-5 text-red-500 mr-3" />
                                            <span class="text-gray-900">123-123/123-449</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <div class="flex items-center">
                                            <span
                                                class="px-2 py-1 text-xs rounded bg-green-50 text-green-700">Tốt</span>
                                        </div>
                                    </td>
                                </tr>
                                <tr>
                                    <td class="px-6 py-4 whitespace-nowrap text-sm">
                                        <div class="flex items-center">
                                            <TrashIcon class="h-5 w-5 text-red-500 mr-3" />
                                            <span class="text-gray-900">123-123/123-450</span>
                                        </div>
                                    </td>
                                    <td class="px-6 py-4 whitespace-nowrap">
                                        <div class="flex items-center">
                                            <span
                                                class="px-2 py-1 text-xs rounded bg-green-50 text-green-700">Tốt</span>
                                        </div>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <!-- Right Panel -->
            <div class="w-2/3 bg-gray-100 p-6">
                <h2 class="text-2xl font-medium text-gray-900 mb-2">Danh sách thiết bị</h2>

                <!-- Info Cards -->
                <div class="bg-white rounded-lg p-4 mb-4 ">
                    <div class="grid grid-cols-3 gap-8">
                        <div>
                            <h3 class="text-base text-gray-500">Quyền mượn:</h3>
                            <p class="mt-1 text-base text-gray-900">Sinh viên (Năm Ba)</p>
                        </div>
                        <div>
                            <h3 class="text-base text-gray-500">Môn đã học:</h3>
                            <p class="mt-1 text-base text-gray-900">Mạch điện - Điện tử</p>
                        </div>
                        <div>
                            <h3 class="text-base text-gray-500">Môn đang học:</h3>
                            <div class="mt-1 text-base text-gray-900">
                                <p>Đồ án Đa ngành</p>
                                <p>Hệ thống số</p>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Device List Table -->
                <div class="overflow-hidden border rounded-lg">
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
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">123-123</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" src="/device-image.svg"
                                            alt="" />
                                        <span>Raspberry Pi 3 GPIO-232 Mạch Mở Rộng Đa Chức Năng</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">10</td>
                            </tr>
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">123-124</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" src="/device-image.svg"
                                            alt="" />
                                        <span>Vinasemi 3005D Máy Cấp Nguồn DC 30V 5A</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">1</td>
                            </tr>
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">123-125</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" src="/device-image.svg"
                                            alt="" />
                                        <span>BeagleBone Black(Rev C)</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">1</td>
                            </tr>
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">123-127</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" src="/device-image.svg"
                                            alt="" />
                                        <span>Arduino Uno Xbee Shield</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">1</td>
                            </tr>
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">123-128</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" src="/device-image.svg"
                                            alt="" />
                                        <span>Vinasemi 938D Máy Hàn Trạm Dạng Nhíp 70W, 220VAC, 100-480°C</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">1</td>
                            </tr>
                            <tr>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">123-129</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" src="/device-image.svg"
                                            alt="" />
                                        <span>Vinasemi 192 Đồng Hồ Đo Nhiệt Độ Dùng Cho Máy Hàn, Khò</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">1</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>
        </div>

        <!-- Bottom Bar -->
        <div class="fixed bottom-0 left-0 right-0 bg-tertiary-darker border-t">
            <div class="flex items-center justify-between px-6 py-4">
                <div class="flex flex-col">
                    <div class="flex items-center space-x-2">
                        <span class="text-sm font-medium">Nơi mượn:</span>
                        <span class="text-sm font-bold">601 H6, Dĩ An</span>
                    </div>
                    <div class="flex items-center space-x-2">
                        <span class="text-sm font-medium">Tổng thiết bị:</span>
                        <span class="text-sm font-bold">20 Cái</span>
                    </div>
                </div>
                <button type="button"
                    class="bg-white text-tertiary-darker font-bold rounded-lg px-8 py-2 hover:bg-white/80"
                    @click="router.push(`/device/${route.params.id}/borrow/confirm`)">
                    Tiếp tục
                </button>
            </div>
        </div>
    </div>
</template>