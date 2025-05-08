<script setup lang="ts">
import { Map, Package, RotateCcw, SendToBack } from "lucide-vue-next";
import type { AcceptableValue } from "reka-ui";

const selectedLab = ref("601 H6, Dĩ An");
const labs = ["601 H6, Dĩ An", "602 H6, Dĩ An", "603 H6, Dĩ An"];

const handleLabChange = (value: AcceptableValue) => {
  selectedLab.value = value as string;
};

const readyBorrowCount = ref(0);
const borrowingCount = ref(0);
const returnedCount = ref(0);

async function fetchCounts() {
  try {
    const [ready, borrowing, returned] = await Promise.all([
      receiptService.fetchReadyBorrowCount(),
      receiptService.fetchBorrowingCount(),
      receiptService.fetchReturnedCount(),
    ]);

    readyBorrowCount.value = ready;
    borrowingCount.value = borrowing;
    returnedCount.value = returned;
  } catch (error) {
    throw error;
  }
}

onMounted(() => {
  fetchCounts();
});
</script>

<template>
  <div class="py-2 mx-auto px-2">
    <div
      class="flex flex-col md:flex-row justify-between items-center mb-2 gap-2"
    >
      <h1 class="text-lg font-bold text-gray-900">Quản lý mượn trả</h1>

      <div class="flex items-center gap-2">
        <div class="flex items-center gap-1">
          <Map class="h-4 w-4 text-gray-500" />
          <Select
            :model-value="selectedLab"
            @update:model-value="handleLabChange"
          >
            <SelectTrigger
              class="w-[160px] bg-white border border-gray-200 shadow-sm text-xs py-1"
            >
              <SelectValue :placeholder="selectedLab" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem
                v-for="lab in labs"
                :key="lab"
                :value="lab"
                class="text-xs"
              >
                {{ lab }}
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>

    <div
      class="bg-white rounded-lg shadow-sm border border-gray-200 overflow-hidden"
    >
      <Tabs default-value="ready-borrow" class="w-full">
        <div class="border-b border-gray-200 px-2">
          <TabsList class="flex w-full space-x-0.5 rounded-none bg-transparent">
            <TabsTrigger
              value="ready-borrow"
              class="rounded-t-lg py-2 px-2 border-b-2 border-transparent data-[state=active]:border-blue-600 data-[state=active]:text-blue-600 focus:outline-none"
            >
              <div class="flex items-center gap-1">
                <Package class="h-3 w-3" />
                <span class="text-xs whitespace-nowrap">Sẵn sàng mượn</span>
                <span
                  class="bg-blue-100 text-blue-700 text-xs font-medium rounded-full px-1.5 py-0.5"
                >
                  {{ readyBorrowCount }}
                </span>
              </div>
            </TabsTrigger>

            <TabsTrigger
              value="borrowing"
              class="rounded-t-lg py-2 px-2 border-b-2 border-transparent data-[state=active]:border-green-600 data-[state=active]:text-green-600 focus:outline-none"
            >
              <div class="flex items-center gap-1">
                <SendToBack class="h-3 w-3" />
                <span class="text-xs whitespace-nowrap">Đang mượn</span>
                <span
                  class="bg-green-100 text-green-700 text-xs font-medium rounded-full px-1.5 py-0.5"
                >
                  {{ borrowingCount }}
                </span>
              </div>
            </TabsTrigger>

            <TabsTrigger
              value="returned"
              class="rounded-t-lg py-2 px-2 border-b-2 border-transparent data-[state=active]:border-purple-600 data-[state=active]:text-purple-600 focus:outline-none"
            >
              <div class="flex items-center gap-1">
                <RotateCcw class="h-3 w-3" />
                <span class="text-xs whitespace-nowrap">Đã trả</span>
                <span
                  class="bg-purple-100 text-purple-700 text-xs font-medium rounded-full px-1.5 py-0.5"
                >
                  {{ returnedCount }}
                </span>
              </div>
            </TabsTrigger>
          </TabsList>
        </div>

        <div class="p-2">
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
