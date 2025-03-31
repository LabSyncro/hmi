<script setup lang="ts">
import { receiptService } from '@/lib/db';
import { columns, type ReturnedReceiptDeviceSchema } from './column';
import { Table, type AugmentedColumnDef } from '@/components/common/table';

async function fetchData(offset: number, length: number, options: { desc?: boolean, sortField?: string })
  : Promise<{ data: ReturnedReceiptDeviceSchema[], totalPages: number, totalCount: number }> {
  const data = await receiptService.fetchReturnedDevices(offset, length, options);
  return data;
}
</script>

<template>
  <Table :fetch-fn="fetchData" :columns="columns as AugmentedColumnDef<any>[]" />
</template>