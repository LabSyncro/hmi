<script setup lang="ts">
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { toast } from "@/components/ui/toast";
import { useOneTimeQR, useVirtualKeyboardDetection } from "@/composables";
import {
  deviceService,
  receiptService,
  userService,
  type DeviceQuality,
  type DeviceStatus,
} from "@/lib/db";
import { cn } from "@/lib/utils";
import {
  qualityColorMap,
  qualityMap,
  statusColorMap,
  statusMap,
  type Device,
  type ReturnDeviceItem,
  type UserInfo,
} from "@/types/status";
import {
  CalendarDate,
  DateFormatter,
  getLocalTimeZone,
  parseDate,
  today,
} from "@internationalized/date";
import {
  Box,
  CalendarIcon,
  ChevronDown,
  MapPin,
  Package,
  PackageCheck,
  Trash,
  User,
} from "lucide-vue-next";
import { computed, onMounted, ref } from "vue";
import { useRoute } from "vue-router";

const mode = ref<"idle" | "borrow" | "return">("idle");

const df = new DateFormatter("vi-VN", {
  day: "2-digit",
  month: "2-digit",
  year: "numeric",
});

const userInfo = ref<UserInfo | null>(null);

const devices = ref<
  (Device & { items: ReturnDeviceItem[]; isBorrowableLabOnly?: boolean })[]
>([]);

const notes = ref<string>("");

const borrowDetails = ref<{
  location: string;
  borrowDate: string;
  returnDate?: Date;
}>({
  location: "",
  borrowDate: df.format(new Date()),
  returnDate: new Date(Date.now() + 7 * 24 * 60 * 60 * 1000),
});

const returnDetails = ref<{
  location: string;
  borrowDate: string;
  returnDate: string;
  actualReturnDate: string;
  returnProgress: string;
}>({
  location: "601 H6, Dĩ An",
  borrowDate: "12/03/2025",
  returnDate: "16/03/2025",
  actualReturnDate: new Date().toLocaleDateString("vi-VN"),
  returnProgress: "Trễ hạn",
});

function generateUniqueId(): string {
  const now = new Date();
  const datePrefix = now.toISOString().split("T")[0].replace(/-/g, "");
  const randomSuffix = Math.floor(Math.random() * 1000000)
    .toString()
    .padStart(6, "0");
  return `${datePrefix}/${randomSuffix}`;
}

const route = useRoute();

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

const { verifyScannedQrCode } = useOneTimeQR();

const calendarModel = computed({
  get: () => {
    if (borrowDetails.value.returnDate) {
      return parseDate(
        borrowDetails.value.returnDate.toISOString().split("T")[0]
      );
    }
    return undefined;
  },
  set: (val: CalendarDate | undefined) => {
    if (val) {
      const date = new Date(val.year, val.month - 1, val.day, 12, 0, 0);
      borrowDetails.value.returnDate = date;
    } else {
      borrowDetails.value.returnDate = undefined;
    }
  },
});

onMounted(async () => {
  const stored = localStorage.getItem("user_info");
  if (stored) {
    const ui = JSON.parse(stored) as { lab?: { room: string; branch: string } };
    const loc = ui.lab ? `${ui.lab.room}, ${ui.lab.branch}` : "";
    borrowDetails.value.location = loc;
    returnDetails.value.location = loc;
  }
  const {
    userId,
    userName,
    userAvatar,
    userRoles,
    deviceId,
    deviceName,
    deviceImage,
    deviceStatus,
    deviceKindId,
    deviceUnit,
  } = route.query;

  if (userId) {
    userInfo.value = {
      id: userId as string,
      name: userName as string,
      avatar: userAvatar as string,
      roles: JSON.parse((userRoles as string) || "[]") as {
        name: string;
        key: string;
      }[],
    };
  }

  if (deviceId) {
    if (deviceStatus === "borrowing") {
      mode.value = "return";

      const initialItem: ReturnDeviceItem = {
        id: deviceId as string,
        status: deviceStatus as DeviceStatus,
        returnCondition: "healthy" as DeviceStatus,
        prevQuality: "healthy" as DeviceQuality,
      };

      const deviceDetail = await deviceService.getDeviceById(
        deviceId as string
      );
      devices.value.push({
        code: deviceKindId as string,
        name: deviceName as string,
        image: deviceImage as string,
        quantity: 1,
        unit: deviceUnit as string,
        expanded: true,
        items: [initialItem],
        isBorrowableLabOnly: deviceDetail?.isBorrowableLabOnly,
      });
    } else {
      mode.value = "borrow";

      const initialItem: ReturnDeviceItem = {
        id: deviceId as string,
        status: deviceStatus as DeviceStatus,
        returnCondition: "healthy" as DeviceStatus,
        prevQuality: "healthy" as DeviceQuality,
      };

      const deviceDetail = await deviceService.getDeviceById(
        deviceId as string
      );
      devices.value.push({
        code: deviceKindId as string,
        name: deviceName as string,
        image: deviceImage as string,
        quantity: 1,
        unit: deviceUnit as string,
        expanded: true,
        items: [initialItem],
        isBorrowableLabOnly: deviceDetail?.isBorrowableLabOnly,
      });
    }
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

    const deviceDetails = await deviceService.getDeviceById(deviceId);
    if (!deviceDetails || !deviceDetails.status) {
      toast({
        title: "Lỗi",
        description: "Không thể lấy thông tin thiết bị",
        variant: "destructive",
      });
      return;
    }

    if (mode.value === "idle") {
      if (
        deviceDetails.status === "healthy" ||
        deviceDetails.status === "broken"
      ) {
        mode.value = "borrow";
      } else if (deviceDetails.status === "borrowing") {
        mode.value = "return";
      } else {
        toast({
          title: "Thông báo",
          description: `Thiết bị đang ở trạng thái '${statusMap[deviceDetails.status]}', không thể mượn/trả.`,
        });
        return;
      }
    }

    if (
      mode.value === "borrow" &&
      deviceDetails.status !== "healthy" &&
      deviceDetails.status !== "broken"
    ) {
      toast({
        title: "Lỗi",
        description: `Thiết bị không khả dụng để mượn (ID: ${deviceId})`,
        variant: "destructive",
      });
      return;
    }

    if (mode.value === "return" && deviceDetails.status !== "borrowing") {
      toast({
        title: "Lỗi",
        description: `Thiết bị không ở trạng thái đang mượn (ID: ${deviceId})`,
        variant: "destructive",
      });
      return;
    }

    if (mode.value === "borrow") {
      const existingDevice = devices.value.find((d) => d.code === deviceKindId);
      if (existingDevice) {
        existingDevice.items.push({
          id: deviceId,
          status: deviceDetails.status,
          returnCondition: "healthy" as DeviceStatus,
          prevQuality: "healthy" as DeviceQuality,
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
              status: deviceDetails.status,
              returnCondition: "healthy" as DeviceStatus,
              prevQuality: "healthy" as DeviceQuality,
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
      const newItem: ReturnDeviceItem = {
        id: deviceId,
        status: deviceDetails.status,
        returnCondition: "healthy" as DeviceStatus,
        prevQuality: "healthy" as DeviceQuality,
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
    toast({
      title: "Lỗi",
      description: "Không thể xử lý thiết bị",
      variant: "destructive",
    });
  }
};

async function handleConfirmBorrow() {
  if (!userInfo.value?.id) {
    toast({
      title: "Lỗi",
      description: "Vui lòng chọn người mượn",
      variant: "destructive",
    });
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

  try {
    if (!devices.value || devices.value.length === 0) {
      toast({
        title: "Lỗi",
        description: "Vui lòng thêm thiết bị vào danh sách mượn",
        variant: "destructive",
      });
      return;
    }

    const stored = localStorage.getItem("user_info");
    if (!stored) {
      toast({
        title: "Lỗi",
        description: "Không tìm thấy thông tin phòng lab",
        variant: "destructive",
      });
      return;
    }

    const storedUserInfo = JSON.parse(stored);
    if (!storedUserInfo?.lab?.id) {
      toast({
        title: "Lỗi",
        description: "Người dùng không thuộc phòng lab nào",
        variant: "destructive",
      });
      return;
    }

    const deviceData = {
      items: devices.value.flatMap((device) =>
        device.items.map((item) => ({
          id: item.id,
          prevQuality: item.prevQuality,
          expectedReturnedAt: borrowDetails.value.returnDate,
        }))
      ),
    };

    const uniqueId = generateUniqueId();
    await receiptService.createReceipt({
      id: uniqueId,
      borrowerId: userInfo.value.id,
      borrowCheckerId: storedUserInfo.user_info.id,
      borrowedLabId: storedUserInfo.lab.id,
      devices: deviceData,
      expectedReturnAt: borrowDetails.value.returnDate,
      borrowDetails: {
        location: borrowDetails.value.location,
      },
    });
    toast({
      title: "Thành công",
      description: "Mượn thiết bị thành công",
      variant: "success",
    });
    resetForm();
  } catch (e) {
    toast({
      title: "Lỗi",
      description: (e as Error).message,
      variant: "destructive",
    });
  }
}

async function handleConfirmReturn() {
  if (!userInfo.value?.id) {
    toast({
      title: "Lỗi",
      description: "Vui lòng chọn người trả",
      variant: "destructive",
    });
    return;
  }

  try {
    const uniqueId = generateUniqueId();
    await receiptService.returnReceipt({
      id: uniqueId,
      returnerId: userInfo.value.id,
      devices: devices.value,
      notes: notes.value,
    });
    toast({
      title: "Thành công",
      description: "Trả thiết bị thành công",
      variant: "success",
    });
    resetForm();
  } catch (e) {
    toast({
      title: "Lỗi",
      description: (e as Error).message,
      variant: "destructive",
    });
  }
}

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

const toggleDevice = (device: Device) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded;
  }
};

const removeDeviceItem = (device: Device, itemId: string) => {
  device.items = device.items.filter((item) => item.id !== itemId);
  device.quantity = device.items.length;

  if (device.items.length === 0) {
    devices.value = devices.value.filter((d) => d.code !== device.code);
  }

  if (devices.value.length === 0) {
    mode.value = "idle";
  }
};

const resetForm = () => {
  mode.value = "idle";
  devices.value = [];
  notes.value = "";
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
    <p class="text-center text-gray-500 mb-6">
      Sử dụng máy scan quét mã QR thiết bị/người dùng để ghi nhận mượn trả
    </p>

    <div class="grid grid-cols-3 gap-6">
      <div
        class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <PackageCheck class="h-5 w-5" />
            {{ leftColumnTitle }}
          </h2>
        </div>

        <div>
          <div
            v-if="devices.length === 0"
            class="flex flex-col items-center justify-center py-20 text-center"
          >
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <Package class="h-8 w-8 text-gray-400" />
            </div>
            <p class="text-sm text-gray-500 max-w-xs">
              Quét mã QR thiết bị để ghi nhận
              <br />
              mượn trả thiết bị
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
                  <div class="flex items-center col-span-5 gap-3">
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
                  <div class="col-span-4 text-center">
                    <span class="text-base text-gray-900 font-medium">
                      SL: {{ device.quantity }} {{ device.unit }}
                    </span>
                  </div>
                  <div class="justify-self-end mr-2">
                    <ChevronDown
                      class="h-5 w-5 text-gray-400 transition-transform"
                      :class="{ 'rotate-180': device.expanded }"
                    />
                  </div>
                </div>
              </div>

              <div v-if="device.expanded" class="bg-gray-50">
                <div class="p-4">
                  <div class="grid grid-cols-10 items-center mb-2">
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
                      <div class="text-right">
                        <Button
                          variant="ghost"
                          size="icon"
                          @click.stop="removeDeviceItem(device, item.id)"
                          class="text-red-500 hover:text-red-600 hover:bg-red-100 rounded-full"
                        >
                          <Trash class="h-4 w-4" />
                        </Button>
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
                <div class="grid grid-cols-10 items-center">
                  <div class="col-span-6 flex items-center gap-3">
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
                  <span
                    class="col-span-3 text-base text-gray-900 font-medium mr-4"
                  >
                    SL: {{ device.quantity }} {{ device.unit }}
                  </span>
                  <ChevronDown
                    class="h-5 w-5 text-gray-400 transition-transform justify-self-end"
                    :class="{ 'rotate-180': device.expanded }"
                  />
                </div>
              </div>

              <div
                v-if="device.expanded && device.items.length > 0"
                class="bg-gray-50"
              >
                <div
                  class="grid grid-cols-10 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200"
                >
                  <div class="col-span-3">THIẾT BỊ GHI NHẬN</div>
                  <div class="col-span-3">TIẾN ĐỘ TRẢ</div>
                  <div class="col-span-3">TÌNH TRẠNG</div>
                  <div class="col-span-1"></div>
                </div>
                <div
                  v-for="item in device.items"
                  :key="item.id"
                  class="grid grid-cols-10 items-center px-4 py-3 border-b border-gray-100 last:border-b-0"
                >
                  <div class="col-span-3 text-sm font-medium text-gray-900">
                    {{ device.code }}/{{ item.id }}
                  </div>
                  <div class="col-span-3 text-sm text-gray-600">
                    Đúng hạn (còn 2 ngày)
                  </div>
                  <div class="col-span-3">
                    <div class="flex items-center gap-1">
                      <span
                        :class="qualityColorMap[item.prevQuality || 'healthy']"
                        class="text-base font-semibold w-fit text-right flex-shrink-0 bg-transparent"
                      >
                        {{ qualityMap[item.prevQuality || "healthy"] }}
                      </span>
                      <span class="text-gray-400 mx-1">→</span>
                      <Select v-model="item.returnCondition" class="flex-grow">
                        <SelectTrigger
                          class="h-9 text-sm bg-white text-base font-semibold w-fit"
                          :class="
                            item.returnCondition
                              ? statusColorMap[item.returnCondition]
                              : 'text-gray-900'
                          "
                        >
                          <SelectValue placeholder="Chọn tình trạng" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem
                            v-for="(label, status) in statusMap"
                            :key="status"
                            :value="status"
                          >
                            <span :class="statusColorMap[status]">{{
                              label
                            }}</span>
                          </SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                  <div class="col-span-1 flex justify-end">
                    <button
                      @click.stop="removeDeviceItem(device, item.id)"
                      class="text-gray-400 hover:text-red-500 transition-colors p-1 rounded-full hover:bg-gray-100"
                      aria-label="Remove device"
                    >
                      <Trash class="h-4 w-4" />
                    </button>
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
            <User class="h-5 w-5" />
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
                <User class="h-6 w-6 text-gray-400" />
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
            v-if="mode === 'borrow' && devices.length > 0"
            class="p-4 border-t border-gray-200"
          >
            <div class="flex items-center gap-3">
              <div class="rounded-full bg-blue-50 p-2">
                <Box class="h-4 w-4 text-blue-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Tổng thiết bị</p>
                <p class="font-medium text-gray-800 text-right mr-4">
                  {{ totalDevices }} cái
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-amber-50 p-2">
                <MapPin class="h-4 w-4 text-amber-600" />
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
                <Popover class="text-right">
                  <PopoverTrigger
                    as-child
                    class="font-medium text-gray-800 justify-self-end"
                  >
                    <Button
                      variant="outline"
                      :class="
                        cn(
                          'w-auto justify-end font-normal',
                          !borrowDetails.returnDate && 'text-muted-foreground'
                        )
                      "
                    >
                      <CalendarIcon class="h-4 w-4 opacity-50" />
                      <span class="text-base">{{
                        borrowDetails.returnDate &&
                        borrowDetails.returnDate instanceof Date
                          ? df.format(borrowDetails.returnDate)
                          : "Pick a date"
                      }}</span>
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent class="w-auto p-0">
                    <Calendar
                      v-model="calendarModel"
                      calendar-label="Date of birth"
                      initial-focus
                      :min-value="today(getLocalTimeZone())"
                    />
                  </PopoverContent>
                </Popover>
              </div>
            </div>

            <button
              :disabled="!userInfo || devices.length === 0"
              class="w-full mt-6 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="handleConfirmBorrow"
            >
              <PackageCheck class="h-5 w-5" />
              Xác nhận mượn
            </button>
          </div>

          <div
            v-if="mode === 'return' && devices.length > 0"
            class="p-4 border-t border-gray-200"
          >
            <div class="flex items-center gap-3">
              <div class="rounded-full bg-blue-50 p-2">
                <Box class="h-4 w-4 text-blue-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Tổng thiết bị</p>
                <p class="font-medium text-gray-800">{{ totalDevices }} cái</p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-amber-50 p-2">
                <MapPin class="h-4 w-4 text-amber-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Nơi trả</p>
                <p class="font-medium text-gray-800">
                  {{ returnDetails.location }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-indigo-50 p-2">
                <Calendar class="h-4 w-4 text-indigo-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Ngày trả</p>
                <p class="font-medium text-gray-800">
                  {{ returnDetails.actualReturnDate }}
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-red-50 p-2">
                <Calendar class="h-4 w-4 text-red-600" />
              </div>
              <div class="grid grid-cols-2 w-full">
                <p class="text-sm text-gray-500">Tiến độ trả</p>
                <p class="font-medium text-red-600">
                  {{ returnDetails.returnProgress }}
                </p>
              </div>
            </div>

            <div>
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

            <button
              :disabled="!userInfo || devices.length === 0"
              class="w-full mt-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
              @click="handleConfirmReturn"
            >
              <PackageCheck class="h-5 w-5" />
              Xác nhận trả
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
