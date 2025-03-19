<script setup lang="ts">
import { ref } from 'vue'
import { Tabs, TabsList, TabsTrigger, TabsContent } from '@/components/ui/tabs'
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { Button } from '@/components/ui/button'
import { ReadyBorrowTable, BorrowTable, ReturnTable } from '@/components/app/borrow-return'
import type { AcceptableValue } from 'reka-ui'

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
</script>

<template>
  <div class="container mx-auto py-6">
    <div class="flex justify-between items-center mb-6">
      <div class="flex items-center gap-4">
        <Select :model-value="selectedLab" @update:model-value="handleLabChange">
          <SelectTrigger class="w-[180px]">
            <SelectValue :placeholder="selectedLab" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem v-for="lab in labs" :key="lab" :value="lab">
              {{ lab }}
            </SelectItem>
          </SelectContent>
        </Select>
      </div>
      <div class="flex gap-2">
        <Button variant="default" class="bg-blue-500 hover:bg-blue-600">
          <Icon name="i-heroicons-arrow-up-tray" class="mr-2 h-4 w-4" />
          Mượn
        </Button>
        <Button variant="default" class="bg-green-500 hover:bg-green-600">
          <Icon name="i-heroicons-arrow-down-tray" class="mr-2 h-4 w-4" />
          Trả
        </Button>
      </div>
    </div>

    <Tabs default-value="ready-borrow" class="w-full">
      <TabsList class="grid w-full grid-cols-3">
        <TabsTrigger value="ready-borrow">Sẵn sàng mượn</TabsTrigger>
        <TabsTrigger value="borrowing">Đang mượn</TabsTrigger>
        <TabsTrigger value="returned">Đã trả</TabsTrigger>
      </TabsList>
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