<script setup lang="ts">
import { useAuth } from "@/composables";
import { onMounted, onUnmounted, ref, watchEffect } from "vue";
import { useRouter } from "vue-router";

const loginUrl = ref("www.ngyngcphu.labsyncro.com");

const {
  hmiCode,
  generateHMICode,
  startLoginPolling,
  stopLoginPolling,
  loginStatus,
  isLoading,
  error,
  isGeneratingCode,
} = useAuth();
const router = useRouter();

onMounted(async () => {
  await generateHMICode();
  if (hmiCode.value) {
    startLoginPolling(hmiCode.value);
  }
});

watchEffect(() => {
  if (loginStatus.value === "success") {
    router.push({ name: "home" });
  }
});

onUnmounted(() => {
  stopLoginPolling();
});

const getQrCodeUrl = () => {
  return "/qr-webapp.svg";
};
</script>

<template>
  <main
    class="flex-1 flex flex-col items-center justify-center px-4 py-4 text-white"
  >
    <h1 class="text-4xl font-bold">Quét mã QR đăng nhập</h1>
    <p class="text-lg mb-4">
      Người dùng quét mã QR này trên thiết bị di động để đăng nhập.
    </p>

    <div v-if="isGeneratingCode" class="mb-4 text-blue-400">Đang tạo mã...</div>
    <div v-else-if="isLoading" class="mb-4 text-yellow-400">
      Đang chờ đăng nhập từ thiết bị di động...
    </div>
    <div v-if="error" class="mb-4 text-red-400">Lỗi: {{ error }}</div>

    <div class="bg-white p-4 rounded-lg mb-4">
      <img :src="getQrCodeUrl()" alt="QR Code" class="w-64 h-64" />
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
  </main>
</template>
