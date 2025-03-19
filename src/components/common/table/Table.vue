<script setup lang="ts">
import { debounce } from 'lodash-es';
import { createColumns, type AugmentedColumnDef } from './column';
import { ref, onMounted, watch, useTemplateRef } from 'vue';

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
  const res = await props.fetchFn(pageIndex.value * pageSize.value, pageSize.value, { searchText: searchText.value || undefined, sortField: sortField.value || undefined as any, desc: sortOrder.value === 'asc' });
  data.value = res.data;
  pageCount.value = res.totalPages;
}, 300);
onMounted(updateData);
watch([pageSize, pageIndex, searchText, sortField, sortOrder], updateData);

const filterBoxRef = useTemplateRef<HTMLInputElement>('filterBox');
onMounted(() => filterBoxRef.value?.focus());
</script>

<template>
  <div>
    <div v -if="isDeleteModalActive"
      class="fixed top-4 z-50 left-0 w-[100vw] h-[100vh] p-10 flex justify-end items-end">
      <div class="bg-white p-4 shadow-[0_0px_16px_-3px_rgba(0,0,0,0.3)]">
        <p class="mb-4"> Bạn có chắc chắn muốn xoá { { rowsToDelete.length } } bản ghi ? </p>
        < div class="flex gap-3 justify-end">
          <button class="bg-gray-300 p-1.5 px-3 rounded-md text-normal" @click="closeDeleteModal"> Hủy bỏ </button>
          <button class="bg-red-500 text-white p-1.5 px-3 rounded-md text-normal" @click="onConfirmDelete">
            Xác nhận
          </button>
      </div>
    </div>
  </div>
  <div class="flex justify-between items-stretch">
    <div v-if="searchable" class="relative items-center flex gap-4 m-auto md:m-0 md:mb-8 mb-8">
      <input ref="filterBox" v - model="searchText" type="search" placeholder="Nhập tên/mã thiết bị"
        class="border-gray-300 border rounded-sm p-2 pl-10 w-[250px] sm:w-[300px] md:w-[350px] lg:w-[400px]"
        @input="handlePageIndexChange(0)">
      <Icon aria - hidden class="absolute left-3 top-[12px] text-xl text-primary-dark"
        name="i-heroicons-magnifying-glass" />
      <button v -if="qrable"
        class="relative bg-slate-200 border border-slate-400 text-slate-dark rounded-md w-11 h-11 lg:w-auto">
        <Icon aria - hidden class="absolute left-3 top-[12px] text-xl" name="i-heroicons-qr-code" />
        <p class="hidden lg:block pl-10 pr-3"> Quét QR </p>
      </button>
      <button v -if="addTriggerFn"
        class="relative md:hidden bg-tertiary-darker items-center text-white px-3 rounded-md w-11 h-11">
        <Icon aria - hidden class="absolute left-3 top-[12px] text-xl" name="i-heroicons-plus" />
      </button>
    </div>
    <div>
      <slot name="custom-button">
        <button v -if="addTriggerFn"
          class="relative hidden md:block bg-tertiary-darker items-center text-white px-3 rounded-md w-11 h-11 md:w-auto"
          @click="addTriggerFn">
          <Icon aria - hidden class="absolute left-3 top-[12px] text-xl" name="i-heroicons-plus" />
          <span class="hidden md:block pl-8 pr-3"> {{ addTitle ?? 'Thêm' }}</span>
        </button>
      </slot>
    </div>
  </div>
  <DataTableCore
    :columns="createColumns(columns as AugmentedColumnDef<any>[], { selectable, deletable: !!deleteFn, sortField: sortField as any, sortOrder: sortOrder as any, rowSelection, onSelectRows, onSelectAllRows, onDeleteRow, onDeleteSelectedRows })"
    :data="data" :page-count="pageCount" :page-size="pageSize" :page-index="pageIndex" :row-selection="rowSelection"
    :selectable="selectable" @page-index-change="handlePageIndexChange" @page-size-change="handlePageSizeChange"
    @sort-order-change="handleSortOrderChange as any" @sort-field-change="handleSortFieldChange as any" />
</template>