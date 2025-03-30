<script setup lang="ts">
import type { Column } from '@tanstack/vue-table';
import { cn } from '@/lib/utils';
import { DropdownMenu, DropdownMenuTrigger } from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';
import { ArrowUp, ArrowDown, ArrowUpDown } from 'lucide-vue-next';

type TableColumnHeaderProps = {
    id: string;
    column: Column<any, unknown>;
    title: string;
    sortOrder: 'desc' | 'asc' | undefined;
    sortField: string | undefined;
}

const props = defineProps<TableColumnHeaderProps>();
const emits = defineEmits<{
    'sort-field-change': [string | undefined];
    'sort-order-change': ['desc' | 'asc' | undefined];
}>();

defineOptions({
    inheritAttrs: false,
});

function toggleSortOrder() {
    if (props.sortOrder === 'asc') {
        emits('sort-field-change', props.id);
        emits('sort-order-change', 'desc');
    } else if (props.sortOrder === 'desc') {
        emits('sort-field-change', undefined);
        emits('sort-order-change', undefined);
    } else {
        emits('sort-field-change', props.id);
        emits('sort-order-change', 'asc');
    }
}

</script>

<template>
    <div :class="cn('flex items-center space-x-2', $attrs.class ?? '')">
        <DropdownMenu>
            <DropdownMenuTrigger as-child>
                <Button variant="ghost" size="sm" class="h-8 data-[state=open]:bg-accent text-md pr-8 text-black"
                    @click="toggleSortOrder()">
                    <span>{{ title }}</span>
                    <span v-if="column.columnDef.enableSorting" class="flex items-center ml-2">
                        <ArrowUp v-if="sortField === id && sortOrder === 'asc'" aria-hidden class="text-xl right-1" />
                        <ArrowDown v-else-if="sortField === id && sortOrder === 'desc'" aria-hidden
                            class="text-xl right-1" />
                        <ArrowUpDown v-else-if="sortField !== id" aria-hidden class="text-xl right-1" />
                    </span>
                </Button>
            </DropdownMenuTrigger>
        </DropdownMenu>
    </div>
</template>