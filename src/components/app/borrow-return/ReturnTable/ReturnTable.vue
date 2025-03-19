<script setup lang="ts">
//import { receiptService } from '~/services';
import { columns } from './column';
import { Table, type AugmentedColumnDef } from '@/components/common/table';
import { useRouter } from 'vue-router';
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { HandIcon, ArrowUpIcon, ArrowDownIcon } from 'lucide-vue-next';

const router = useRouter();

async function fetchData(offset: number, length: number, options: { desc?: boolean, sortField?: string, searchText?: string, searchFields?: string[] }): Promise<{ data: unknown[], totalPages: number }> {
  return {
    data: [],
    totalPages: 0
  }
}

</script>

<template>
  <Table :selectable="true" :searchable="true" :qrable="true" :fetch-fn="fetchData"
    :columns="columns as AugmentedColumnDef<unknown>[]">
    <template #custom-button>
      <DropdownMenu>
        <DropdownMenuTrigger as-child>
          <Button class="w-full md:w-auto bg-tertiary-dark hover:bg-tertiary-darker text-white">
            <HandIcon class="w-5 h-5 mr-2" />
            Mượn / Trả
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent class="w-56">
          <DropdownMenuItem class="cursor-pointer" @click="router.push('/admin/borrows/form')">
            <div class="flex items-center gap-2">
              <ArrowUpIcon class="w-5 h-5 text-gray-500" />
              <span>Mượn thiết bị</span>
            </div>
          </DropdownMenuItem>
          <DropdownMenuItem class="cursor-pointer" @click="router.push('/admin/returns/form')">
            <div class="flex items-center gap-2">
              <ArrowDownIcon class="w-5 h-5 text-gray-500" />
              <span>Trả thiết bị</span>
            </div>
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </template>
  </Table>
</template>