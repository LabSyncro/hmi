import { ref, readonly } from 'vue'
import type { BorrowReturnDeviceSchema, ReadyBorrowedDeviceSchema, ReturnedReceiptDeviceSchema } from '@/components/app/borrow-return'

export type CachedData<T> = {
    data: T[];
    totalPages: number;
    totalCount: number;
    timestamp: number;
}

export type ReceiptCache = {
    readyBorrow: CachedData<ReadyBorrowedDeviceSchema> | null;
    borrowing: CachedData<BorrowReturnDeviceSchema> | null;
    returned: CachedData<ReturnedReceiptDeviceSchema> | null;
}

const CACHE_DURATION = 5 * 60 * 1000;

const cache = ref<ReceiptCache>({
    readyBorrow: null,
    borrowing: null,
    returned: null,
})

export function useReceiptCache() {
    const isCacheValid = (timestamp: number | undefined) => {
        if (!timestamp) return false;
        return Date.now() - timestamp < CACHE_DURATION;
    }

    const updateReadyBorrowCache = (data: Omit<CachedData<ReadyBorrowedDeviceSchema>, 'timestamp'>) => {
        cache.value.readyBorrow = {
            ...data,
            timestamp: Date.now()
        }
    }

    const updateBorrowingCache = (data: Omit<CachedData<BorrowReturnDeviceSchema>, 'timestamp'>) => {
        cache.value.borrowing = {
            ...data,
            timestamp: Date.now()
        }
    }

    const updateReturnedCache = (data: Omit<CachedData<ReturnedReceiptDeviceSchema>, 'timestamp'>) => {
        cache.value.returned = {
            ...data,
            timestamp: Date.now()
        }
    }

    const clearCache = () => {
        cache.value = {
            readyBorrow: null,
            borrowing: null,
            returned: null,
        }
    }

    const isReadyBorrowCacheValid = () => isCacheValid(cache.value.readyBorrow?.timestamp)
    const isBorrowingCacheValid = () => isCacheValid(cache.value.borrowing?.timestamp)
    const isReturnedCacheValid = () => isCacheValid(cache.value.returned?.timestamp)

    return {
        cache: readonly(cache),
        updateReadyBorrowCache,
        updateBorrowingCache,
        updateReturnedCache,
        clearCache,
        isReadyBorrowCacheValid,
        isBorrowingCacheValid,
        isReturnedCacheValid,
    }
} 