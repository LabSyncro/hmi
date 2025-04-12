<script setup lang="ts">
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useAuth } from "@/composables/useAuth";
import type { AcceptableValue } from "reka-ui";
import { onMounted, ref } from "vue";

const selectedLocation = ref("601 H6, Dĩ An");
const locations = ["601 H6, Dĩ An", "602 H6, Dĩ An", "603 H6, Dĩ An"];
const loginUrl = ref("www.ngyngcphu.labsyncro.com");

const { deviceCode, generateDeviceCode } = useAuth();

const handleLocationChange = (value: AcceptableValue) => {
  selectedLocation.value = value as string;
};

onMounted(() => {
  generateDeviceCode();
});

const getQrCodeUrl = () => {
  return "/qr-webapp.svg";
};
</script>

<template>
  <div class="min-h-screen bg-[#060b28] text-white flex flex-col">
    <header class="p-4 flex justify-between items-center">
      <div class="flex items-center gap-2">
        <div
          class="h-8 w-8 bg-white text-[#060b28] flex items-center justify-center font-bold"
        >
          <span class="text-sm">L</span>
          <span class="text-sm">A</span>
          <span class="text-sm">B</span>
        </div>
        <h1 class="text-xl font-bold">Lab Syncro</h1>
      </div>

      <div class="flex items-center gap-2">
        <span>Địa điểm:</span>
        <Select
          :model-value="selectedLocation"
          @update:model-value="handleLocationChange"
        >
          <SelectTrigger
            class="w-[180px] bg-white text-black border border-gray-200 shadow-sm"
          >
            <SelectValue :placeholder="selectedLocation" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem
              v-for="location in locations"
              :key="location"
              :value="location"
            >
              {{ location }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
    </header>

    <main class="flex-1 flex flex-col items-center justify-center px-4 py-12">
      <h1 class="text-4xl font-bold mb-4">Quét mã QR đăng nhập</h1>
      <p class="text-lg mb-8">
        Người dùng quét mã QR này trên thiết bị di động để đăng nhập.
      </p>

      <div class="bg-white p-4 rounded-lg mb-8">
        <img :src="getQrCodeUrl()" alt="QR Code" class="w-64 h-64" />
      </div>

      <div class="flex items-center gap-4 mb-8">
        <div class="w-16 h-[1px] bg-gray-500"></div>
        <span>hoặc</span>
        <div class="w-16 h-[1px] bg-gray-500"></div>
      </div>

      <div
        class="bg-white text-black rounded-lg w-full max-w-xl p-6 flex justify-between items-center"
      >
        <div>
          <p class="text-gray-700 mb-1">Truy cập và nhập mã thiết bị:</p>
          <p class="font-bold">{{ loginUrl }}</p>
        </div>

        <div class="text-right">
          <p class="text-gray-700 mb-1">Mã thiết bị:</p>
          <p class="text-5xl font-bold">{{ deviceCode }}</p>
        </div>
      </div>
    </main>
  </div>
</template>
