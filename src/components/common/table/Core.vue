<script setup lang="ts">
import { FlexRender, getCoreRowModel, useVueTable, type ColumnDef } from '@tanstack/vue-table';
import { Table, TableHeader, TableRow, TableHead, TableBody, TableCell } from '@/components/ui/table';
import TablePagination from './Pagination.vue';

type DataTableProps = {
    columns: ColumnDef<any, unknown>[];
    data: any[];
    pageCount: number;
    pageIndex: number;
    pageSize: number;
    sortField: string | null;
    sortOrder: 'desc' | 'asc' | null;
}

const props = defineProps<DataTableProps>();
const emits = defineEmits<{
    'page-size-change': [number];
    'page-index-change': [number];
    'sort-field-change': [string | null];
    'sort-order-change': ['desc' | 'asc' | null];
}>();

const table = useVueTable({
    get data() { return props.data; },
    get columns() { return props.columns; },
    manualPagination: true,
    pageCount: props.pageCount,
    manualSorting: true,
    enableRowSelection: true,
    getCoreRowModel: getCoreRowModel(),
});
</script>

<template>
    <div class="space-y-4">
        <div>
            <Table>
                <TableHeader>
                    <TableRow v-for="headerGroup in table.getHeaderGroups()" :key="headerGroup.id">
                        <TableHead v-for="header in headerGroup.headers" :key="header.id">
                            <FlexRender v-if="!header.isPlaceholder" :render="header.column.columnDef.header"
                                :props="header.getContext()"
                                @sort-order-change="(value) => emits('sort-order-change', value)"
                                @sort-field-change="(value) => emits('sort-field-change', value)" />
                        </TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    <template v-if="table.getRowModel().rows?.length">
                        <TableRow v-for="row in table.getRowModel().rows" :key="row.id"
                            :data-state="row.getIsSelected() && 'selected'">
                            <TableCell v-for="cell in row.getVisibleCells()" :key="cell.id">
                                <FlexRender :render="cell.column.columnDef.cell" :props="cell.getContext()" />
                            </TableCell>
                        </TableRow>
                    </template>

                    <TableRow v-else>
                        <TableCell :colspan="columns.length" class="h-24 text-center text-lg text-slate-500">
                            Không có kết quả
                        </TableCell>
                    </TableRow>
                </TableBody>
            </Table>
        </div>

        <div class="text-sm text-slate flex items-center justify-between">
            <TablePagination :table="table" :page-index="pageIndex" :page-size="pageSize" :page-count="pageCount"
                @page-size-change="(value: number) => emits('page-size-change', value)"
                @page-index-change="(value: number) => emits('page-index-change', value)" />
        </div>
    </div>
</template>