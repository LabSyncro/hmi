import type { ColumnDef } from '@tanstack/vue-table';
import { Checkbox } from '@/components/ui/checkbox';
import ColumnHeader from './ColumnHeader.vue';
import { Trash } from 'lucide-vue-next';
import { h } from 'vue';

export type AugmentedColumnDef<T> = Omit<ColumnDef<T>, 'header'> & { id: string, title: string };

export function createColumns<T extends { id: string }>(
  dataColumns: AugmentedColumnDef<T>[], {
    selectable,
    deletable,
    sortField,
    sortOrder,
    rowSelection,
    onSelectRows,
    onSelectAllRows,
    onDeleteRow,
    onDeleteSelectedRows
  }: {
    selectable: boolean,
    deletable: boolean,
    sortField: string | undefined;
    sortOrder: 'desc' | 'asc' | undefined;
    rowSelection: string[];
    onSelectRows: (_: string[]) => void;
    onSelectAllRows: (_: string[]) => void;
    onDeleteRow: (_: string) => void;
    onDeleteSelectedRows: (_: string[]) => void
  }): ColumnDef<T>[] {
  const selectCol: ColumnDef<T> = {
    accessorKey: 'select',
    id: 'select',
    header: ({ table }) =>
      h('div', { class: 'flex items-center' },
        [h(Checkbox, {
          checked: table.getCoreRowModel().rows.every((row) => rowSelection.includes(row.original.id)),
          'onUpdate:checked': () => onSelectAllRows(table.getCoreRowModel().rows.map((row) => row.original.id)),
          ariaLabel: 'Select all',
          class: 'translate-y-0.5',
        })],
      ),
    cell: ({ row }) =>
      h('div', { class: 'flex items-center' },
        [h(Checkbox, {
          checked: rowSelection.includes(row.original.id),
          'onUpdate:checked': () => onSelectRows([row.original.id]),
          ariaLabel: 'Select row',
          class: 'translate-y-0.5',
        })],
      ),
    enableSorting: false,
    enableHiding: false,
  };
  const deleteCol: ColumnDef<T> = {
    accessorKey: 'delete',
    id: 'delete',
    header: ({ table }) =>
      h('div', { class: 'flex items-center hover:cursor-pointer', onClick: () => onDeleteSelectedRows(table.getCoreRowModel().rows.map((row) => row.original.id)) },
        [
          h(Trash, { class: `${rowSelection.length === 0 ? 'hidden' : 'block'} ml-4 text-danger-darker text-lg font-bold` }),
        ],
      ),
    cell: ({ row }) =>
      h('div', { class: 'flex items-center hover:cursor-pointer', onClick: () => onDeleteRow(row.original.id) },
        [
          h(Trash, { class: 'ml-4 text-danger-darker text-lg font-bold' }),
        ],
      ),
    enableSorting: false,
    enableHiding: false,
  };

  return [
    ...(selectable ? [selectCol] : []),
    ...dataColumns.map((colWithoutHeader: AugmentedColumnDef<T>) => {
      const colWithHeader: ColumnDef<T> = {
        ...colWithoutHeader,
        header: ({ column }) => h(ColumnHeader, { id: colWithoutHeader.id!, sortField, sortOrder, column, title: colWithoutHeader.title }),
      };
      return colWithHeader;
    }),
    ...(deletable ? [deleteCol] : []),
  ];
}