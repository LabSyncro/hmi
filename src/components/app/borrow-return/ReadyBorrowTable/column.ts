import { type AugmentedColumnDef } from '@/components/common/table';
import { h } from 'vue';

type ReadyBorrowedDeviceSchema = {
    kind: string;
    name: string;
    image: string;
    quantity: number;
    place: string;
}

export const columns: AugmentedColumnDef<ReadyBorrowedDeviceSchema>[] = [
    {
        id: 'kind',
        title: 'Mã',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.kind,
            ),
        enableSorting: true,
    },
    {
        id: 'name',
        title: 'Tên thiết bị',
        cell: ({ row }) =>
            h(
                'div',
                {
                    class: 'justify-center items-start gap-3 inline-flex',
                },
                [
                    h('img', {
                        src: row.original.image,
                        alt: row.original.name,
                        class: 'w-8 h-8 relative object-cover rounded-lg',
                    }),
                    h(
                        'span',
                        { class: 'text-slate-500 text-xs font-normal leading-none' },
                        row.original.name,
                    ),
                ],
            ),
        enableSorting: true,
    },
    {
        id: 'quantity',
        title: 'Số lượng',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.quantity,
            ),
        enableSorting: true,
    },
    {
        id: 'place',
        title: 'Địa điểm',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.place,
            ),
        enableSorting: true,
    },
];