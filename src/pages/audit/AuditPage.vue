<script setup lang="ts">
import {
  BoxIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  ClipboardCheckIcon,
  MapPinIcon,
  PackageIcon,
  TrashIcon,
  UserIcon,
} from "lucide-vue-next";

const mode = ref<"idle" | "audit">("idle");

const userInfo = ref<UserInfo | null>(null);
const storedUserInfo = ref<{
  id: string;
  lab: { id: string; room: string; branch: string };
} | null>(null);

const devices = ref<AuditDevice[]>([]);

const notes = ref<string>("");

const auditDetails = ref({
  location: "",
  auditDate: new Date().toLocaleDateString("vi-VN"),
  totalDevices: 0,
  deviceConditions: {
    good: 0,
    damaged: 0,
    missing: 0,
    discarded: 0,
  },
});

const router = useRouter();

const isConfirming = ref(false);
const showSuccessModal = ref(false);
const successMessage = ref("");
const auditId = ref("");

const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0);
});

const { verifyScannedQrCode } = useOneTimeQR();

function generateUniqueId(): string {
  const now = new Date();
  const datePrefix = now.toISOString().split("T")[0].replace(/-/g, "");
  const randomSuffix = Math.floor(Math.random() * 1000000)
    .toString()
    .padStart(6, "0");
  return `${datePrefix}/${randomSuffix}`;
}

onMounted(async () => {
  const stored = localStorage.getItem("user_info");
  if (stored) {
    const ui = JSON.parse(stored) as {
      id: string;
      lab: { id: string; room: string; branch: string };
    };
    const loc = ui.lab ? `${ui.lab.room}, ${ui.lab.branch}` : "";
    auditDetails.value.location = loc;
    storedUserInfo.value = ui;
  }
});

async function handleUserCodeChange(userId: string) {
  const isValidUserCode = /^\d{7}$/.test(userId);

  if (!isValidUserCode) {
    toast({
      title: "Lỗi",
      description: "Mã người dùng không hợp lệ",
      variant: "destructive",
    });
    userInfo.value = null;
    return;
  }

  isConfirming.value = true;
  try {
    const userMeta = await userService.getUserById(userId);
    if (!userMeta) {
      toast({
        title: "Lỗi",
        description: "Không tìm thấy người dùng",
        variant: "destructive",
      });
      userInfo.value = null;
      return;
    }

    userInfo.value = {
      id: userMeta.id,
      name: userMeta.name,
      avatar: userMeta.avatar,
      roles: userMeta.roles,
    };

    toast({
      title: "Thành công",
      description: `Đã nhận diện: ${userMeta.name}`,
      variant: "success",
    });
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể tìm thấy thông tin người dùng",
      variant: "destructive",
    });
    userInfo.value = null;
  } finally {
    isConfirming.value = false;
  }
}

const handleDeviceScan = async (input: string) => {
  try {
    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1];
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1];

    if (!deviceId) {
      toast({
        title: "Lỗi",
        description: "Không thể trích xuất ID thiết bị từ mã QR",
        variant: "destructive",
      });
      return;
    }

    const isAlreadyAdded = devices.value.some((device) =>
      device.items.some((item) => item.id === deviceId)
    );

    if (isAlreadyAdded) {
      toast({
        title: "Lỗi",
        description: "Thiết bị này đã được thêm vào danh sách.",
        variant: "destructive",
      });
      return;
    }

    try {
      const deviceDetails = await deviceService.getDeviceAuditById(
        deviceId,
        storedUserInfo.value?.lab.id || ""
      )!;

      const inventoryData = await deviceService.getDeviceInventoryByKindId(
        deviceKindId!,
        storedUserInfo.value?.lab.id || ""
      );
      const labInventory = inventoryData.find(
        (inv) =>
          inv.room === deviceDetails.labRoom &&
          inv.branch === deviceDetails.labBranch
      );
      const expectedQuantity = labInventory?.availableQuantity || 0;

      if (mode.value === "idle") {
        mode.value = "audit";
      }

      const newItem: AuditDeviceItem = {
        id: deviceId,
        status: deviceDetails.status,
        auditCondition: DeviceStatus.HEALTHY,
      };

      const existingDevice = devices.value.find((d) => d.code === deviceKindId);
      if (existingDevice) {
        existingDevice.items.push(newItem);
        existingDevice.quantity = existingDevice.items.length;
      } else {
        devices.value.push({
          code: deviceKindId!,
          name: deviceDetails.deviceName,
          image: deviceDetails.image,
          quantity: 1,
          unit: deviceDetails.unit,
          expanded: true,
          items: [newItem],
          isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly || false,
          expectedQuantity,
          unscannedCondition: DeviceStatus.LOST,
        });
      }

      updateAuditCounts();

      toast({
        title: "Thành công",
        description: "Đã thêm thiết bị vào danh sách kiểm đếm",
        variant: "success",
      });
    } catch (error) {
      if (error instanceof Error) {
        if (error.message === "Device not found") {
          toast({
            title: "Lỗi",
            description: "Không tìm thấy thiết bị",
            variant: "destructive",
          });
        } else if (error.message === "Device does not belong to this lab") {
          toast({
            title: "Lỗi",
            description: "Thiết bị không thuộc phòng lab này",
            variant: "destructive",
          });
        } else if (error.message === "Missing device ID or lab ID") {
          toast({
            title: "Lỗi",
            description: "Thiếu thông tin thiết bị hoặc phòng lab",
            variant: "destructive",
          });
        } else {
          toast({
            title: "Lỗi",
            description: "Không thể xử lý thiết bị",
            variant: "destructive",
          });
        }
      }
    }
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể xử lý mã QR",
      variant: "destructive",
    });
  }
};

const updateAuditCounts = () => {
  let good = 0;
  let damaged = 0;
  let missing = 0;
  let discarded = 0;

  devices.value.forEach((device) => {
    device.items.forEach((item) => {
      if (item.auditCondition === "healthy") good++;
      else if (item.auditCondition === "broken") damaged++;
      else if (item.auditCondition === "lost") missing++;
      else if (item.auditCondition === "discarded") discarded++;
    });
  });

  auditDetails.value.deviceConditions = {
    good,
    damaged,
    missing,
    discarded,
  };

  auditDetails.value.totalDevices = good + damaged + missing + discarded;
};

const handleOneTimeQRScan = async (input: string) => {
  try {
    const result = await verifyScannedQrCode(input);
    if (result && result.user) {
      const { user } = result;
      userInfo.value = {
        id: user.id,
        name: user.name,
        avatar: user.avatar,
        roles: user.roles,
      };

      toast({
        title: "Thành công",
        description: `Đã nhận diện: ${user.name}`,
        variant: "success",
      });
    }
  } catch (error) {
    toast({
      title: "Lỗi xử lý mã QR",
      description: "Vui lòng thử lại",
      variant: "destructive",
    });
  }
};

const handleVirtualKeyboardDetection = async (
  input: string,
  type?: "userId" | "device" | "oneTimeQR"
) => {
  if (type === "userId") {
    await handleUserCodeChange(input);
  } else if (type === "device") {
    await handleDeviceScan(input);
  } else if (type === "oneTimeQR") {
    await handleOneTimeQRScan(input);
  }
};

const toggleDevice = (device: AuditDevice) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded;
  }
};

const removeDeviceItem = (device: AuditDevice, itemId: string) => {
  device.items = device.items.filter((item) => item.id !== itemId);
  device.quantity = device.items.length;

  if (device.items.length === 0) {
    devices.value = devices.value.filter((d) => d.code !== device.code);
  }

  if (devices.value.length === 0) {
    mode.value = "idle";
  }

  updateAuditCounts();
};

const updateDeviceCondition = (
  item: AuditDeviceItem,
  condition: (typeof DeviceStatus)[keyof typeof DeviceStatus]
) => {
  item.auditCondition = condition;
  updateAuditCounts();
};

const goToHome = () => {
  showSuccessModal.value = false;
  mode.value = "idle";
  devices.value = [];
  notes.value = "";
  router.push("/");
};

const completeAudit = async () => {
  if (!userInfo.value) {
    toast({
      title: "Lỗi",
      description: "Vui lòng quét mã người dùng",
      variant: "destructive",
    });
    return;
  }

  if (devices.value.length === 0) {
    toast({
      title: "Lỗi",
      description: "Vui lòng quét ít nhất một thiết bị",
      variant: "destructive",
    });
    return;
  }

  isConfirming.value = true;
  try {
    const uniqueId = generateUniqueId();

    const deviceItems = devices.value.reduce(
      (acc, device) => {
        const items = device.items.map((item) => ({
          id: item.id,
          condition: item.auditCondition,
        }));

        const unscannedItems = getUnscannedItems(device).map((item) => ({
          id: `${device.code}/unscanned-${uniqueId}`,
          condition: item.auditCondition,
        }));

        return acc.concat(items, unscannedItems);
      },
      [] as {
        id: string;
        condition: (typeof DeviceStatus)[keyof typeof DeviceStatus];
      }[]
    );

    if (deviceItems.length === 0) {
      toast({
        title: "Lỗi",
        description: "Không có thiết bị nào được chọn để kiểm đếm.",
        variant: "destructive",
      });
      isConfirming.value = false;
      return;
    }

    await auditService.createAudit({
      id: uniqueId,
      auditorId: userInfo.value.id,
      location: auditDetails.value.location,
      devices: deviceItems,
      notes: notes.value || undefined,
    });

    successMessage.value = "Đã hoàn tất kiểm đếm thiết bị";
    auditId.value = uniqueId;
    showSuccessModal.value = true;
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể lưu dữ liệu kiểm đếm",
      variant: "destructive",
    });
  } finally {
    isConfirming.value = false;
  }
};

const getUnscannedCount = (device: AuditDevice) => {
  return (device.expectedQuantity || 0) - device.items.length;
};

const getUnscannedItems = (device: AuditDevice) => {
  const count = getUnscannedCount(device);
  return Array.from({ length: Math.max(0, count) }).map((_, index) => ({
    id: `unscanned-${device.code}-${index}`,
    status: DeviceStatus.LOST,
    auditCondition: DeviceStatus.LOST,
  }));
};

useVirtualKeyboardDetection(handleVirtualKeyboardDetection, {
  userId: { length: 7 },
  device: {
    pattern: /^https?:\/\/[^/]+\/devices\/[a-fA-F0-9]{8}\?id=[a-fA-F0-9]+$/,
  },
  oneTimeQR: {
    pattern:
      /^\{"token":"\d{6}","userId":"\d{7}","timestamp":\d+,"expiry":\d+\}$/,
  },
  scannerThresholdMs: 100,
  maxInputTimeMs: 1000,
});
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-center">GHI NHẬN KIỂM ĐẾM</h1>
    <p class="text-center text-gray-500 mb-6">
      Sử dụng máy scan quét mã QR thiết bị/người dùng để ghi nhận kiểm đếm
    </p>

    <div class="grid grid-cols-3 gap-6">
      <div
        class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <ClipboardCheckIcon class="h-5 w-5" />
            DANH SÁCH GHI NHẬN
          </h2>
        </div>

        <div class="h-[calc(100vh-16rem)] overflow-y-auto">
          <div
            v-if="devices.length === 0"
            class="flex flex-col items-center justify-center py-20 text-center"
          >
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <PackageIcon class="h-8 w-8 text-gray-400" />
            </div>
            <p class="text-sm text-gray-500 max-w-xs">
              Quét mã QR thiết bị để ghi nhận
              <br />
              kiểm đếm thiết bị
            </p>
          </div>

          <div v-else class="divide-y divide-gray-200">
            <div
              v-for="device in devices"
              :key="device.code"
              class="divide-y divide-gray-100"
            >
              <div
                class="p-4 hover:bg-gray-50 cursor-pointer"
                @click="toggleDevice(device)"
              >
                <div class="grid grid-cols-10 items-center">
                  <div class="flex items-center col-span-7 gap-3">
                    <img
                      :src="device.image.mainImage"
                      alt="Device image"
                      class="h-12 w-12 rounded-full object-cover"
                    />
                    <div>
                      <div class="flex items-center gap-2 mb-0.5">
                        <h3 class="font-medium text-gray-900 text-sm">
                          Mã loại:
                          <span class="font-bold text-base">{{
                            device.code
                          }}</span>
                        </h3>
                        <Badge
                          v-if="device.isBorrowableLabOnly"
                          variant="outline"
                          class="text-blue-600 border-blue-200 bg-blue-50 text-xs"
                        >
                          Không mượn về
                        </Badge>
                      </div>
                      <p class="text-base text-gray-900 font-medium">
                        {{ device.name }}
                      </p>
                    </div>
                  </div>
                  <div class="col-span-3 text-center flex items-center">
                    <span
                      class="text-base text-gray-900 font-medium w-full text-start"
                    >
                      SL: {{ device.quantity }} /
                      {{ device.expectedQuantity || device.quantity }}
                    </span>
                    <ChevronDownIcon
                      class="h-5 w-5 text-gray-400 transition-transform"
                      :class="{ 'rotate-180': device.expanded }"
                    />
                  </div>
                </div>
              </div>

              <div v-if="device.expanded" class="bg-gray-50">
                <div
                  class="grid grid-cols-10 items-center px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200"
                >
                  <span class="col-span-1"></span>
                  <span class="col-span-6">THIẾT BỊ GHI NHẬN</span>
                  <span class="col-span-3">TÌNH TRẠNG</span>
                </div>
                <div class="divide-y divide-gray-100">
                  <div
                    v-for="item in device.items"
                    :key="item.id"
                    class="grid grid-cols-10 items-center px-4 py-3"
                  >
                    <div class="col-span-1 flex justify-start">
                      <Button
                        variant="ghost"
                        size="icon"
                        @click.stop="removeDeviceItem(device, item.id)"
                        class="text-red-500 hover:text-red-600 hover:bg-red-100 rounded-full"
                      >
                        <TrashIcon class="h-4 w-4" />
                      </Button>
                    </div>
                    <div class="col-span-6 text-sm font-medium text-gray-900">
                      {{ device.code }}/{{ item.id }}
                    </div>
                    <div
                      class="col-span-3 flex items-center justify-start gap-2"
                    >
                      <span
                        :class="statusColorMap[item.status]"
                        class="text-sm font-semibold"
                      >
                        {{ statusMap[item.status] }}
                      </span>
                      <span class="text-gray-400">→</span>
                      <div class="w-32">
                        <Select
                          v-model="item.auditCondition"
                          class="flex-grow"
                          @update:modelValue="
                            updateDeviceCondition(item, item.auditCondition)
                          "
                        >
                          <SelectTrigger
                            class="h-9 text-sm bg-white text-base font-semibold w-fit"
                            :class="
                              item.auditCondition
                                ? statusColorMap[item.auditCondition]
                                : 'text-gray-900'
                            "
                          >
                            <SelectValue placeholder="Chọn tình trạng" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="healthy">
                              <span :class="statusColorMap['healthy']">{{
                                statusMap["healthy"]
                              }}</span>
                            </SelectItem>
                            <SelectItem value="broken">
                              <span :class="statusColorMap['broken']">{{
                                statusMap["broken"]
                              }}</span>
                            </SelectItem>
                            <SelectItem value="lost">
                              <span :class="statusColorMap['lost']">{{
                                statusMap["lost"]
                              }}</span>
                            </SelectItem>
                            <SelectItem value="discarded">
                              <span :class="statusColorMap['discarded']">{{
                                statusMap["discarded"]
                              }}</span>
                            </SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                  </div>
                </div>

                <div v-if="getUnscannedCount(device) > 0">
                  <div
                    class="grid grid-cols-10 items-center px-4 py-2 text-sm font-medium text-gray-500 border-y border-gray-200 mt-1"
                  >
                    <span class="col-span-1"></span>
                    <span class="col-span-6">THIẾT BỊ CHƯA GHI NHẬN</span>
                    <span class="col-span-3">TÌNH TRẠNG</span>
                  </div>

                  <div class="bg-amber-50 px-4 py-3 border-b border-amber-100">
                    <div class="grid grid-cols-10 items-center">
                      <div
                        class="h-6 w-6 rounded-full bg-amber-100 flex items-center justify-center"
                      >
                        <span class="text-amber-600 text-xs font-bold">!</span>
                      </div>
                      <span
                        class="text-sm font-medium text-gray-900 col-span-6"
                      >
                        Chưa ghi nhận:
                        <span class="font-bold text-amber-600"
                          >{{ getUnscannedCount(device) }}
                          {{ device.unit }}</span
                        >
                      </span>
                      <div class="text-sm text-amber-600 col-span-3">
                        Cần được xác định tình trạng
                      </div>
                    </div>
                  </div>

                  <div class="divide-y divide-gray-100">
                    <div
                      v-for="(unscannedItem, index) in getUnscannedItems(
                        device
                      )"
                      :key="unscannedItem.id"
                      class="grid grid-cols-10 items-center px-4 py-3"
                    >
                      <div class="col-span-1 flex justify-start"></div>
                      <div class="col-span-5 text-sm font-medium text-gray-900">
                        {{ device.code }}/unscanned-{{ index + 1 }}
                      </div>
                      <div class="col-span-4">
                        <div class="flex items-center justify-center gap-2">
                          <div class="w-32">
                            <Select
                              v-model="unscannedItem.auditCondition"
                              class="flex-grow"
                            >
                              <SelectTrigger
                                class="h-9 text-sm bg-white text-base font-semibold w-full"
                                :class="
                                  unscannedItem.auditCondition
                                    ? statusColorMap[
                                        unscannedItem.auditCondition
                                      ]
                                    : 'text-gray-900'
                                "
                              >
                                <SelectValue placeholder="Chọn tình trạng" />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="lost">
                                  <span :class="statusColorMap['lost']">{{
                                    statusMap["lost"]
                                  }}</span>
                                </SelectItem>
                                <SelectItem value="discarded">
                                  <span :class="statusColorMap['discarded']">{{
                                    statusMap["discarded"]
                                  }}</span>
                                </SelectItem>
                              </SelectContent>
                            </Select>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div
        class="flex-1 bg-white rounded-lg shadow-sm border border-gray-200 h-fit"
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <UserIcon class="h-5 w-5" />
            NGƯỜI KIỂM ĐẾM
          </h2>
        </div>

        <div>
          <div class="space-y-4 bg-gray-50 rounded-lg p-2">
            <div
              v-if="!userInfo"
              class="border border-dashed border-gray-300 rounded-lg p-1 flex flex-col items-center justify-center"
            >
              <div class="bg-gray-100 rounded-full p-3">
                <UserIcon class="h-6 w-6 text-gray-400" />
              </div>
              <p class="text-sm text-gray-500 text-center">
                Quét QR định danh người dùng
              </p>
            </div>

            <div v-else class="rounded-lg px-4 py-1">
              <div class="flex items-center">
                <img
                  :src="userInfo.avatar"
                  alt="User avatar"
                  class="h-12 w-12 rounded-full object-cover"
                />
                <div class="ml-3">
                  <h4 class="text-sm font-medium text-gray-500">
                    Mã số:
                    <span class="text-gray-500 font-semibold">{{
                      userInfo.id
                    }}</span>
                    <span class="text-sm text-gray-500 italic font-semibold">
                      ({{
                        userInfo.roles?.map((r) => r.name).join(", ") ||
                        "Không có vai trò"
                      }})
                    </span>
                  </h4>
                  <p class="text-base font-semibold text-gray-900">
                    {{ userInfo.name }}
                  </p>
                </div>
              </div>
            </div>
          </div>

          <div
            v-if="mode === 'audit' && devices.length > 0"
            class="p-4 border-t border-gray-200"
          >
            <div class="flex items-center gap-3">
              <div class="rounded-full bg-blue-50 p-2">
                <BoxIcon class="h-4 w-4 text-blue-600" />
              </div>
              <div class="flex justify-between w-full">
                <p class="text-sm text-gray-500">Tổng thiết bị</p>
                <p class="font-medium text-blue-600 text-base">
                  {{ totalDevices }}
                </p>
              </div>
            </div>

            <div class="flex gap-4">
              <div class="w-px bg-gray-200 relative ml-12">
                <!-- Dots or markers can be added here if needed -->
              </div>
              <div class="space-y-2 flex-1">
                <div class="flex justify-between">
                  <span class="text-sm text-green-600 font-medium">Tốt</span>
                  <span class="text-sm font-medium text-gray-700">{{
                    auditDetails.deviceConditions.good
                  }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-red-600 font-medium">Hư</span>
                  <span class="text-sm font-medium text-gray-700">{{
                    auditDetails.deviceConditions.damaged
                  }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500 font-medium">Mất</span>
                  <span class="text-sm font-medium text-gray-700">{{
                    auditDetails.deviceConditions.missing
                  }}</span>
                </div>
                <div class="flex justify-between">
                  <span class="text-sm text-gray-500 font-medium">Loại bỏ</span>
                  <span class="text-sm font-medium text-gray-700">{{
                    auditDetails.deviceConditions.discarded
                  }}</span>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-amber-50 p-2">
                <MapPinIcon class="h-4 w-4 text-amber-600" />
              </div>
              <div class="flex justify-between w-full">
                <p class="text-sm text-gray-500">Nơi thực hiện</p>
                <p class="font-medium text-gray-800 text-right">
                  {{ auditDetails.location }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-green-50 p-2">
                <CalendarIcon class="h-4 w-4 text-green-600" />
              </div>
              <div class="flex justify-between w-full">
                <p class="text-sm text-gray-500">Ngày thực hiện</p>
                <p class="font-medium text-gray-800">
                  {{ auditDetails.auditDate }}
                </p>
              </div>
            </div>

            <div class="mt-4">
              <label
                for="notes"
                class="block text-sm font-medium text-gray-700 mb-1"
                >Ghi chú</label
              >
              <Textarea
                id="notes"
                v-model="notes"
                placeholder="Thêm ghi chú về tình trạng thiết bị hoặc kết quả kiểm đếm..."
                class="min-h-[80px]"
              />
            </div>

            <button
              :disabled="isConfirming || !userInfo || devices.length === 0"
              class="w-full mt-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="completeAudit"
            >
              <svg
                v-if="isConfirming"
                class="mr-3 -ml-1 size-5 animate-spin text-white"
                xmlns="http://www.w3.org/2000/svg"
                fill="none"
                viewBox="0 0 24 24"
              >
                <circle
                  class="opacity-25"
                  cx="12"
                  cy="12"
                  r="10"
                  stroke="currentColor"
                  stroke-width="4"
                ></circle>
                <path
                  class="opacity-75"
                  fill="currentColor"
                  d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                ></path>
              </svg>
              <ClipboardCheckIcon v-else class="h-5 w-5" />
              Hoàn tất kiểm đếm
            </button>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showSuccessModal"
      class="fixed inset-0 flex items-center justify-center z-50"
    >
      <div
        class="fixed inset-0 bg-black bg-opacity-60"
        @click="showSuccessModal = false"
      ></div>

      <div
        class="bg-white rounded-lg shadow-xl z-10 max-w-md w-full mx-4 overflow-hidden"
      >
        <div class="flex flex-col items-center text-center py-6 px-6">
          <div
            class="mx-auto flex h-20 w-20 items-center justify-center rounded-full bg-green-100 mb-4"
          >
            <CheckIcon class="h-12 w-12 text-green-600" />
          </div>
          <h2 class="text-xl font-semibold leading-6 text-gray-900 mb-2">
            Hoàn tất
          </h2>
          <p class="text-lg text-gray-900 mb-1">
            {{ successMessage }}
          </p>
          <p v-if="auditId" class="text-base text-gray-600 mb-6">
            Mã đơn: {{ auditId }}
          </p>
          <Button
            type="button"
            class="w-full justify-center rounded-md bg-blue-600 px-3 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700"
            @click="goToHome"
          >
            Về trang chủ
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
