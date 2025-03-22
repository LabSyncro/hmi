import { type AugmentedColumnDef } from '@/components/common/table';
import { h } from 'vue';

const statusMap = {
    late: 'Trễ hạn',
    on_time: 'Đúng hạn',
};

const borrowStateMap = {
    borrowing: 'Đang mượn',
    returned: 'Trả xong'
};

type BorrowReturnDeviceSchema = {
    receiptCode: string,
    borrowerName: string,
    borrowerImage: string,
    totalQty: number,
    returnedQty: number,
    borrowedPlace: string,
    borrowedAt: Date,
    expectedReturnedAt: Date,
    status: 'on_time' | 'late',
    borrowState: 'borrowing' | 'returned'
}

export const columns: AugmentedColumnDef<BorrowReturnDeviceSchema>[] = [
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
        id: 'borrowerName',
        title: 'Người mượn',
        cell: ({ row }) =>
            h(
                'div',
                {
                    class: 'justify-center items-start gap-3 inline-flex',
                },
                [
                    h('img', {
                        src: row.original.borrowerImage,
                        alt: row.original.borrowerName,
                        class: 'w-8 h-8 relative object-cover rounded-lg',
                    }),
                    h(
                        'span',
                        { class: 'text-slate-500 text-xs font-normal leading-none' },
                        row.original.borrowerName,
                    ),
                ],
            ),
        enableSorting: true,
    },
    {
        id: 'totalQty',
        title: 'Đã trả',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                `${row.original.returnedQty}/${row.original.totalQty}`,
            ),
        enableSorting: true,
    },
    {
        id: 'borrowedPlace',
        title: 'Nơi mượn',
        cell: ({ row }) =>
            h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                row.original.borrowedPlace,
            ),
        enableSorting: true,
    },
    {
        id: 'borrowedAt',
        title: 'Ngày mượn',
        cell: ({ row }) => {
            const value = row.original.borrowedAt.toString();
            return h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                value,
            );
        },
        enableSorting: true,
    },
    {
        id: 'expectedReturnedAt',
        title: 'Ngày hẹn trả',
        cell: ({ row }) => {
            const value = row.original.expectedReturnedAt.toString();
            return h(
                'span',
                { class: 'text-slate-500 text-sm font-normal leading-tight' },
                value,
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
        id: 'borrowState',
        title: 'Trạng thái',
        cell: ({ row }) =>
            h(
                'span',
                {
                    class: `inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium ${borrowStateMap[row.original.borrowState] === 'Đang mượn'
                        ? 'bg-blue-100 text-blue-800'
                        : 'bg-green-100 text-green-800'
                        }`,
                },
                borrowStateMap[row.original.borrowState],
            ),
        enableSorting: true,
    },
];