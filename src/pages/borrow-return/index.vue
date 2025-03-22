<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { ReadyBorrowTable, BorrowTable, ReturnTable } from '@/components/app/borrow-return'
import { Upload, Download } from 'lucide-vue-next'
import type { AcceptableValue } from 'reka-ui'
import { receiptService } from '@/lib/db/receipt'

const selectedLab = ref('601 H6, Dĩ An')
const labs = [
  '601 H6, Dĩ An',
  '602 H6, Dĩ An',
  '603 H6, Dĩ An',
  '604 H6, Dĩ An',
]

const handleLabChange = (value: AcceptableValue) => {
  selectedLab.value = value as string
}

const readyBorrowCount = ref(0)
const borrowingCount = ref(0)
const returnedCount = ref(0)

const fetchData = async () => {
  const readyBorrowRes = await receiptService.fetchReadyBorrowDevices(0, 10, {
    sortField: undefined,
    desc: true,
  })
  const borrowingRes = await receiptService.fetchBorrowingDevices(0, 10, {
    sortField: undefined,
    desc: true,
  })
  const returnedRes = await receiptService.fetchReturnedDevices(0, 10, {
    sortField: undefined,
    desc: true,
  })
  readyBorrowCount.value = readyBorrowRes.totalCount
  borrowingCount.value = borrowingRes.totalCount
  returnedCount.value = returnedRes.totalCount
}

onMounted(() => {
  fetchData()
})
</script>

<template>
  <div class="mt-4 mx-auto">
    <div class="flex justify-between items-center mb-4">
      <div class="flex gap-2">
        <Button variant="default" class="bg-tertiary-darker hover:bg-blue-900 w-24">
          <Upload class="h-4 w-4" />
          Mượn
        </Button>
        <Button variant="default" class="bg-tertiary-darker hover:bg-blue-900 w-24">
          <Download class="h-4 w-4" />
          Trả
        </Button>
      </div>
    </div>

    <Tabs default-value="ready-borrow" class="w-full">
      <div class="flex justify-between items-center">
        <TabsList class="grid w-[600px] grid-cols-3">
          <TabsTrigger value="ready-borrow">Sẵn sàng mượn ({{ readyBorrowCount }})</TabsTrigger>
          <TabsTrigger value="borrowing">Đang mượn ({{ borrowingCount }})</TabsTrigger>
          <TabsTrigger value="returned">Đã trả ({{ returnedCount }})</TabsTrigger>
        </TabsList>
        <Select :model-value="selectedLab" @update:model-value="handleLabChange">
          <SelectTrigger class="w-[180px] bg-white">
            <SelectValue :placeholder="selectedLab" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="lab in labs" :key="lab" :value="lab">
              {{ lab }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <TabsContent value="ready-borrow">
        <ReadyBorrowTable />
      </TabsContent>
      <TabsContent value="borrowing">
        <BorrowTable />
      </TabsContent>
      <TabsContent value="returned">
        <ReturnTable />
      </TabsContent>
    </Tabs>
  </div>
</template>