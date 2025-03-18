<template>
  <div class="py-6">
    <div class="text-center">
      <h1 class="text-2xl font-bold mb-4">Device Scanner</h1>
      <p class="text-gray-600">Scan a device QR code to view details</p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useVirtualKeyboardDetection } from '@/hooks/useVirtualKeyboardDetection';

const router = useRouter();

const handleVirtualKeyboardDetection = async (input: string, type?: 'userId' | 'device') => {
  if (type === 'device') {
    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1];
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1];

    if (deviceKindId && deviceId) {
      router.push({
        name: 'device-detail',
        params: { id: deviceId },
        query: { deviceKindId }
      });
    }
  }
};

useVirtualKeyboardDetection(handleVirtualKeyboardDetection, {
  device: { pattern: /^https?:\/\/[^/]+\/devices\/[a-fA-F0-9]{8}\?id=[a-fA-F0-9]+$/ },
  scannerThresholdMs: 100,
  maxInputTimeMs: 1000,
});
</script>