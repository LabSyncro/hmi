<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { ReadyBorrowTable, BorrowTable, ReturnTable } from '@/components/app/borrow-return'
import { Map, Package, SendToBack, RotateCcw } from 'lucide-vue-next'
import type { AcceptableValue } from 'reka-ui'
import { receiptService } from '@/lib/db'

const selectedLab = ref('601 H6, Dĩ An')
const labs = ['601 H6, Dĩ An', '602 H6, Dĩ An', '603 H6, Dĩ An']

const handleLabChange = (value: AcceptableValue) => {
  selectedLab.value = value as string
}

const readyBorrowCount = ref(0)
const borrowingCount = ref(0)
const returnedCount = ref(0)

async function fetchCounts() {
  try {
    const [ready, borrowing, returned] = await Promise.all([
      receiptService.fetchReadyBorrowCount(),
      receiptService.fetchBorrowingCount(),
      receiptService.fetchReturnedCount()
    ])

    readyBorrowCount.value = ready
    borrowingCount.value = borrowing
    returnedCount.value = returned
  } catch (error) {
    throw error
  }
}

onMounted(() => { fetchCounts() })

</script>

<template>
  <div class="py-6 mx-auto max-w-7xl px-4 sm:px-6 lg:px-8">
    <div class="flex flex-col md:flex-row justify-between items-center mb-6 gap-4">
      <h1 class="text-2xl font-bold text-gray-900">Quản lý mượn trả</h1>

      <div class="flex items-center gap-3">
        <div class="flex items-center gap-2">
          <Map class="h-5 w-5 text-gray-500" />
          <Select :model-value="selectedLab" @update:model-value="handleLabChange">
            <SelectTrigger class="w-[180px] bg-white border border-gray-200 shadow-sm">
              <SelectValue :placeholder="selectedLab" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem v-for="lab in labs" :key="lab" :value="lab">
                {{ lab }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>

    <div class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden">
      <Tabs default-value="ready-borrow" class="w-full">
        <div class="border-b border-gray-200 px-4">
          <TabsList class="flex w-full space-x-1 rounded-none bg-transparent">
            <TabsTrigger value="ready-borrow"
              class="rounded-t-lg py-3 px-4 border-b-2 border-transparent data-[state=active]:border-blue-600 data-[state=active]:text-blue-700 focus:outline-none">
              <div class="flex items-center gap-2">
                <Package class="h-5 w-5" />
                <span>Sẵn sàng mượn</span>
                <span class="bg-blue-100 text-blue-700 text-xs font-medium rounded-full px-2 py-0.5">
                  {{ readyBorrowCount }}
                </span>
              </div>
            </TabsTrigger>

            <TabsTrigger value="borrowing"
              class="rounded-t-lg py-3 px-4 border-b-2 border-transparent data-[state=active]:border-green-600 data-[state=active]:text-green-700 focus:outline-none">
              <div class="flex items-center gap-2">
                <SendToBack class="h-5 w-5" />
                <span>Đang mượn</span>
                <span class="bg-green-100 text-green-700 text-xs font-medium rounded-full px-2 py-0.5">
                  {{ borrowingCount }}
                </span>
              </div>
            </TabsTrigger>

            <TabsTrigger value="returned"
              class="rounded-t-lg py-3 px-4 border-b-2 border-transparent data-[state=active]:border-purple-600 data-[state=active]:text-purple-700 focus:outline-none">
              <div class="flex items-center gap-2">
                <RotateCcw class="h-5 w-5" />
                <span>Đã trả</span>
                <span class="bg-purple-100 text-purple-700 text-xs font-medium rounded-full px-2 py-0.5">
                  {{ returnedCount }}
                </span>
              </div>
            </TabsTrigger>
          </TabsList>
        </div>

        <div class="p-4">
          <TabsContent value="ready-borrow" class="mt-0">
            <ReadyBorrowTable />
          </TabsContent>

          <TabsContent value="borrowing" class="mt-0">
            <BorrowTable />
          </TabsContent>

          <TabsContent value="returned" class="mt-0">
            <ReturnTable />
          </TabsContent>
        </div>
      </Tabs>
    </div>
  </div>
</template>