import type { ColumnDef } from '@tanstack/vue-table';
import ColumnHeader from './ColumnHeader.vue';
import { h } from 'vue';

export type AugmentedColumnDef<T> = Omit<ColumnDef<T>, 'header'> & { id: string, title: string };

export function createColumns<T extends { id: string }>(
  dataColumns: AugmentedColumnDef<T>[], {
    sortField,
    sortOrder,
  }: {
    sortField: string | undefined;
    sortOrder: 'desc' | 'asc' | undefined;
  }): ColumnDef<T>[] {

  return [
    ...dataColumns.map((colWithoutHeader: AugmentedColumnDef<T>) => {
      const colWithHeader: ColumnDef<T> = {
        ...colWithoutHeader,
        header: ({ column }) => h(ColumnHeader, { id: colWithoutHeader.id!, sortField, sortOrder, column, title: colWithoutHeader.title }),
      };
      return colWithHeader;
    }),
  ];
}