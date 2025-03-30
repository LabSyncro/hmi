<script setup lang="ts">
import { createColumns, type AugmentedColumnDef } from './column'
import { ref, onMounted, watch } from 'vue'
import TableCore from './Core.vue'

const props = defineProps<{
  fetchFn: (offset: number, length: number, options: { desc?: boolean, sortField?: string, searchText?: string }) => Promise<{ data: unknown[], totalPages: number }>,
  columns: AugmentedColumnDef<any>[],
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

const data = ref<any[]>([])
const error = ref<Error | null>(null)
const isLoading = ref(false)

const updateData = async () => {
  try {
    isLoading.value = true
    error.value = null
    const res = await props.fetchFn(pageIndex.value * pageSize.value, pageSize.value, {
      sortField: sortField.value === null ? undefined : sortField.value,
      desc: sortOrder.value === 'asc'
    })
    data.value = res.data
    pageCount.value = res.totalPages
  } catch (err) {
    error.value = err as Error
    data.value = []
    pageCount.value = 0
  } finally {
    isLoading.value = false
  }
}

onMounted(() => { updateData() })

watch([pageSize, pageIndex, sortField, sortOrder], updateData)

</script>

<template>
  <div class="bg-white rounded-lg shadow-md p-4">
    <div v-if="error" class="p-4 text-red-500 bg-red-50 rounded mb-4 flex items-center justify-between">
      <p class="mt-1">Đã xảy ra lỗi khi tải dữ liệu. Vui lòng thử lại sau.</p>
      <button @click="updateData" class="px-3 py-1 text-sm bg-red-100 hover:bg-red-200 rounded">
        Thử lại
      </button>
    </div>

    <div v-if="isLoading" class="p-4 text-gray-500 text-center">
      <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-gray-500 mx-auto"></div>
      <p class="mt-2">Đang tải dữ liệu...</p>
    </div>

    <TableCore v-else :columns="createColumns(columns as AugmentedColumnDef<any>[], {
      sortField: sortField ?? undefined,
      sortOrder: sortOrder ?? undefined,
    })" :data="data" :page-count="pageCount" :page-size="pageSize" :page-index="pageIndex" :sort-field="sortField"
      :sort-order="sortOrder" @page-index-change="handlePageIndexChange" @page-size-change="handlePageSizeChange"
      @sort-order-change="handleSortOrderChange" @sort-field-change="handleSortFieldChange" />
  </div>
</template>