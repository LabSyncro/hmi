<script setup lang="ts">
import { debounce } from 'lodash-es'
import { createColumns, type AugmentedColumnDef } from './column'
import { ref, onMounted, watch } from 'vue'
import TableCore from './Core.vue'

const props = defineProps<{
  fetchFn: (offset: number, length: number, options: { desc?: boolean, sortField?: string, searchText?: string }) => Promise<{ data: unknown[], totalPages: number }>,
  columns: AugmentedColumnDef<unknown>[],
}>()

const pageIndex = ref(0)
const pageSize = ref(10)
function handlePageIndexChange(value: number) {
  pageIndex.value = value
}
function handlePageSizeChange(value: number) {
  pageSize.value = value
}
const pageCount = ref(0)

const sortField = ref<string | null>(null)
const sortOrder = ref<'desc' | 'asc' | null>(null)

function handleSortFieldChange(value: string | null) {
  sortField.value = value
}
function handleSortOrderChange(value: 'desc' | 'asc' | null) {
  sortOrder.value = value
}

const data = ref<unknown[]>([])
const updateData = debounce(async () => {
  const res = await props.fetchFn(pageIndex.value * pageSize.value, pageSize.value, {
    sortField: sortField.value === null ? undefined : sortField.value,
    desc: sortOrder.value === 'asc'
  })
  data.value = res.data
  pageCount.value = res.totalPages
}, 300)

onMounted(() => { updateData() })

watch([pageSize, pageIndex, sortField, sortOrder], updateData)
</script>

<template>
  <TableCore :columns="createColumns(columns as AugmentedColumnDef<any>[], {
    sortField: sortField ?? undefined,
    sortOrder: sortOrder ?? undefined,
  })" :data="data" :page-count="pageCount" :page-size="pageSize" :page-index="pageIndex" :sort-field="sortField"
    :sort-order="sortOrder" @page-index-change="handlePageIndexChange" @page-size-change="handlePageSizeChange"
    @sort-order-change="handleSortOrderChange" @sort-field-change="handleSortFieldChange" />
</template>