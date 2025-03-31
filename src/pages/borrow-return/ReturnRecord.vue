<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ChevronDown, Trash, Box, User, Calendar, MapPin, PackageCheck } from 'lucide-vue-next'
import BorrowReturnLayout from '@/layouts/BorrowReturnLayout.vue'

const router = useRouter()

// Sample data for the return record
const devices = ref([
  {
    code: '123-123',
    name: 'Tên thiết bị A',
    quantity: 3,
    expanded: true,
    items: [
      { id: '123-123/123-456', status: 'Tốt', condition: 'Bình thường' },
      { id: '123-123/123-457', status: 'Hỏng', condition: 'Hư hỏng nhẹ' },
      { id: '123-123/123-459', status: 'Tốt', condition: 'Bình thường' },
    ]
  },
  {
    code: '123-124',
    name: 'Tên thiết bị B',
    quantity: 1,
    expanded: false,
    items: [
      { id: '123-124/456-789', status: 'Tốt', condition: 'Bình thường' },
    ]
  },
])

// Borrower information
const borrower = ref({
  id: '2111243',
  name: 'Nguyễn Văn A',
  role: 'Sinh viên',
  avatar: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?ixlib=rb-1.2.1&ixid=eyJhcHBfaWQiOjEyMDd9&auto=format&fit=facearea&facepad=2&w=256&h=256&q=80',
})

// Return details
const returnDetails = ref({
  totalDevices: 20,
  location: '601 H6, Dĩ An',
  borrowDate: '12/03/2025',
  returnDate: '16/03/2025',
  actualReturnDate: '15/03/2025',
})

// Toggle device expansion
const toggleDevice = (device: any) => {
  device.expanded = !device.expanded
}

// Remove a device item
const removeDeviceItem = (device: any, itemId: string) => {
  device.items = device.items.filter((item: any) => item.id !== itemId)
  device.quantity = device.items.length
  
  // If no items left, remove the device from the list
  if (device.items.length === 0) {
    devices.value = devices.value.filter(d => d.code !== device.code)
  }
}

// Confirm return
const confirmReturn = () => {
  // Navigate to confirmation page or submit data
  router.push({
    name: 'confirm-return',
    params: { id: '123' } // Replace with actual ID
  })
}

// Calculate total devices
const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0)
})
</script>

<template>
  <BorrowReturnLayout
    title="GHI NHẬN TRẢ"
    left-column-title="DANH SÁCH TRẢ"
    right-column-title="THÔNG TIN TRẢ"
    :left-icon="PackageCheck"
    :right-icon="User"
  >
    <!-- Left Column Content -->
    <template #left-column>
      <div class="divide-y divide-gray-200">
        <div 
          v-for="device in devices" 
          :key="device.code" 
          class="divide-y divide-gray-100"
        >
          <!-- Device Header -->
          <div 
            class="p-4 hover:bg-gray-50 cursor-pointer"
            @click="toggleDevice(device)"
          >
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <Box class="h-5 w-5 text-gray-500" />
                <div>
                  <h3 class="font-medium text-gray-900">{{ device.name }}</h3>
                  <p class="text-sm text-gray-500">Mã: {{ device.code }}</p>
                </div>
              </div>
              <div class="flex items-center gap-4">
                <span class="text-sm text-gray-500">
                  {{ device.quantity }} cái
                </span>
                <ChevronDown 
                  class="h-5 w-5 text-gray-400 transition-transform"
                  :class="{ 'rotate-180': device.expanded }"
                />
              </div>
            </div>
          </div>
          
          <!-- Device Items -->
          <div v-if="device.expanded" class="bg-gray-50 divide-y divide-gray-100">
            <div 
              v-for="item in device.items" 
              :key="item.id"
              class="p-4"
            >
              <div class="flex items-center">
                <div class="flex-1">
                  <p class="text-sm font-medium text-gray-700">{{ item.id }}</p>
                  <div class="flex gap-2 mt-1">
                    <span 
                      class="text-xs px-2 py-1 rounded-full"
                      :class="item.status === 'Tốt' ? 'bg-green-100 text-green-700' : 'bg-red-100 text-red-700'"
                    >
                      {{ item.status }}
                    </span>
                    <span class="text-xs text-gray-500">
                      {{ item.condition }}
                    </span>
                  </div>
                </div>
                <button 
                  @click.stop="removeDeviceItem(device, item.id)"
                  class="text-gray-400 hover:text-red-500 transition-colors p-1 rounded-full hover:bg-gray-100"
                  aria-label="Remove device"
                >
                  <Trash class="h-4 w-4" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </template>

    <!-- Right Column Content -->
    <template #right-column>
      <div class="p-4 space-y-6">
        <!-- Borrower Information -->
        <div class="flex flex-col items-center p-4 bg-gray-50 rounded-lg">
          <div class="w-20 h-20 rounded-full overflow-hidden border-4 border-white shadow-sm mb-3">
            <img :src="borrower.avatar" alt="User avatar" class="w-full h-full object-cover" />
          </div>
          <div class="flex flex-col items-center text-center">
            <p class="font-medium text-gray-500">ID: {{ borrower.id }}</p>
            <p class="font-semibold text-gray-800 text-lg">{{ borrower.name }}</p>
            <p class="text-sm text-gray-500">{{ borrower.role }}</p>
          </div>
        </div>
        
        <!-- Return Details -->
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
              <p class="font-medium text-gray-800">{{ returnDetails.location }}</p>
            </div>
          </div>
          
          <div class="flex items-center gap-3">
            <div class="rounded-full bg-green-50 p-2">
              <Calendar class="h-4 w-4 text-green-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Ngày mượn</p>
              <p class="font-medium text-gray-800">{{ returnDetails.borrowDate }}</p>
            </div>
          </div>
          
          <div class="flex items-center gap-3">
            <div class="rounded-full bg-purple-50 p-2">
              <Calendar class="h-4 w-4 text-purple-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Ngày hẹn trả</p>
              <p class="font-medium text-gray-800">{{ returnDetails.returnDate }}</p>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <div class="rounded-full bg-indigo-50 p-2">
              <Calendar class="h-4 w-4 text-indigo-600" />
            </div>
            <div class="flex-1">
              <p class="text-sm text-gray-500">Ngày trả thực tế</p>
              <p class="font-medium text-gray-800">{{ returnDetails.actualReturnDate }}</p>
            </div>
          </div>
        </div>
        
        <!-- Confirm Button -->
        <button 
          @click="confirmReturn"
          class="w-full mt-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2"
        >
          <PackageCheck class="h-5 w-5" />
          Xác nhận trả
        </button>
      </div>
    </template>
  </BorrowReturnLayout>
</template>