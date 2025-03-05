<script setup lang="ts">
const formId = '#RT_18-03-2025/123-123'

const devices = [
    {
        id: 1,
        code: '123-123',
        name: 'Raspberry Pi 3 GPIO-232 Mạch Mở Rộng...',
        image: '/device-image.svg',
        quantity: 10
    },
    {
        id: 2,
        code: '123-124',
        name: 'Vinasemi 3005D Máy Cấp Nguồn DC 30V...',
        image: '/device-image.svg',
        quantity: 1
    },
    {
        id: 3,
        code: '123-125',
        name: 'BeagleBone Black(Rev C)',
        image: '/device-image.svg',
        quantity: 1
    },
    {
        id: 4,
        code: '123-127',
        name: 'Arduino Uno Xbee Shield',
        image: '/device-image.svg',
        quantity: 1
    },
    {
        id: 5,
        code: '123-128',
        name: 'Vinasemi 938D Máy Hàn Trạm Dạng Nhíp...',
        image: '/device-image.svg',
        quantity: 1
    },
    {
        id: 6,
        code: '123-129',
        name: 'Vinasemi 192 Đồng Hồ Đo Nhiệt Độ Dùng...',
        image: '/device-image.svg',
        quantity: 1
    }
]

const stats = {
    onTime: 17,
    late: 3,
    good: 15,
    damaged: 5
}

</script>

<template>
    <div class="bg-gray-100 min-h-screen">
        <div class="flex py-6 sm:px-6 lg:px-8 gap-4">
            <!-- Left Panel - Device List -->
            <div class="w-2/3">
                <h2 class="text-2xl font-medium text-gray-900 mb-2">Danh sách thiết bị</h2>

                <!-- Stats Cards -->
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

                <!-- Device Table -->
                <div class="bg-white rounded-lg overflow-hidden border">
                    <table class="min-w-full divide-y divide-gray-200">
                        <thead class="bg-gray-50">
                            <tr>
                                <th scope="col" class="px-6 py-3 text-left text-sm font-medium text-gray-500">Mã</th>
                                <th scope="col" class="px-6 py-3 text-left text-sm font-medium text-gray-500">Tên thiết
                                    bị</th>
                                <th scope="col" class="px-6 py-3 text-right text-sm font-medium text-gray-500">SL</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-gray-200">
                            <tr v-for="device in devices" :key="device.id" class="bg-white">
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">{{ device.code }}</td>
                                <td class="px-6 py-4 text-sm text-gray-900">
                                    <div class="flex items-center">
                                        <img class="h-12 w-12 rounded object-cover mr-3" :src="device.image"
                                            :alt="device.name" />
                                        <span>{{ device.name }}</span>
                                    </div>
                                </td>
                                <td class="px-6 py-4 whitespace-nowrap text-sm text-right text-gray-900">{{
                                    device.quantity }}</td>
                            </tr>
                        </tbody>
                    </table>
                </div>
            </div>

            <!-- Right Panel - Borrow Form -->
            <div class="w-1/3 rounded-xl space-y-4">
                <div class="bg-white p-6 rounded-xl h-fit">
                    <h2 class="text-xl font-medium text-gray-900 mb-1">Đơn trả</h2>

                    <div class="mb-8">
                        <p class="text-base text-gray-500">{{ formId }}</p>
                    </div>

                    <div class="space-y-2">
                        <div class="space-y-2">
                            <div class="grid grid-cols-2">
                                <label class="block text-sm text-gray-500">Tổng thiết bị</label>
                                <p class="text-base font-medium text-gray-900 ">20 cái</p>
                            </div>
                            <div class="grid grid-cols-2">
                                <label class="block text-sm text-gray-500">Nơi mượn</label>
                                <p class="text-base font-medium text-gray-900">601 H6, Dĩ An</p>
                            </div>
                            <div class="grid grid-cols-2">
                                <label class="block text-sm text-gray-500">Nơi trả</label>
                                <p class="text-base font-medium text-gray-900">601 H6, Dĩ An</p>
                            </div>
                            <div class="grid grid-cols-2">
                                <label class="block text-sm text-gray-500">Ngày mượn</label>
                                <div class="text-base text-gray-900 font-medium">
                                    12/03/2025
                                </div>
                            </div>
                            <div class="grid grid-cols-2">
                                <label class="block text-sm text-gray-500">Ngày hẹn trả</label>
                                <div class="text-base text-gray-900 font-medium">
                                    18/03/2025
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Borrower Info -->
                <div class="space-y-4 bg-white p-6 rounded-xl">
                    <h3 class="text-xl font-medium text-gray-900">Người trả</h3>

                    <div class="space-y-2">
                        <div class="grid grid-cols-2">
                            <label class="block text-sm text-gray-500">Mã số</label>
                            <p class="text-base font-medium text-gray-900">2244134</p>
                        </div>
                        <div class="grid grid-cols-2">
                            <label class="block text-sm text-gray-500">Người mượn</label>
                            <p class="text-base font-medium text-gray-900">Nguyễn Thị A</p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>