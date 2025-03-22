import type { AugmentedColumnDef } from '@/components/common/table/column';
import { h } from 'vue';

type ReturnedReceiptDeviceSchema = {
    receiptCode: string,
    returnedName: string,
    returnedImage: string,
    quantity: number,
    returnedPlace: string,
    returnedAt: Date,
    status: 'on_time' | 'late',
    note: string,
}

const statusMap = {
    late: 'Trễ hạn',
    on_time: 'Đúng hạn',
};

export const columns: AugmentedColumnDef<ReturnedReceiptDeviceSchema>[] = [
    {
        id: 'receiptCode',
        title: 'Mã đơn',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.receiptCode,
            ),
        enableSorting: true,
    },
    {
        id: 'returnedName',
        title: 'Người trả',
        cell: ({ row }) =>
            h(
                'div',
                {
                    class: 'justify-center items-start gap-3 inline-flex',
                },
                [
                    h('img', {
                        src: row.original.returnedImage,
                        alt: row.original.returnedName,
                        class: 'w-8 h-8 relative object-cover rounded-lg',
                    }),
                    h(
                        'span',
                        { class: 'text-slate-500 text-xs font-normal leading-none' },
                        row.original.returnedName,
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
        id: 'returnedPlace',
        title: 'Nơi trả',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.returnedPlace,
            ),
        enableSorting: true,
    },
    {
        id: 'returnedAt',
        title: 'Ngày thực trả',
        cell: ({ row }) => {
            const value = row.original.returnedAt?.toString();
            return h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                value ? value : 'Chưa trả',
            );
        },
        enableSorting: true,
    },
    {
        id: 'status',
        title: 'Tiến độ',
        cell: ({ row }) =>
            h(
                'span',
                {
                    class: `inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${statusMap[row.original.status] === 'Đúng hạn'
                        ? 'bg-green-100 text-green-800'
                        : 'bg-red-100 text-red-800'
                        }`,
                },
                statusMap[row.original.status],
            ),
        enableSorting: true,
    },
    {
        id: 'note',
        title: 'Ghi chú',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.note,
            ),
        enableSorting: true,
    },
];