<script setup lang="ts">
import { ref } from 'vue'
import { CalendarIcon, TriangleAlertIcon, CheckIcon } from 'lucide-vue-next'
import { DatePicker } from 'v-calendar'
import { useRouter } from 'vue-router'
import 'v-calendar/style.css'

const router = useRouter()
const showSuccessModal = ref(false)
const formId = '#BR_12-03-2025/123-123'

const handleConfirm = () => {
    showSuccessModal.value = true
}

const goToHome = () => {
    router.push('/')
}

const viewForm = () => {
    showSuccessModal.value = false
    router.push('/device/borrow-invoice')
}

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

const returnDate = ref(new Date())
const masks = {
    input: 'DD/MM/YYYY'
}
</script>

<template>
    <div class="bg-gray-100 min-h-screen">
        <div class="flex py-6 sm:px-6 lg:px-8 gap-4">
            <div class="w-2/3">
                <h2 class="text-2xl font-medium text-gray-900 mb-2">Danh sách thiết bị</h2>

                <div class="bg-white rounded-lg p-4 mb-4">
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

            <div class="w-1/3 rounded-xl space-y-4">
                <div class="bg-white p-6 rounded-xl h-fit">
                    <h2 class="text-xl font-medium text-gray-900 mb-1">Đơn mượn</h2>

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
                        </div>

                        <div class="grid grid-cols-2 gap-4">
                            <div>
                                <label class="block text-sm text-gray-500">Ngày mượn</label>
                                <div class="p-2 bg-gray-100 rounded text-base text-gray-900 mt-1">
                                    12/03/2025
                                </div>
                            </div>
                            <div>
                                <label class="block text-sm text-gray-500">Ngày hẹn trả<span
                                        class="text-red-500">*</span></label>
                                <div class="relative border border-gray-300 rounded-md p-2 mt-1">
                                    <DatePicker v-model="returnDate" :masks="masks" :min-date="new Date()"
                                        :model-config="{ type: 'string', mask: 'DD/MM/YYYY' }"
                                        class="block w-full rounded border-gray-300 focus:border-blue-500 focus:ring-blue-500 text-sm">
                                        <template #default="{ inputValue, inputEvents }">
                                            <input :value="inputValue" v-on="inputEvents"
                                                class="block w-full rounded border-gray-300 pr-10 focus:border-blue-500 focus:ring-blue-500 text-base text-gray-900"
                                                placeholder="DD/MM/YYYY" />
                                        </template>
                                    </DatePicker>
                                    <div class="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
                                        <CalendarIcon class="h-5 w-5 text-gray-400" />
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="space-y-4 bg-white p-6 rounded-xl">
                    <h3 class="text-xl font-medium text-gray-900">Người mượn</h3>

                    <div class="bg-red-50 border border-red-200 rounded-lg p-1">
                        <div>
                            <div class="flex items-center">
                                <TriangleAlertIcon class="h-4 w-4 stroke-2 stroke-red-500 mr-2" />
                                <p class="font-bold text-sm text-red-600">Không đủ điều kiện mượn thiết bị.</p>
                            </div>
                            <p class="mt-1 text-sm text-red-600">Chỉ sinh viên học Đồ án Đa ngành mới được mượn các
                                thiết bị có điều
                                kiện
                                này. Vui lòng
                                bỏ thiết bị đó khỏi danh sách để tiếp tục.</p>
                        </div>
                    </div>

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

                <div class="mt-8">
                    <button type="button"
                        class="w-full bg-blue-600 text-white rounded-lg py-2 px-4 hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
                        @click="handleConfirm">
                        Xác nhận mượn
                    </button>
                </div>
            </div>
        </div>

        <div v-if="showSuccessModal" class="fixed inset-0 bg-gray-500 bg-opacity-75 transition-opacity">
            <div class="fixed inset-0 z-10 w-screen overflow-y-auto">
                <div class="flex min-h-full items-end justify-center p-4 text-center sm:items-center sm:p-0">
                    <div
                        class="relative transform overflow-hidden rounded-lg bg-white px-4 pb-4 pt-5 text-left shadow-xl transition-all sm:my-8 sm:w-full sm:max-w-lg sm:p-6">
                        <div>
                            <div class="mx-auto flex h-24 w-24 items-center justify-center rounded-full bg-green-100">
                                <CheckIcon class="h-16 w-16 text-green-600" />
                            </div>
                            <div class="mt-3 text-center sm:mt-5">
                                <h3 class="text-2xl font-semibold leading-6 text-gray-900">Hoàn tất</h3>
                                <div class="mt-4">
                                    <p class="text-xl text-gray-900">Ghi nhận mượn thành công</p>
                                    <p class="mt-2 text-base text-gray-600">Mã đơn: {{ formId }}</p>
                                </div>
                            </div>
                        </div>
                        <div class="mt-8 sm:grid sm:grid-flow-row-dense sm:grid-cols-2 sm:gap-3">
                            <button type="button"
                                class="inline-flex w-full justify-center rounded-md bg-gray-100 px-3 py-2 text-sm font-semibold text-gray-900 shadow-sm hover:bg-gray-200 sm:col-start-1"
                                @click="viewForm">
                                Xem lại đơn
                            </button>
                            <button type="button"
                                class="mt-3 inline-flex w-full justify-center rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 sm:col-start-2 sm:mt-0"
                                @click="goToHome">
                                Về trang chủ
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>