<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection'
import { Package, Scan, User } from 'lucide-vue-next'
import { userService, deviceService, type UserDetail } from '@/lib/db'
import { toast } from '@/components/ui/toast'

const router = useRouter()

const userInfo = ref<UserDetail | null>(null)

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

    userInfo.value = userMeta
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

const handleVirtualKeyboardDetection = async (input: string, type?: 'userId' | 'device') => {
  if (type === 'userId') {
    await handleUserCodeChange(input)
  } else if (type === 'device') {
    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1]
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1]

    if (!deviceId) {
      toast({ title: 'Lỗi', description: 'Không thể trích xuất ID thiết bị từ mã QR', variant: 'destructive' })
      return
    }

    const deviceStatus = await deviceService.getDeviceStatusById(deviceId)

    if (!deviceStatus) {
      toast({ title: 'Lỗi', description: `Không tìm thấy thiết bị hoặc trạng thái không hợp lệ (ID: ${deviceId})`, variant: 'destructive' })
      return
    }

    if (deviceStatus === 'healthy' || deviceStatus === 'broken') {
      router.push({
        name: 'borrow-record',
        query: { userId: userInfo.value?.id }
      })
    } else if (deviceStatus === 'borrowing') {
      router.push({
        name: 'return-record',
        query: { userId: userInfo.value?.id }
      })
    } else {
      toast({ title: 'Thông báo', description: `Thiết bị đang ở trạng thái '${deviceStatus}', không thể mượn/trả.` })
    }

  } else if (type === 'device' && !userInfo.value) {
    toast({ title: 'Cảnh báo', description: 'Vui lòng quét mã người dùng trước khi quét thiết bị.' })
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
          <div class="flex flex-col items-center justify-center py-12 text-center">
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <Package class="h-8 w-8 text-gray-400" />
            </div>
            <h3 class="text-lg font-medium mb-1">Không có thiết bị nào được ghi nhận</h3>
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
            <div v-if="!userInfo"
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
              <div class="mt-2 flex items-center">
                <img :src="userInfo.avatar || 'default-avatar.png'" alt="User avatar"
                  class="h-12 w-12 rounded-full object-cover" />
                <div class="ml-3">
                  <h4 class="text-sm font-medium text-gray-500">{{ userInfo.id }}</h4>
                  <p class="text-base font-semibold text-gray-900">{{ userInfo.name }}
                    <span class="text-sm text-gray-500 italic font-normal">
                      ({{userInfo.roles?.map(r => r.name).join(', ') || 'Không có vai trò'}})
                    </span>
                  </p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>