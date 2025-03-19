<script setup lang="ts">
import { debounce } from 'lodash-es';
import { createColumns, type AugmentedColumnDef } from './column';
import { ref, onMounted, watch, nextTick } from 'vue';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Dialog, DialogContent, DialogFooter } from '@/components/ui/dialog';
import { Search, QrCode, Plus } from 'lucide-vue-next';
import TableCore from './Core.vue';

const props = defineProps<{
  deleteFn?: (ids: string[]) => Promise<void>;
  fetchFn: (offset: number, length: number, options: { desc?: boolean, sortField?: string, searchText?: string }) => Promise<{ data: unknown[], totalPages: number }>,
  addTriggerFn?: () => void,
  addTitle?: string;
  columns: AugmentedColumnDef<unknown>[],
  qrable: boolean;
  searchable: boolean;
  selectable: boolean;
}>();

const searchText = ref('');
const filterBoxRef = ref<HTMLInputElement | null>(null);

const pageIndex = ref(0);
const pageSize = ref(10);
function handlePageIndexChange(value: number) {
  pageIndex.value = value;
}
function handlePageSizeChange(value: number) {
  pageSize.value = value;
}
const pageCount = ref(0);

const sortField = ref<string | null>(null);
const sortOrder = ref<'desc' | 'asc' | null>(null);

function handleSortFieldChange(value: string | null) {
  sortField.value = value;
}
function handleSortOrderChange(value: 'desc' | 'asc' | null) {
  sortOrder.value = value;
}

const rowSelection = ref<string[]>([]);
function onSelectRows(ids: string[]) {
  for (const id of ids) {
    const index = rowSelection.value.indexOf(id);
    if (index >= 0) {
      rowSelection.value.splice(index, 1);
    } else {
      rowSelection.value.push(id);
    }
  }
}
function onSelectAllRows(ids: string[]) {
  if (ids.every((id) => rowSelection.value.includes(id))) {
    ids.forEach((id) => rowSelection.value.splice(rowSelection.value.indexOf(id)));
    return;
  }
  for (const id of ids) {
    const index = rowSelection.value.indexOf(id);
    if (index === -1) {
      rowSelection.value.push(id);
    }
  }
}

const rowsToDelete = ref<string[]>([]);
const isDeleteModalActive = ref(false);
function onDeleteSelectedRows() {
  isDeleteModalActive.value = true;
  rowsToDelete.value = [...rowSelection.value];
}
function onDeleteRow(id: string) {
  isDeleteModalActive.value = true;
  rowsToDelete.value = [id];
}
async function onConfirmDelete() {
  await props.deleteFn!(rowsToDelete.value);
  rowsToDelete.value.forEach((id) => {
    const index = rowSelection.value.indexOf(id);
    if (index > -1) rowSelection.value.splice(index);
  });
  updateData();
  rowsToDelete.value = [];
  isDeleteModalActive.value = false;
}
function closeDeleteModal() {
  isDeleteModalActive.value = false;
}

const data = ref<unknown[]>([]);
const updateData = debounce(async () => {
  const res = await props.fetchFn(pageIndex.value * pageSize.value, pageSize.value, {
    searchText: searchText.value || undefined,
    sortField: sortField.value === null ? undefined : sortField.value,
    desc: sortOrder.value === 'asc'
  });
  data.value = res.data;
  pageCount.value = res.totalPages;
}, 300);

onMounted(() => {
  updateData();
  // Focus the search input if it exists
  if (props.searchable) {
    nextTick(() => {
      filterBoxRef.value?.focus();
    });
  }
});

watch([pageSize, pageIndex, searchText, sortField, sortOrder], updateData);
</script>

<template>
  <Dialog :open="isDeleteModalActive" @update:open="closeDeleteModal">
    <DialogContent>
      <p class="mb-4">Bạn có chắc chắn muốn xoá {{ rowsToDelete.length }} bản ghi?</p>
      <DialogFooter>
        <Button variant="outline" @click="closeDeleteModal">Hủy bỏ</Button>
        <Button variant="destructive" @click="onConfirmDelete">Xác nhận</Button>
      </DialogFooter>
    </DialogContent>
  </Dialog>

  <div class="flex justify-between items-stretch">
    <div v-if="searchable" class="relative items-center flex gap-4 m-auto md:m-0 md:mb-8 mb-8">
      <div class="relative">
        <Search class="absolute left-3 top-[12px] h-4 w-4 text-gray-500" />
        <Input ref="filterBoxRef" v-model="searchText" type="search" placeholder="Nhập tên/mã thiết bị"
          class="pl-10 w-[250px] sm:w-[300px] md:w-[350px] lg:w-[400px]" @input="handlePageIndexChange(0)" />
      </div>

      <Button v-if="qrable" variant="outline" class="relative w-11 lg:w-auto">
        <QrCode class="h-4 w-4 lg:mr-2" />
        <span class="hidden lg:inline">Quét QR</span>
      </Button>

      <Button v-if="addTriggerFn" class="md:hidden" @click="addTriggerFn">
        <Plus class="h-4 w-4" />
      </Button>
    </div>

    <div>
      <slot name="custom-button">
        <Button v-if="addTriggerFn" class="hidden md:flex items-center" @click="addTriggerFn">
          <Plus class="h-4 w-4 mr-2" />
          <span>{{ addTitle ?? 'Thêm' }}</span>
        </Button>
      </slot>
    </div>
  </div>

  <TableCore :columns="createColumns(columns as AugmentedColumnDef<any>[], {
    selectable,
    deletable: !!deleteFn,
    sortField: sortField ?? undefined,
    sortOrder: sortOrder ?? undefined,
    rowSelection,
    onSelectRows,
    onSelectAllRows,
    onDeleteRow,
    onDeleteSelectedRows
  })" :data="data" :page-count="pageCount" :page-size="pageSize" :page-index="pageIndex" :row-selection="rowSelection"
    :selectable="selectable" :sort-field="sortField" :sort-order="sortOrder" @page-index-change="handlePageIndexChange"
    @page-size-change="handlePageSizeChange" @sort-order-change="handleSortOrderChange"
    @sort-field-change="handleSortFieldChange" />
</template>