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
  const baseUrl =
    import.meta.env.VITE_API_BASE_URL || "http://localhost:3000";
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
    class="flex-1 flex flex-col items-center justify-center px-4 py-4 text-white overflow-hidden"
  >
    <Transition name="fade" mode="out-in">
      <div
        v-if="showSuccessScreen"
        key="success"
        class="flex flex-col items-center justify-center text-center w-full h-full"
      >
        <CheckCircle class="h-24 w-24 text-white mb-6" />
        <h1 class="text-4xl font-bold mb-4">Đăng nhập thành công</h1>
        <p class="text-lg">Đang chuyển hướng đến trang chính...</p>
      </div>

      <div v-else key="login" class="flex flex-col items-center w-full">
        <h1 class="text-4xl font-bold">Quét mã QR đăng nhập</h1>
        <p class="text-lg mb-4">
          Người dùng quét mã QR này trên thiết bị di động để đăng nhập.
        </p>

        <div class="h-6 mb-4">
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

        <div class="bg-white p-4 rounded-lg mb-4">
          <QRCode
            v-if="hmiCode"
            :value="qrCodeUrl"
            :options="qrOptions"
            class="!w-64 !h-64"
          />
          <div v-else class="w-64 h-64 flex items-center justify-center">
            <p class="text-gray-400">Đang tạo mã QR...</p>
          </div>
        </div>

        <div class="flex items-center gap-4 mb-4">
          <div class="w-16 h-[1px] bg-gray-500"></div>
          <span>hoặc</span>
          <div class="w-16 h-[1px] bg-gray-500"></div>
        </div>

        <div
          class="bg-white text-black rounded-lg w-full max-w-xl p-4 flex justify-between items-center"
        >
          <div>
            <p class="text-gray-700 mb-1">Truy cập và nhập mã thiết bị:</p>
            <p class="font-bold">{{ loginUrl }}</p>
          </div>

          <div class="text-right">
            <p class="text-gray-700 mb-1">Mã thiết bị:</p>
            <p class="text-5xl font-bold">{{ hmiCode }}</p>
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
