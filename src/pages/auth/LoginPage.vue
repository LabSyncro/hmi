<script setup lang="ts">
import { useAuth } from "@/composables";
import { CheckCircle } from "lucide-vue-next";
import QRCode from "qrcode.vue";
import { computed, onMounted, onUnmounted, ref, watchEffect } from "vue";
import { useRouter } from "vue-router";

const loginUrl = ref("www.ngyngcphu.labsyncro.com");
const showSuccessScreen = ref(false);

const {
  hmiCode,
  generateHMICode,
  startLoginPolling,
  stopLoginPolling,
  loginStatus,
  isLoading,
  error,
  isGeneratingCode,
  labInfo,
} = useAuth();
const router = useRouter();

const qrCodeUrl = computed(() => {
  const code = hmiCode.value ? hmiCode.value.replace(/\s/g, "") : "";
  const baseUrl = import.meta.env.VITE_API_BASE_URL || "http://localhost:3000";
  return `${baseUrl}/auth/hmi?hmiCode=${code}`;
});

const qrOptions = {
  color: {
    dark: "#000000FF",
    light: "#FFFFFFFF",
  },
};

onMounted(async () => {
  await generateHMICode();
  if (hmiCode.value) {
    startLoginPolling(hmiCode.value);
  }
});

watchEffect(() => {
  if (loginStatus.value === "success" && labInfo.value) {
    showSuccessScreen.value = true;
    setTimeout(() => {
      router.push({ name: "home" });
    }, 2000);
  } else {
    showSuccessScreen.value = false;
  }
});

onUnmounted(() => {
  stopLoginPolling();
});
</script>

<template>
  <main
    class="flex-1 flex flex-col items-center justify-center px-3 py-2 text-white overflow-hidden"
  >
    <Transition name="fade" mode="out-in">
      <div
        v-if="showSuccessScreen"
        key="success"
        class="flex flex-col items-center justify-center text-center w-full h-full"
      >
        <CheckCircle class="h-16 w-16 text-white mb-4" />
        <h1 class="text-2xl font-bold mb-3">Đăng nhập thành công</h1>
        <p class="text-base">Đang chuyển hướng đến trang chính...</p>
      </div>

      <div v-else key="login" class="flex flex-col items-center w-full">
        <h1 class="text-2xl font-bold">Quét mã QR đăng nhập</h1>
        <p class="text-base mb-3">
          Người dùng quét mã QR này trên thiết bị di động để đăng nhập.
        </p>

        <div class="h-6 mb-3">
          <Transition name="fade" mode="out-in">
            <div v-if="isGeneratingCode" key="generating" class="text-blue-400">
              Đang tạo mã...
            </div>
            <div v-else-if="isLoading" key="loading" class="text-yellow-400">
              <template v-if="loginStatus === 'awaiting_lab'">
                Vui lòng chọn phòng thí nghiệm trên ứng dụng web...
              </template>
              <template v-else>
                Đang chờ đăng nhập từ thiết bị di động...
              </template>
            </div>
            <div v-else-if="error" key="error" class="text-red-400">
              Lỗi: {{ error }}
            </div>
            <div v-else key="placeholder"></div>
          </Transition>
        </div>

        <div class="bg-white p-3 rounded-lg mb-3">
          <QRCode
            v-if="hmiCode"
            :value="qrCodeUrl"
            :options="qrOptions"
            class="!w-48 !h-48"
          />
          <div v-else class="w-48 h-48 flex items-center justify-center">
            <p class="text-gray-400">Đang tạo mã QR...</p>
          </div>
        </div>

        <div class="flex items-center gap-3 mb-3">
          <div class="w-14 h-[1px] bg-gray-500"></div>
          <span class="text-sm">hoặc</span>
          <div class="w-14 h-[1px] bg-gray-500"></div>
        </div>

        <div
          class="bg-white text-black rounded-lg w-full max-w-lg p-3 flex justify-between items-center"
        >
          <div>
            <p class="text-gray-700 mb-1 text-sm">
              Truy cập và nhập mã thiết bị:
            </p>
            <p class="font-bold text-sm">{{ loginUrl }}</p>
          </div>

          <div class="text-right">
            <p class="text-gray-700 mb-1 text-sm">Mã thiết bị:</p>
            <p class="text-3xl font-bold">{{ hmiCode }}</p>
          </div>
        </div>
      </div>
    </Transition>
  </main>
</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.5s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
