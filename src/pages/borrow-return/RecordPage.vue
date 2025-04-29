<script setup lang="ts">
import {
  CalendarDate,
  DateFormatter,
  getLocalTimeZone,
  parseDate,
  today,
} from "@internationalized/date";
import {
  BoxIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  InfoIcon,
  LoaderIcon,
  MapPinIcon,
  PackageCheckIcon,
  PackageIcon,
  TrashIcon,
  UserIcon,
} from "lucide-vue-next";

const mode = ref<"idle" | "borrow" | "return">("idle");

const deviceBorrowerMap = ref(new Map<string, string>());

const df = new DateFormatter("vi-VN", {
  day: "2-digit",
  month: "2-digit",
  year: "numeric",
});

const userInfo = ref<UserInfo | null>(null);
const storedUserInfo = ref<{
  id: string;
  lab: { id: string; room: string; branch: string };
} | null>(null);

const devices = ref<(Device & { items: QualityDeviceItem[] })[]>([]);

const notes = ref<string>("");

const borrowDetails = ref<{
  location: string;
  borrowDate: string;
  returnDate?: Date;
}>({
  location: "",
  borrowDate: df.format(new Date()),
  returnDate: undefined,
});

const returnDetails = ref<{
  location: string;
  expectedReturnAt: string;
  actualReturnDate: string;
  returnProgress: string;
  notes: string;
}>({
  location: "",
  expectedReturnAt: df.format(new Date()),
  actualReturnDate: df.format(new Date()),
  returnProgress: "",
  notes: "",
});

function generateUniqueId(): string {
  const now = new Date();
  const datePrefix = now.toISOString().split("T")[0].replace(/-/g, "");
  const randomSuffix = Math.floor(Math.random() * 1000000)
    .toString()
    .padStart(6, "0");
  return `${datePrefix}/${randomSuffix}`;
}

const router = useRouter();

const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0);
});

const pageTitle = computed(() => {
  if (mode.value === "borrow") return "GHI NHẬN MƯỢN";
  if (mode.value === "return") return "GHI NHẬN TRẢ";
  return "GHI NHẬN MƯỢN TRẢ";
});

const leftColumnTitle = computed(() => {
  if (mode.value === "borrow") return "DANH SÁCH MƯỢN";
  if (mode.value === "return") return "DANH SÁCH TRẢ";
  return "DANH SÁCH GHI NHẬN";
});

const rightColumnTitle = computed(() => {
  if (mode.value === "borrow") return "NGƯỜI MƯỢN";
  if (mode.value === "return") return "NGƯỜI TRẢ";
  return "THÔNG TIN NGƯỜI MƯỢN/TRẢ";
});

const overallReturnStatus = computed(() => {
  if (!devices.value || devices.value.length === 0) {
    return "";
  }

  const isAnyDeviceLate = devices.value.some((device) =>
    device.items.some((item) => {
      const qualityItem = item as QualityDeviceItem;
      if (!qualityItem.expectedReturnAt) return false;
      return calculateReturnProgress(qualityItem.expectedReturnAt).includes(
        "Trễ"
      );
    })
  );

  return isAnyDeviceLate ? "Trễ hạn" : "Đúng hạn";
});

const { verifyScannedQrCode } = useOneTimeQR();

const isConfirming = ref(false);
const isLoadingDeviceScan = ref(false);
const isLoadingUser = ref(false);
const showSuccessModal = ref(false);
const successMessage = ref("");
const receiptId = ref("");

const hasBorrowableLabOnlyDevice = computed(() => {
  return devices.value.some((device) => device.isBorrowableLabOnly);
});

const calendarModel = computed({
  get: () => {
    if (hasBorrowableLabOnlyDevice.value) {
      return parseDate(new Date().toISOString().split("T")[0]);
    }
    if (borrowDetails.value.returnDate) {
      return parseDate(
        borrowDetails.value.returnDate.toISOString().split("T")[0]
      );
    }
    return undefined;
  },
  set: (val: CalendarDate | undefined) => {
    if (hasBorrowableLabOnlyDevice.value) {
      borrowDetails.value.returnDate = new Date();
      return;
    }
    if (val) {
      const date = new Date(val.year, val.month - 1, val.day, 12, 0, 0);
      borrowDetails.value.returnDate = date;
    } else {
      borrowDetails.value.returnDate = undefined;
    }
  },
});

const goToHome = () => {
  showSuccessModal.value = false;
  mode.value = "idle";
  devices.value = [];
  notes.value = "";
  userInfo.value = null;
  router.push("/");
};

onMounted(async () => {
  const stored = localStorage.getItem("user_info");
  if (stored) {
    const ui = JSON.parse(stored) as {
      id: string;
      lab: { id: string; room: string; branch: string };
    };
    const loc = ui.lab ? `${ui.lab.room}, ${ui.lab.branch}` : "";
    borrowDetails.value.location = loc;
    returnDetails.value.location = loc;
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
  isLoadingUser.value = true;
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

    if (mode.value === "return" && devices.value.length > 0) {
      for (const device of devices.value) {
        for (const item of device.items) {
          const borrowerId = deviceBorrowerMap.value.get(item.id);
          if (borrowerId && borrowerId !== userMeta.id) {
            toast({
              title: "Lỗi",
              description:
                "Người dùng không khớp với người mượn của một số thiết bị đã quét",
              variant: "destructive",
            });
            userInfo.value = null;
            return;
          }
        }
      }
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
    isLoadingUser.value = false;
  }
}

const calculateReturnProgress = (expectedDate: string) => {
  const today = new Date();
  const returnDate = new Date(expectedDate);
  if (today > returnDate) {
    return "Trễ hạn";
  } else {
    const diffTime = Math.abs(returnDate.getTime() - today.getTime());
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    return `Đúng hạn (còn ${diffDays} ngày)`;
  }
};

const handleDeviceScan = async (input: string) => {
  isLoadingDeviceScan.value = true;
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
      const deviceDetails = await deviceService.getDeviceReceiptById(
        deviceId,
        storedUserInfo.value?.lab.id || ""
      )!;

      if (deviceDetails.borrower?.id) {
        deviceBorrowerMap.value.set(deviceId, deviceDetails.borrower.id);
      }

      if (mode.value === "return") {
        if (userInfo.value) {
          if (deviceDetails.borrower?.id !== userInfo.value.id) {
            toast({
              title: "Lỗi",
              description: `Thiết bị này được mượn bởi người dùng khác (${deviceDetails.borrower?.name || "Không xác định"})`,
              variant: "destructive",
            });
            return;
          }
        } else if (devices.value.length > 0) {
          const firstItem = devices.value[0].items[0];
          if (firstItem) {
            const firstBorrowerId = deviceBorrowerMap.value.get(firstItem.id);
            if (
              firstBorrowerId &&
              firstBorrowerId !== deviceDetails.borrower?.id
            ) {
              toast({
                title: "Lỗi",
                description:
                  "Thiết bị này được mượn bởi người dùng khác, không khớp với các thiết bị đã quét",
                variant: "destructive",
              });
              return;
            }
          }
        }
      }

      if (mode.value === "idle") {
        if (
          deviceDetails.status === DeviceStatus.HEALTHY ||
          deviceDetails.status === DeviceStatus.BROKEN
        ) {
          mode.value = "borrow";
        } else if (deviceDetails.status === DeviceStatus.BORROWING) {
          mode.value = "return";
        } else {
          toast({
            title: "Thông báo",
            description: `Thiết bị đang ở trạng thái '${statusMap[deviceDetails.status!]}', không thể mượn/trả.`,
            variant: "destructive",
          });
          return;
        }
      }

      if (
        mode.value === "borrow" &&
        deviceDetails.status !== DeviceStatus.HEALTHY &&
        deviceDetails.status !== DeviceStatus.BROKEN
      ) {
        toast({
          title: "Lỗi",
          description: `Thiết bị không khả dụng để mượn (ID: ${deviceId})`,
          variant: "destructive",
        });
        return;
      }

      if (
        mode.value === "return" &&
        deviceDetails.status !== DeviceStatus.BORROWING
      ) {
        toast({
          title: "Lỗi",
          description: `Thiết bị không ở trạng thái đang mượn (ID: ${deviceId})`,
          variant: "destructive",
        });
        return;
      }

      if (mode.value === "borrow") {
        const existingDevice = devices.value.find(
          (d) => d.code === deviceKindId
        );
        if (existingDevice) {
          existingDevice.items.push({
            id: deviceId,
            status: deviceDetails.status!,
            prevQuality: DeviceStatus.HEALTHY,
            expectedReturnAt: deviceDetails.expectedReturnAt,
          });
          existingDevice.quantity = existingDevice.items.length;
        } else {
          devices.value.push({
            code: deviceKindId!,
            name: deviceDetails.deviceName,
            image: deviceDetails.image,
            quantity: 1,
            unit: deviceDetails.unit,
            expanded: true,
            items: [
              {
                id: deviceId,
                status: deviceDetails.status!,
                prevQuality: DeviceStatus.HEALTHY,
                expectedReturnAt: deviceDetails.expectedReturnAt,
              },
            ],
            isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly,
          });
        }
        toast({
          title: "Thành công",
          description: "Đã thêm thiết bị vào danh sách mượn",
          variant: "success",
        });
      } else if (mode.value === "return") {
        const qualityValue = deviceDetails.prevQuality || DeviceStatus.HEALTHY;
        const returnDate =
          deviceDetails.expectedReturnAt || df.format(new Date());

        const newItem = {
          id: deviceId,
          status: deviceDetails.status!,
          returnCondition: DeviceStatus.HEALTHY,
          prevQuality: qualityValue,
          expectedReturnAt: returnDate,
        };

        const existingDevice = devices.value.find(
          (d) => d.code === deviceKindId
        );
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
            isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly,
          });
        }
        toast({
          title: "Thành công",
          description: "Đã thêm thiết bị vào danh sách trả",
          variant: "success",
        });
      }
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
  } finally {
    isLoadingDeviceScan.value = false;
  }
};

async function handleConfirmBorrow() {
  if (!userInfo.value || devices.value.length === 0) {
    return;
  }

  if (!borrowDetails.value.returnDate) {
    toast({
      title: "Lỗi",
      description: "Vui lòng chọn ngày hẹn trả",
      variant: "destructive",
    });
    return;
  }

  isConfirming.value = true;
  try {
    if (!storedUserInfo.value?.lab?.id) {
      toast({
        title: "Lỗi",
        description: "Không tìm thấy thông tin phòng lab",
        variant: "destructive",
      });
      return;
    }

    const allDeviceItems = devices.value.reduce((acc, device) => {
      const items = device.items.map((item) => ({
        id: item.id,
        prevQuality:
          (item as QualityDeviceItem).prevQuality || DeviceStatus.HEALTHY,
        expectedReturnedAt: borrowDetails.value.returnDate!,
        expectedReturnedLabId: storedUserInfo.value?.lab.id,
      }));
      return acc.concat(items);
    }, [] as any[]);

    if (allDeviceItems.length === 0) {
      toast({
        title: "Lỗi",
        description: "Không có thiết bị nào được chọn để mượn.",
        variant: "destructive",
      });
      isConfirming.value = false;
      return;
    }

    const uniqueId = generateUniqueId();
    await receiptService.createReceipt({
      id: uniqueId,
      borrowerId: userInfo.value.id,
      borrowCheckerId: storedUserInfo.value?.id,
      borrowedLabId: storedUserInfo.value?.lab.id,
      devices: allDeviceItems,
    });

    successMessage.value = "Ghi nhận mượn thành công";
    receiptId.value = uniqueId;
    showSuccessModal.value = true;
  } catch (e) {
    toast({
      title: "Lỗi",
      description: (e as Error).message,
      variant: "destructive",
    });
  } finally {
    isConfirming.value = false;
  }
}

async function handleConfirmReturn() {
  if (!userInfo.value || devices.value.length === 0) {
    return;
  }

  isConfirming.value = true;
  try {
    if (!storedUserInfo.value?.lab?.id) {
      toast({
        title: "Lỗi",
        description: "Người dùng không thuộc phòng lab nào",
        variant: "destructive",
      });
      return;
    }

    const allDeviceItems = devices.value.reduce((acc, device) => {
      const items = device.items.map((item: QualityDeviceItem) => ({
        id: item.id,
        afterQuality: item.returnCondition || DeviceStatus.HEALTHY,
      }));
      return acc.concat(items);
    }, [] as any[]);

    if (allDeviceItems.length === 0) {
      toast({
        title: "Lỗi",
        description: "Không có thiết bị nào được chọn để trả.",
        variant: "destructive",
      });
      isConfirming.value = false;
      return;
    }

    const borrowReceiptId = devices.value[0]?.items[0]?.id;
    if (!borrowReceiptId) {
      toast({
        title: "Lỗi",
        description: "Không thể xác định mã phiếu mượn gốc.",
        variant: "destructive",
      });
      isConfirming.value = false;
      return;
    }

    const returnReceiptUniqueId = generateUniqueId();

    await receiptService.returnReceipt({
      id: returnReceiptUniqueId,
      returnerId: userInfo.value.id,
      returnedCheckerId: storedUserInfo.value?.id,
      returnedLabId: storedUserInfo.value?.lab.id,
      devices: allDeviceItems,
      note: notes.value,
    });

    successMessage.value = "Ghi nhận trả thành công";
    receiptId.value = returnReceiptUniqueId;
    showSuccessModal.value = true;
  } catch (e) {
    toast({
      title: "Lỗi",
      description: (e as Error).message,
      variant: "destructive",
    });
  } finally {
    isConfirming.value = false;
  }
}

const handleOneTimeQRScan = async (input: string) => {
  isLoadingUser.value = true;
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
  } finally {
    isLoadingUser.value = false;
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

const toggleDevice = (device: Device) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded;
  }
};

const removeDeviceItem = (device: Device, itemId: string) => {
  device.items = device.items.filter(
    (item) => item.id !== itemId
  ) as QualityDeviceItem[];
  device.quantity = device.items.length;

  if (device.items.length === 0) {
    devices.value = devices.value.filter((d) => d.code !== device.code);
  }

  if (devices.value.length === 0) {
    mode.value = "idle";
  }
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
    <h1 class="text-2xl font-bold text-center">{{ pageTitle }}</h1>
    <p class="text-center text-gray-500 mb-2">
      Sử dụng máy scan quét mã QR thiết bị/người dùng để ghi nhận mượn trả
    </p>

    <div class="grid grid-cols-3 gap-6">
      <div
        class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <PackageCheckIcon class="h-5 w-5" />
            {{ leftColumnTitle }}
          </h2>
        </div>

        <div class="h-[calc(100vh-16rem)] overflow-y-auto">
          <div
            v-if="devices.length === 0"
            class="flex flex-col items-center justify-center py-20 text-center"
          >
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <LoaderIcon
                v-if="isLoadingDeviceScan"
                class="h-8 w-8 text-blue-500 animate-spin"
              />
              <PackageIcon v-else class="h-8 w-8 text-gray-400" />
            </div>
            <p class="text-sm text-gray-500 max-w-xs">
              {{
                isLoadingDeviceScan
                  ? "Đang xử lý thiết bị..."
                  : "Quét mã QR thiết bị để ghi nhận mượn trả thiết bị"
              }}
            </p>
          </div>

          <div v-else-if="mode === 'borrow'" class="divide-y divide-gray-200">
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
                  <div class="flex items-center col-span-6 gap-3">
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
                  <div class="col-span-4 text-center flex items-center">
                    <span class="text-base text-gray-900 font-medium w-full">
                      SL: {{ device.quantity }} {{ device.unit }}
                    </span>
                    <ChevronDownIcon
                      class="h-5 w-5 text-gray-400 transition-transform"
                      :class="{ 'rotate-180': device.expanded }"
                    />
                  </div>
                </div>
              </div>

              <div v-if="device.expanded" class="bg-gray-50">
                <div class="p-4">
                  <div class="grid grid-cols-10 items-center mb-2">
                    <div class="col-span-1"></div>
                    <h4 class="text-sm font-medium text-gray-500 col-span-5">
                      THIẾT BỊ GHI NHẬN
                    </h4>
                    <h4
                      class="text-sm font-medium text-gray-500 col-span-4 text-center"
                    >
                      TÌNH TRẠNG
                    </h4>
                  </div>
                  <div class="space-y-3">
                    <div
                      v-for="item in device.items"
                      :key="item.id"
                      class="flex items-center justify-between grid grid-cols-10"
                    >
                      <div class="text-left">
                        <Button
                          variant="ghost"
                          size="icon"
                          @click.stop="removeDeviceItem(device, item.id)"
                          class="text-red-500 hover:text-red-600 hover:bg-red-100 rounded-full"
                        >
                          <TrashIcon class="h-4 w-4" />
                        </Button>
                      </div>
                      <div class="text-sm font-medium text-gray-900 col-span-5">
                        {{ device.code }}/{{ item.id }}
                      </div>
                      <div class="col-span-4 text-center">
                        <Badge
                          :class="statusColorMap[item.status]"
                          class="text-base font-semibold text-center"
                          variant="outline"
                        >
                          {{ statusMap[item.status] }}
                        </Badge>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <div v-else-if="mode === 'return'" class="divide-y divide-gray-200">
            <div
              v-for="device in devices"
              :key="device.code"
              class="divide-y divide-gray-100"
            >
              <div
                class="p-4 hover:bg-gray-50 cursor-pointer"
                :class="{
                  'cursor-pointer': device.items.length > 0,
                  'opacity-50': device.items.length === 0,
                }"
                @click="toggleDevice(device)"
              >
                <div class="grid grid-cols-12 items-center">
                  <div class="col-span-9 flex items-center gap-3">
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
                  <div class="col-span-3 flex items-center">
                    <span class="text-base text-gray-900 font-medium w-full">
                      SL: {{ device.quantity }} {{ device.unit }}
                    </span>
                    <ChevronDownIcon
                      class="h-5 w-5 text-gray-400 transition-transform justify-self-end"
                      :class="{ 'rotate-180': device.expanded }"
                    />
                  </div>
                </div>
              </div>

              <div
                v-if="device.expanded && device.items.length > 0"
                class="bg-gray-50"
              >
                <div
                  class="grid grid-cols-12 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200"
                >
                  <div class="col-span-1"></div>
                  <div class="col-span-3">THIẾT BỊ GHI NHẬN</div>
                  <div class="col-span-3">TIẾN ĐỘ TRẢ</div>
                  <div class="col-span-2">HẸN TRẢ</div>
                  <div class="col-span-3">TÌNH TRẠNG</div>
                </div>
                <div
                  v-for="item in device.items"
                  :key="item.id"
                  class="grid grid-cols-12 items-center px-4 py-3 border-b border-gray-100 last:border-b-0"
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
                  <div class="col-span-3 text-sm font-medium text-gray-900">
                    {{ device.code }}/{{ item.id }}
                  </div>
                  <div
                    class="col-span-3 text-sm"
                    :class="
                      item.expectedReturnAt
                        ? calculateReturnProgress(
                            item.expectedReturnAt
                          ).includes('Trễ')
                          ? 'text-red-600'
                          : 'text-gray-600'
                        : 'text-gray-600'
                    "
                  >
                    {{
                      item.expectedReturnAt
                        ? calculateReturnProgress(item.expectedReturnAt)
                        : "Chưa có ngày hẹn"
                    }}
                  </div>
                  <div class="col-span-2 text-sm text-gray-600">
                    {{
                      item.expectedReturnAt
                        ? df.format(new Date(item.expectedReturnAt))
                        : "---"
                    }}
                  </div>
                  <div class="col-span-3">
                    <div class="flex items-center gap-1">
                      <Badge
                        :class="
                          qualityColorMap[
                            (item as QualityDeviceItem).prevQuality ||
                              DeviceStatus.HEALTHY
                          ]
                        "
                        class="h-8 text-sm font-semibold w-fit"
                        variant="outline"
                      >
                        {{
                          qualityMap[
                            (item as QualityDeviceItem).prevQuality ||
                              DeviceStatus.HEALTHY
                          ]
                        }}
                      </Badge>
                      <span class="text-gray-400 mx-1">→</span>
                      <Select
                        v-model="(item as QualityDeviceItem).returnCondition"
                        class="flex-grow"
                      >
                        <SelectTrigger
                          class="h-8 text-sm bg-white font-semibold w-fit"
                          :class="
                            (item as QualityDeviceItem).returnCondition
                              ? qualityColorMap[
                                  (item as QualityDeviceItem).returnCondition!
                                ]
                              : 'text-gray-900'
                          "
                        >
                          <SelectValue placeholder="Chọn tình trạng" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem
                            v-for="(label, status) in qualityMap"
                            :key="status"
                            :value="status"
                            class="cursor-pointer"
                          >
                            <Badge
                              :class="qualityColorMap[status]"
                              variant="outline"
                            >
                              {{ label }}
                            </Badge>
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

      <div
        class="flex-1 bg-white rounded-lg shadow-sm border border-gray-200 h-fit"
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <UserIcon class="h-5 w-5" />
            {{ rightColumnTitle }}
          </h2>
        </div>

        <div>
          <div class="space-y-4 bg-gray-50 rounded-lg p-2">
            <div
              v-if="!userInfo"
              class="border border-dashed border-gray-300 rounded-lg p-1 flex flex-col items-center justify-center"
            >
              <div class="bg-gray-100 rounded-full p-3">
                <LoaderIcon
                  v-if="isLoadingUser"
                  class="h-6 w-6 text-blue-500 animate-spin"
                />
                <UserIcon v-else class="h-6 w-6 text-gray-400" />
              </div>
              <p class="text-sm text-gray-500 text-center">
                {{
                  isLoadingUser
                    ? "Đang xử lý người dùng..."
                    : "Quét QR định danh người dùng"
                }}
              </p>
            </div>

            <div v-else class="rounded-lg px-4 py-1">
              <div class="flex items-center">
                <img
                  :src="userInfo.avatar || undefined"
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
            v-if="mode === 'borrow' && devices.length > 0"
            class="p-4 border-t border-gray-200"
          >
            <div class="flex items-center gap-3">
              <div class="rounded-full bg-blue-50 p-2">
                <BoxIcon class="h-4 w-4 text-blue-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Tổng thiết bị</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ totalDevices }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-amber-50 p-2">
                <MapPinIcon class="h-4 w-4 text-amber-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Nơi mượn</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ borrowDetails.location }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-green-50 p-2">
                <CalendarIcon class="h-4 w-4 text-green-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Ngày mượn</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ borrowDetails.borrowDate }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-purple-50 p-2">
                <CalendarIcon class="h-4 w-4 text-purple-600" />
              </div>
              <div class="grid grid-cols-2 w-full items-center">
                <p class="text-sm text-gray-500">Ngày hẹn trả</p>
                <div class="text-right">
                  <template v-if="hasBorrowableLabOnlyDevice">
                    <TooltipProvider>
                      <Tooltip>
                        <TooltipTrigger as-child>
                          <div
                            class="text-base font-medium text-gray-800 inline-flex gap-2 items-center mr-4"
                          >
                            <InfoIcon class="h-4 w-4 text-gray-400" />
                            {{ borrowDetails.borrowDate }}
                          </div>
                        </TooltipTrigger>
                        <TooltipContent>
                          <p>Trả trong ngày</p>
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  </template>
                  <template v-else>
                    <Popover class="text-right">
                      <PopoverTrigger
                        as-child
                        class="font-medium text-gray-800 justify-self-end"
                      >
                        <Button
                          variant="outline"
                          :class="
                            cn(
                              'w-auto justify-end font-normal text-base',
                              !borrowDetails.returnDate &&
                                'text-muted-foreground'
                            )
                          "
                        >
                          <CalendarIcon class="h-4 w-4 opacity-50" />
                          {{
                            borrowDetails.returnDate &&
                            borrowDetails.returnDate instanceof Date
                              ? df.format(borrowDetails.returnDate)
                              : "Chọn ngày trả"
                          }}
                        </Button>
                      </PopoverTrigger>
                      <PopoverContent class="w-auto p-0">
                        <Calendar
                          v-model="calendarModel"
                          calendar-label="Return date"
                          initial-focus
                          :min-value="today(getLocalTimeZone())"
                        />
                      </PopoverContent>
                    </Popover>
                  </template>
                </div>
              </div>
            </div>

            <Button
              :disabled="isConfirming || !userInfo || devices.length === 0"
              class="w-full mt-4 bg-blue-600 hover:bg-blue-700"
              @click="handleConfirmBorrow"
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
              <PackageCheckIcon v-else class="h-5 w-5 mr-2" />
              Xác nhận mượn
            </Button>
          </div>

          <div
            v-if="mode === 'return' && devices.length > 0"
            class="p-4 border-t border-gray-200"
          >
            <div class="flex items-center gap-3">
              <div class="rounded-full bg-blue-50 p-2">
                <BoxIcon class="h-4 w-4 text-blue-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Tổng thiết bị</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ totalDevices }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-amber-50 p-2">
                <MapPinIcon class="h-4 w-4 text-amber-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Nơi trả</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ returnDetails.location }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-red-50 p-2">
                <CalendarIcon class="h-4 w-4 text-red-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Tiến độ trả</p>
                <p
                  class="font-medium text-right mr-4"
                  :class="
                    overallReturnStatus === 'Trễ hạn'
                      ? 'text-red-600'
                      : 'text-green-600'
                  "
                >
                  {{ overallReturnStatus }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-indigo-50 p-2">
                <CalendarIcon class="h-4 w-4 text-indigo-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Ngày trả thực tế</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ returnDetails.actualReturnDate }}
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
                placeholder="Thêm ghi chú về tình trạng thiết bị hoặc lý do trả trễ (nếu có)..."
                class="min-h-[80px]"
              ></Textarea>
            </div>

            <Button
              :disabled="isConfirming || !userInfo || devices.length === 0"
              class="w-full mt-4 bg-blue-600 hover:bg-blue-700"
              @click="handleConfirmReturn"
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
              <PackageCheckIcon v-else class="h-5 w-5 mr-2" />
              Xác nhận trả
            </Button>
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
          <p v-if="receiptId" class="text-base text-gray-600 mb-6">
            Mã đơn: {{ receiptId }}
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
