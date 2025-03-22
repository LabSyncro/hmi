<script setup lang="ts">
import { receiptService } from '@/lib/db/receipt';
import { columns, type ReadyBorrowedDeviceSchema } from './column';
import { Table, type AugmentedColumnDef } from '@/components/common/table';

async function fetchData(offset: number, length: number, options: { desc?: boolean, sortField?: string }): Promise<{ data: ReadyBorrowedDeviceSchema[], totalPages: number, totalCount: number }> {
  return await receiptService.fetchReadyBorrowDevices(offset, length, options);
}
</script>

<template>
  <Table :fetch-fn="fetchData" :columns="columns as AugmentedColumnDef<any>[]" />
</template>