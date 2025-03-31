<script setup lang="ts">
import { receiptService } from '@/lib/db';
import { columns, type BorrowReturnDeviceSchema } from './column';
import { Table, type AugmentedColumnDef } from '@/components/common/table';

async function fetchData(offset: number, length: number, options: { desc?: boolean, sortField?: string })
  : Promise<{ data: BorrowReturnDeviceSchema[], totalPages: number, totalCount: number }> {
  const data = await receiptService.fetchBorrowingDevices(offset, length, options);
  return data;
}
</script>

<template>
  <Table :fetch-fn="fetchData" :columns="columns as AugmentedColumnDef<any>[]" />
</template>