<script setup lang="ts">
// import { receiptService } from '~/services';
import { columns } from './column';
import { Table, type AugmentedColumnDef } from '@/components/common/table';
import { useRouter } from 'vue-router';

const router = useRouter();

async function fetchData(offset: number, length: number, options: { desc?: boolean, sortField?: string, searchText?: string, searchFields?: string[] }): Promise<{ data: unknown[], totalPages: number }> {
  return {
    data: [],
    totalPages: 0
  }
}
</script>

<template>
  <Table :fetch-fn="fetchData" :columns="columns as AugmentedColumnDef<unknown>[]">
  </Table>
</template>