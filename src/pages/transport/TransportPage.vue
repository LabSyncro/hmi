<script setup lang="ts">
import {
  BoxIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  LoaderIcon,
  MapPinIcon,
  PackageIcon,
  TrashIcon,
  TruckIcon,
  UserIcon,
} from "lucide-vue-next";

type ShipmentMode = "idle" | "inbound" | "outbound";

const mode = ref<ShipmentMode>("idle");
const userInfo = ref<UserInfo | null>(null);
const devices = ref<ShipmentDevice[]>([]);
const notes = ref<string>("");
const isLoadingDeviceScan = ref(false);
const isLoadingUser = ref(false);
const isConfirming = ref(false);
const labs = ref<{ id: string; name: string; room: string; branch: string }[]>(
  []
);
const selectedDestinationLab = ref<string | null>(null);
const checkAtDestination = ref(false);

const pendingDevices = ref<
  {
    id: string;
    status: (typeof DeviceStatus)[keyof typeof DeviceStatus];
    shipmentCondition: (typeof DeviceStatus)[keyof typeof DeviceStatus];
    prevCondition?: (typeof DeviceStatus)[keyof typeof DeviceStatus] | null;
    afterCondition?: (typeof DeviceStatus)[keyof typeof DeviceStatus] | null;
    shipmentId?: string | null;
    scanned?: boolean;
  }[]
>([]);

const storedUserInfo = ref<{
  id: string;
  lab: { id: string; room: string; branch: string };
} | null>(null);

const showSuccessModal = ref(false);
const successMessage = ref("");
const shipmentId = ref("");

const shipmentDetails = ref({
  sourceLocation: "",
  destinationLocation: "",
  transportDate: new Date().toLocaleDateString("vi-VN"),
  status: ShipmentStatus.SHIPPING,
  deviceCount: 0,
  repairStatus: {
    healthy: 0,
    broken: 0,
  },
});

const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0);
});

const leftColumnTitle = computed(() => {
  if (mode.value === "idle") return "DANH SÁCH GHI NHẬN";
  if (mode.value === "inbound") return "DANH SÁCH THIẾT BỊ CHUYỂN ĐI";
  return "DANH SÁCH THIẾT BỊ NHẬN VỀ";
});

const rightColumnTitle = computed(() => {
  if (mode.value === "idle") return "NGƯỜI VẬN CHUYỂN";
  if (mode.value === "inbound") return "NGƯỜI CHUYỂN ĐI";
  return "NGƯỜI NHẬN VỀ";
});

const router = useRouter();

onMounted(async () => {
  const stored = localStorage.getItem("user_info");
  if (stored) {
    const ui = JSON.parse(stored) as {
      id: string;
      lab: { id: string; room: string; branch: string };
    };
    shipmentDetails.value.sourceLocation = ui.lab
      ? `${ui.lab.room}, ${ui.lab.branch}`
      : "";
    storedUserInfo.value = ui;
  }

  try {
    const allLabs = await labService.getAllLabs();
    labs.value = allLabs;
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể tải danh sách phòng lab",
      variant: "destructive",
    });
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

    const deviceDetails = await deviceService.getDeviceShipmentById(
      deviceId,
      storedUserInfo.value?.lab.id
    );

    if (mode.value === "idle") {
      if (deviceDetails.status === DeviceStatus.SHIPPING) {
        mode.value = "outbound";
      } else if (
        deviceDetails.status === DeviceStatus.HEALTHY ||
        deviceDetails.status === DeviceStatus.BROKEN
      ) {
        mode.value = "inbound";
      } else {
        const status = deviceDetails.status
          ? statusMap[deviceDetails.status].toLowerCase()
          : "không xác định";
        toast({
          title: "Thông báo",
          description: `Thiết bị ${status}, không thể vận chuyển.`,
          variant: "destructive",
        });
        return;
      }
    }

    if (
      mode.value === "inbound" &&
      deviceDetails.status &&
      deviceDetails.status !== DeviceStatus.HEALTHY &&
      deviceDetails.status !== DeviceStatus.BROKEN
    ) {
      toast({
        title: "Lỗi",
        description: `Thiết bị ${statusMap[deviceDetails.status].toLowerCase()}, không thể chuyển đi. Chỉ có thể chuyển đi thiết bị đang ở trạng thái tốt hoặc hư hỏng.`,
        variant: "destructive",
      });
      return;
    }

    if (
      mode.value === "outbound" &&
      deviceDetails.status &&
      deviceDetails.status !== DeviceStatus.SHIPPING
    ) {
      toast({
        title: "Lỗi",
        description: `Thiết bị ${statusMap[deviceDetails.status].toLowerCase()}, không thể nhận về. Chỉ có thể nhận về thiết bị đang trong quá trình vận chuyển.`,
        variant: "destructive",
      });
      return;
    }

    if (userInfo.value === null) {
      pendingDevices.value.push({
        id: deviceId,
        status: deviceDetails.status || DeviceStatus.HEALTHY,
        shipmentCondition: DeviceStatus.HEALTHY,
        prevCondition: deviceDetails.prevCondition || null,
        afterCondition: deviceDetails.afterCondition || null,
        shipmentId: deviceDetails.shipmentId || null,
        scanned: true,
      });

      addDeviceToList(deviceId, deviceDetails, deviceKindId as string);

      toast({
        title: "Thiết bị đã được thêm",
        description: "Vui lòng quét mã người dùng để bắt đầu vận chuyển",
        variant: "success",
      });
    } else {
      handleDeviceForActiveMode(
        deviceId,
        deviceDetails,
        deviceKindId as string
      );
    }

    updateDeviceCount();
  } catch (error) {
    handleScanError(error);
  } finally {
    isLoadingDeviceScan.value = false;
  }
};

const handleDeviceForActiveMode = async (
  deviceId: string,
  deviceDetails: any,
  deviceKindId: string
) => {
  addDeviceToList(deviceId, deviceDetails, deviceKindId);

  if (mode.value === "outbound" && deviceDetails.shipmentId) {
    try {
      const shipment = await shipmentService.getShipmentById(
        deviceDetails.shipmentId
      );

      if (shipment && shipment.check_at_destination) {
        try {
          const shipmentDevicesResponse =
            await shipmentService.getShipmentDevices(deviceDetails.shipmentId);

          type ShipmentDeviceInfo = {
            shipment_id: string;
            device_id: string;
            prev_status:
              | (typeof DeviceStatus)[keyof typeof DeviceStatus]
              | null;
            after_status:
              | (typeof DeviceStatus)[keyof typeof DeviceStatus]
              | null;
            status: (typeof DeviceStatus)[keyof typeof DeviceStatus];
            kind_id: string;
            device_name: string;
            unit: string;
            is_borrowable_lab_only: boolean;
            main_image: string | null;
          };

          const shipmentDevices = shipmentDevicesResponse as any[];

          if (shipmentDevices.length === 0) {
            return;
          }

          const devicesByKind: Record<string, ShipmentDeviceInfo[]> = {};

          for (const device of shipmentDevices) {
            if (!device.kind_id) {
              continue;
            }

            const kindId = device.kind_id;
            if (!devicesByKind[kindId]) {
              devicesByKind[kindId] = [];
            }
            devicesByKind[kindId].push(device as ShipmentDeviceInfo);
          }

          for (const [kindId, devicesOfKind] of Object.entries(devicesByKind)) {
            const scannedDeviceIds = devices.value
              .filter((d) => d.code === kindId)
              .flatMap((d) => d.items)
              .map((item) => item.id);

            const unscannedDevices = devicesOfKind.filter(
              (d) => !scannedDeviceIds.includes(d.device_id)
            );

            unscannedDevices.forEach((unscannedDevice) => {
              if (
                !devices.value.some((d) =>
                  d.items.some((item) => item.id === unscannedDevice.device_id)
                )
              ) {
                const deviceInfo = {
                  id: unscannedDevice.device_id,
                  status: unscannedDevice.status,
                  shipmentId: deviceDetails.shipmentId,
                  deviceName: unscannedDevice.device_name,
                  image: { mainImage: unscannedDevice.main_image },
                  unit: unscannedDevice.unit,
                  isBorrowableLabOnly: unscannedDevice.is_borrowable_lab_only,
                  prevCondition: unscannedDevice.prev_status,
                };

                addUnscannedDevice(
                  unscannedDevice.device_id,
                  deviceInfo,
                  kindId
                );
              }
            });
          }
        } catch (deviceError) {
          toast({
            title: "Lỗi",
            description:
              "Không thể tải dữ liệu của tất cả thiết bị trong lô hàng",
            variant: "destructive",
          });
        }
      } else if (shipment) {
        toast({
          title: "Lỗi",
          description: "Lô hàng không được đặt cờ kiểm tra tại đích",
          variant: "destructive",
        });
      }
    } catch (shipmentError) {
      toast({
        title: "Lỗi",
        description: "Không thể tải dữ liệu lô hàng",
        variant: "destructive",
      });
    }
  }

  toast({
    title: "Thành công",
    description: "Đã thêm thiết bị vào danh sách vận chuyển",
    variant: "success",
  });
};

const addUnscannedDevice = (
  deviceId: string,
  deviceDetails: any,
  deviceKindId: string
) => {
  const newItem: ShipmentDeviceItem = {
    id: deviceId,
    status: deviceDetails.status || DeviceStatus.HEALTHY,
    shipmentCondition: DeviceStatus.HEALTHY,
    prevCondition: deviceDetails.prevCondition || null,
    afterCondition: deviceDetails.afterCondition || null,
    shipmentId: deviceDetails.shipmentId || null,
    scanned: false,
  };

  const existingDevice = devices.value.find((d) => d.code === deviceKindId);
  if (existingDevice) {
    existingDevice.items.push(newItem);
    existingDevice.quantity = existingDevice.items.length;
  } else {
    devices.value.push({
      code: deviceKindId,
      name: deviceDetails.deviceName,
      image: deviceDetails.image,
      quantity: 1,
      unit: deviceDetails.unit,
      expanded: true,
      isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly,
      items: [newItem],
      bulkUnscannedCondition: DeviceStatus.HEALTHY,
    });
  }
};

const addDeviceToList = (
  deviceId: string,
  deviceDetails: any,
  deviceKindId: string
) => {
  const newItem: ShipmentDeviceItem = {
    id: deviceId,
    status: deviceDetails.status || DeviceStatus.HEALTHY,
    shipmentCondition: DeviceStatus.HEALTHY,
    prevCondition: deviceDetails.prevCondition || null,
    afterCondition: deviceDetails.afterCondition || null,
    shipmentId: deviceDetails.shipmentId || null,
    scanned: true,
  };

  const existingDevice = devices.value.find((d) => d.code === deviceKindId);
  if (existingDevice) {
    existingDevice.items.push(newItem);
    existingDevice.quantity = existingDevice.items.length;
  } else {
    devices.value.push({
      code: deviceKindId,
      name: deviceDetails.deviceName,
      image: deviceDetails.image,
      quantity: 1,
      unit: deviceDetails.unit,
      expanded: true,
      isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly,
      items: [newItem],
      bulkUnscannedCondition: DeviceStatus.HEALTHY,
    });
  }
};

const handleScanError = (error: any) => {
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
    } else {
      toast({
        title: "Lỗi",
        description: "Không thể xử lý thiết bị",
        variant: "destructive",
      });
    }
  } else {
    toast({
      title: "Lỗi",
      description: "Không thể xử lý thiết bị",
      variant: "destructive",
    });
  }
};

const updateDeviceCount = () => {
  shipmentDetails.value.deviceCount = totalDevices.value;

  let healthy = 0;
  let broken = 0;

  devices.value.forEach((device) => {
    device.items.forEach((item) => {
      const condition = item.shipmentCondition;

      if (condition === DeviceStatus.HEALTHY) healthy++;
      else if (condition === DeviceStatus.BROKEN) broken++;
    });
  });

  shipmentDetails.value.repairStatus = {
    healthy,
    broken,
  };
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

const toggleDevice = (device: ShipmentDevice) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded;
  }
};

const removeDeviceItem = async (device: ShipmentDevice, itemId: string) => {
  device.items = device.items.filter((item) => item.id !== itemId);
  device.quantity = device.items.length;

  if (device.items.length === 0) {
    devices.value = devices.value.filter((d) => d.code !== device.code);
  }

  if (devices.value.length === 0) {
    mode.value = "idle";
  }

  updateDeviceCount();
};

const updateShipmentCondition = async (
  item: ShipmentDeviceItem,
  condition: (typeof DeviceStatus)[keyof typeof DeviceStatus]
) => {
  item.shipmentCondition = condition;

  const pendingDevice = pendingDevices.value.find((pd) => pd.id === item.id);
  if (pendingDevice) {
    pendingDevice.shipmentCondition = condition;
  }

  updateDeviceCount();
};

const completeShipment = async () => {
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

  if (mode.value === "inbound" && !selectedDestinationLab.value) {
    toast({
      title: "Lỗi",
      description: "Vui lòng chọn địa điểm đích",
      variant: "destructive",
    });
    return;
  }

  isConfirming.value = true;
  try {
    const deviceItems = devices.value.flatMap((device) =>
      device.items.map((item) => ({
        id: item.id,
        status: item.status,
        condition: item.shipmentCondition,
      }))
    );

    if (deviceItems.length === 0) {
      toast({
        title: "Lỗi",
        description: "Không có thiết bị nào được chọn để vận chuyển.",
        variant: "destructive",
      });
      isConfirming.value = false;
      return;
    }

    if (mode.value === "inbound") {
      const response = await shipmentService.confirmInboundShipment({
        technicianId: userInfo.value.id,
        sourceLabId: storedUserInfo.value?.lab.id || "",
        destinationLabId: selectedDestinationLab.value || "",
        notes: notes.value || undefined,
        devices: deviceItems.map((d) => ({
          id: d.id,
          inboundCondition: d.condition,
        })),
        checkAtDestination: checkAtDestination.value,
      });
      shipmentId.value = response?.id || "";
      successMessage.value = "Ghi nhận chuyển đi thành công!";
    } else {
      let currentShipmentId = "";

      if (devices.value.length > 0 && devices.value[0].items.length > 0) {
        const firstItem = devices.value[0].items[0];
        currentShipmentId = firstItem.shipmentId || "";
      }

      if (!currentShipmentId) {
        toast({
          title: "Lỗi",
          description: "Không tìm thấy mã vận chuyển",
          variant: "destructive",
        });
        isConfirming.value = false;
        return;
      }

      const response = await shipmentService.confirmOutboundShipment({
        technicianId: userInfo.value.id,
        shipmentId: currentShipmentId,
        notes: notes.value || undefined,
        devices: deviceItems.map((d) => ({
          id: d.id,
          outboundCondition: d.condition,
        })),
      });
      shipmentId.value = response?.id || "";
      successMessage.value = "Ghi nhận nhận về thành công!";
    }

    showSuccessModal.value = true;
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể lưu dữ liệu vận chuyển",
      variant: "destructive",
    });
  } finally {
    isConfirming.value = false;
  }
};

const goToHome = () => {
  showSuccessModal.value = false;
  mode.value = "idle";
  devices.value = [];
  notes.value = "";
  userInfo.value = null;
  router.push("/transport");
};

const handleOneTimeQRScan = async (input: string) => {
  isLoadingUser.value = true;
  try {
    const { verifyScannedQrCode } = useOneTimeQR();
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
      title: "Lỗi",
      description: "Lỗi xử lý mã QR",
      variant: "destructive",
    });
  } finally {
    isLoadingUser.value = false;
  }
};

const getStatusClass = (item: ShipmentDeviceItem) => {
  if (
    mode.value === "outbound" &&
    item.prevCondition &&
    statusColorMap[item.prevCondition]
  ) {
    return statusColorMap[item.prevCondition];
  }
  return statusColorMap[item.status];
};

const getStatusText = (item: ShipmentDeviceItem) => {
  if (
    mode.value === "outbound" &&
    item.prevCondition &&
    statusMap[item.prevCondition]
  ) {
    return statusMap[item.prevCondition];
  }
  return statusMap[item.status];
};

const getUnscannedCount = (device: ShipmentDevice) => {
  return device.items.filter((item) => !item.scanned).length;
};

const getUnscannedItems = (device: ShipmentDevice) => {
  return device.items.filter((item) => !item.scanned);
};

const updateAllUnscannedDevices = (device: ShipmentDevice) => {
  if (!device.bulkUnscannedCondition) return;

  const unscannedItems = getUnscannedItems(device);
  unscannedItems.forEach((item) => {
    if (device.bulkUnscannedCondition) {
      item.shipmentCondition = device.bulkUnscannedCondition;
      updateShipmentCondition(item, device.bulkUnscannedCondition);
    }
  });

  updateDeviceCount();
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
    <div class="grid grid-cols-3 gap-6">
      <div
        class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
      >
        <div class="p-4 border-b border-gray-200">
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <TruckIcon class="h-5 w-5" />
            {{ leftColumnTitle }}
          </h2>
        </div>

        <div class="h-[calc(100vh-10rem)] overflow-y-auto">
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
            <h3 class="text-lg font-medium mb-1">CHƯA GHI NHẬN</h3>
            <p class="text-sm text-gray-500 max-w-xs">
              {{
                isLoadingDeviceScan
                  ? "Đang xử lý thiết bị..."
                  : "Quét mã QR thiết bị để ghi nhận vận chuyển"
              }}
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
                <div
                  class="grid grid-cols-10 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200"
                >
                  <div class="col-span-1"></div>
                  <div class="col-span-6">THIẾT BỊ GHI NHẬN</div>
                  <div class="col-span-3">TÌNH TRẠNG</div>
                </div>

                <div
                  v-for="item in device.items"
                  :key="item.id"
                  class="grid grid-cols-10 items-center px-4 py-3 border-b border-gray-100 last:border-b-0"
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
                  <div class="col-span-3 flex items-center justify-start gap-2">
                    <Badge
                      :class="getStatusClass(item)"
                      variant="outline"
                      class="h-8 text-sm font-semibold w-fit whitespace-nowrap"
                    >
                      {{ getStatusText(item) }}
                    </Badge>
                    <span class="text-gray-400">→</span>
                    <div class="w-32">
                      <Select
                        v-model="item.shipmentCondition"
                        class="flex-grow"
                        @update:modelValue="
                          updateShipmentCondition(item, item.shipmentCondition)
                        "
                      >
                        <SelectTrigger
                          class="h-8 text-sm bg-white font-semibold w-fit"
                          :class="
                            item.shipmentCondition
                              ? statusColorMap[item.shipmentCondition]
                              : 'text-gray-900'
                          "
                        >
                          <SelectValue placeholder="Chọn tình trạng" />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="healthy" class="cursor-pointer">
                            <Badge
                              :class="statusColorMap['healthy']"
                              variant="outline"
                            >
                              Tốt
                            </Badge>
                          </SelectItem>
                          <SelectItem value="broken" class="cursor-pointer">
                            <Badge
                              :class="statusColorMap['broken']"
                              variant="outline"
                            >
                              Hư hỏng
                            </Badge>
                          </SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </div>
                </div>

                <div
                  v-if="mode === 'outbound' && getUnscannedCount(device) > 0"
                >
                  <div
                    class="grid grid-cols-10 items-center px-4 py-2 text-sm font-medium text-gray-500 border-y border-gray-200 mt-1"
                  >
                    <span class="col-span-1"></span>
                    <span class="col-span-6">THIẾT BỊ CHƯA GHI NHẬN</span>
                    <span class="col-span-3">TÌNH TRẠNG</span>
                  </div>

                  <div class="bg-white px-4 py-3 border-b border-gray-100">
                    <div class="grid grid-cols-10 items-center">
                      <span class="col-span-1"></span>
                      <div class="col-span-6">
                        <span class="text-sm font-medium text-gray-900">
                          {{ getUnscannedCount(device) }} {{ device.unit }}
                        </span>
                      </div>
                      <div class="col-span-3">
                        <Select
                          v-model="device.bulkUnscannedCondition"
                          @update:modelValue="updateAllUnscannedDevices(device)"
                          class="flex-grow"
                        >
                          <SelectTrigger
                            class="h-8 text-sm bg-white font-semibold w-fit"
                            :class="
                              device.bulkUnscannedCondition
                                ? statusColorMap[device.bulkUnscannedCondition]
                                : 'text-gray-900'
                            "
                          >
                            <SelectValue placeholder="Chọn tình trạng" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="healthy" class="cursor-pointer">
                              <Badge
                                :class="statusColorMap['healthy']"
                                variant="outline"
                              >
                                Tốt
                              </Badge>
                            </SelectItem>
                            <SelectItem value="broken" class="cursor-pointer">
                              <Badge
                                :class="statusColorMap['broken']"
                                variant="outline"
                              >
                                Hư hỏng
                              </Badge>
                            </SelectItem>
                            <SelectItem value="lost" class="cursor-pointer">
                              <Badge
                                :class="statusColorMap['lost']"
                                variant="outline"
                              >
                                Thất lạc
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
            v-if="
              (mode === 'inbound' || mode === 'outbound') && devices.length > 0
            "
            class="p-4 border-t border-gray-200"
          >
            <div
              v-if="mode === 'inbound'"
              class="flex items-center justify-between mb-3 py-1"
            >
              <div class="flex items-center gap-1">
                <span class="text-sm font-medium">Kiểm tại nơi nhận</span>
              </div>
              <Switch
                v-model="checkAtDestination"
                class="data-[state=checked]:bg-teal-500"
              />
            </div>

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
                  <span class="text-green-600 text-sm bg-white font-semibold">
                    Tốt
                  </span>
                  {{ shipmentDetails.repairStatus.healthy }}
                </div>
                <div class="flex justify-between">
                  <span class="text-red-600 text-sm bg-white font-semibold">
                    Hư hỏng
                  </span>
                  {{ shipmentDetails.repairStatus.broken }}
                </div>
              </div>
            </div>

            <div v-if="mode === 'outbound'" class="flex items-center gap-3">
              <div class="rounded-full bg-amber-50 p-2">
                <MapPinIcon class="h-4 w-4 text-amber-600" />
              </div>
              <div class="flex justify-between w-full">
                <p class="text-sm text-gray-500">Địa điểm gốc</p>
                <p class="font-medium text-gray-800 text-right">
                  {{ shipmentDetails.sourceLocation }}
                </p>
              </div>
            </div>

            <div v-if="mode === 'inbound'" class="flex items-center gap-3">
              <div class="rounded-full bg-purple-50 p-2">
                <MapPinIcon class="h-4 w-4 text-purple-600" />
              </div>
              <div class="flex justify-between items-center w-full">
                <p class="text-sm text-gray-500">Địa điểm đích</p>
                <div class="w-fit">
                  <Select
                    v-model="selectedDestinationLab"
                    @update:modelValue="
                      (value) => {
                        if (typeof value === 'string') {
                          const selectedLab = labs.find(
                            (lab) => lab.id === value
                          );
                          if (selectedLab) {
                            shipmentDetails.destinationLocation = `${selectedLab.room}, ${selectedLab.branch}`;
                          }
                        }
                      }
                    "
                  >
                    <SelectTrigger class="h-9 text-sm bg-white select-trigger">
                      <TruckIcon class="h-6 w-6 mr-2 text-gray-500" />
                      <SelectValue placeholder="Chọn lab" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem
                        v-for="lab in labs"
                        :key="lab.id"
                        :value="lab.id"
                        class="cursor-pointer"
                      >
                        {{ lab.room }}, {{ lab.branch }}
                      </SelectItem>
                    </SelectContent>
                  </Select>
                </div>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <div class="rounded-full bg-green-50 p-2">
                <CalendarIcon class="h-4 w-4 text-green-600" />
              </div>
              <div class="flex justify-between w-full">
                <p v-if="mode === 'inbound'" class="text-sm text-gray-500">
                  Ngày chuyển
                </p>
                <p v-if="mode === 'outbound'" class="text-sm text-gray-500">
                  Ngày nhận
                </p>
                <p class="font-medium text-gray-800">
                  {{ shipmentDetails.transportDate }}
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
                placeholder="Thêm ghi chú về lý do vận chuyển hoặc các yêu cầu đặc biệt..."
                class="min-h-[80px]"
              />
            </div>

            <Button
              :disabled="
                isConfirming ||
                !userInfo ||
                devices.length === 0 ||
                (mode === 'inbound' && !selectedDestinationLab)
              "
              class="w-full mt-4 bg-blue-600 hover:bg-blue-700"
              @click="completeShipment"
            >
              <LoaderIcon
                v-if="isConfirming"
                class="h-5 w-5 mr-2 animate-spin"
              />
              <TruckIcon v-else class="h-5 w-5 mr-2" />
              {{
                mode === "inbound" ? "Xác nhận chuyển đi" : "Xác nhận nhận về"
              }}
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
          <p class="text-base text-gray-600 mb-6">
            Mã vận chuyển: {{ shipmentId }}
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

<style scoped>
:deep(.select-trigger svg:last-child) {
  display: none;
}
</style>
