<script setup lang="ts">
import { receiptService } from '@/lib/db/receipt';
import { columns, type BorrowReturnDeviceSchema } from './column';
import { Table, type AugmentedColumnDef } from '@/components/common/table';

async function fetchData(offset: number, length: number, options: { desc?: boolean, sortField?: string }): Promise<{ data: BorrowReturnDeviceSchema[], totalPages: number, totalCount: number }> {
  return await receiptService.fetchBorrowingDevices(offset, length, options);
}
</script>

<template>
  <Table :fetch-fn="fetchData" :columns="columns as AugmentedColumnDef<any>[]" />
</template>