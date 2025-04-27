<script setup lang="ts">
import {
  BoxIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  ListIcon,
  LoaderIcon,
  MapPinIcon,
  PackageIcon,
  TrashIcon,
  UserIcon,
  WrenchIcon,
  XIcon,
} from "lucide-vue-next";

interface PendingDevice {
  id: string;
  status: (typeof DeviceStatus)[keyof typeof DeviceStatus];
  maintenanceOutcome: (typeof DeviceStatus)[keyof typeof DeviceStatus];
}

const mode = ref<"idle" | "maintenance">("idle");
const pendingDevices = ref<PendingDevice[]>([]);

const userInfo = ref<UserInfo | null>(null);
const storedUserInfo = ref<{
  id: string;
  lab: { id: string; room: string; branch: string };
} | null>(null);

const validStatuses = [
  DeviceStatus.HEALTHY,
  DeviceStatus.BROKEN,
  DeviceStatus.ASSESSING,
  DeviceStatus.DISCARDED,
];

const devices = ref<MaintenanceDevice[]>([]);

const notes = ref<string>("");

const maintenanceDetails = ref({
  location: "",
  maintenanceDate: new Date().toLocaleDateString("vi-VN"),
  repairStatus: {
    fixed: 0,
    partiallyFixed: 0,
    needsReplacement: 0,
    unrepairable: 0,
  },
});

const router = useRouter();

const isLoadingDeviceScan = ref(false);
const isLoadingUser = ref(false);
const isConfirming = ref(false);
const showSuccessModal = ref(false);
const successMessage = ref("");
const maintenanceId = ref("");

const showMaintenanceSessionsModal = ref(false);
const maintenanceSessions = ref<MaintenanceSession[]>([]);
const selectedSession = ref<MaintenanceSession | null>(null);
const loadingMaintenanceSessions = ref(false);
const continueMode = ref(false);

const totalDevices = computed(() => {
  return devices.value.reduce((total, device) => total + device.quantity, 0);
});

const MAINTENANCE_MESSAGES = {
  SCAN_DEVICE_ERROR: "Không thể trích xuất ID thiết bị từ mã QR",
  DEVICE_ALREADY_ADDED: "Thiết bị này đã được thêm vào danh sách.",
  DEVICE_INFO_ERROR: "Không thể lấy thông tin thiết bị",
  DEVICE_ADDED_SUCCESS: "Đã thêm thiết bị vào danh sách sửa chữa",
  DEVICE_PROCESS_ERROR: "Không thể xử lý thiết bị",
  DEVICE_NOT_FOUND: "Không tìm thấy thiết bị",
  DEVICE_WRONG_LAB: "Thiết bị không thuộc phòng lab này",
  MISSING_INFO: "Thiếu thông tin thiết bị hoặc phòng lab",
  USER_NOT_FOUND: "Không tìm thấy người dùng",
  USER_INFO_ERROR: "Không thể tìm thấy thông tin người dùng",
  USER_SCAN_REQUIRED: "Vui lòng quét mã người dùng",
  DEVICE_SCAN_REQUIRED: "Vui lòng quét ít nhất một thiết bị",
  MAINTENANCE_SUCCESS: "Ghi nhận sửa chữa thành công",
  MAINTENANCE_ERROR: "Không thể lưu dữ liệu sửa chữa",
  MAINTENANCE_LIST_ERROR: "Không thể tải danh sách phiên sửa chữa",
  MAINTENANCE_CONTINUE_ERROR: "Không thể tiếp tục phiên sửa chữa này",
  LAB_ID_ERROR: "Lab ID not found",
  QR_PROCESSING_ERROR: "Lỗi xử lý mã QR",
} as const;

const PAGE_TITLES = {
  MAIN: "GHI NHẬN SỬA CHỮA",
  LEFT_COLUMN: "DANH SÁCH GHI NHẬN",
  RIGHT_COLUMN: "NGƯỜI SỬA CHỮA",
  NO_RECORDS: "CHƯA GHI NHẬN",
} as const;

const PROCESSING_MESSAGES = {
  DEVICE: "Đang xử lý thiết bị...",
  USER: "Đang xử lý người dùng...",
  SCAN_DEVICE: "Quét mã QR thiết bị để ghi nhận sửa chữa",
  SCAN_USER: "Quét QR định danh người dùng",
} as const;

const pageTitle = computed(() => PAGE_TITLES.MAIN);
const leftColumnTitle = computed(() => PAGE_TITLES.LEFT_COLUMN);
const rightColumnTitle = computed(() => PAGE_TITLES.RIGHT_COLUMN);

const goToHome = () => {
  showSuccessModal.value = false;
  mode.value = "idle";
  devices.value = [];
  notes.value = "";
  userInfo.value = null;
  router.push("/maintenance");
};

onMounted(async () => {
  const stored = localStorage.getItem("user_info");
  if (stored) {
    const ui = JSON.parse(stored) as {
      id: string;
      lab: { id: string; room: string; branch: string };
    };
    maintenanceDetails.value.location = ui.lab
      ? `${ui.lab.room}, ${ui.lab.branch}`
      : "";
    storedUserInfo.value = ui;
  }
});

async function handleUserCodeChange(userId: string) {
  const isValidUserCode = /^\d{7}$/.test(userId);

  if (!isValidUserCode) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.USER_INFO_ERROR,
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
        description: MAINTENANCE_MESSAGES.USER_NOT_FOUND,
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

    if (pendingDevices.value.length > 0 && storedUserInfo.value?.lab?.id) {
      try {
        const maintenanceDevices = pendingDevices.value.map((pd) => ({
          id: pd.id,
          maintenanceOutcome: pd.maintenanceOutcome,
          prevStatus: pd.status,
        }));

        const result = await maintenanceService.createMaintenance({
          technicianId: userMeta.id,
          location: storedUserInfo.value.lab.id,
          notes: notes.value || undefined,
          devices: maintenanceDevices,
        });

        maintenanceId.value = result.id;
        mode.value = "maintenance";
        pendingDevices.value = [];

        toast({
          title: "Thành công",
          description: `Đã nhận diện: ${userMeta.name} và bắt đầu sửa chữa`,
          variant: "success",
        });
      } catch (error) {
        toast({
          title: "Lỗi",
          description: MAINTENANCE_MESSAGES.MAINTENANCE_ERROR,
          variant: "destructive",
        });
      }
    } else {
      toast({
        title: "Thành công",
        description: `Đã nhận diện: ${userMeta.name}`,
        variant: "success",
      });
    }
  } catch (error) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.USER_INFO_ERROR,
      variant: "destructive",
    });
    userInfo.value = null;
  } finally {
    isConfirming.value = false;
    isLoadingUser.value = false;
  }
}

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

      if (pendingDevices.value.length > 0 && storedUserInfo.value?.lab?.id) {
        try {
          const maintenanceDevices = pendingDevices.value.map((pd) => ({
            id: pd.id,
            maintenanceOutcome: pd.maintenanceOutcome,
            prevStatus: pd.status,
          }));

          const result = await maintenanceService.createMaintenance({
            technicianId: user.id,
            location: storedUserInfo.value.lab.id,
            notes: notes.value || undefined,
            devices: maintenanceDevices,
          });

          maintenanceId.value = result.id;
          mode.value = "maintenance";
          pendingDevices.value = [];

          toast({
            title: "Thành công",
            description: `Đã nhận diện: ${user.name} và bắt đầu sửa chữa`,
            variant: "success",
          });
        } catch (error) {
          toast({
            title: "Lỗi",
            description: MAINTENANCE_MESSAGES.MAINTENANCE_ERROR,
            variant: "destructive",
          });
          return;
        }
      } else {
        toast({
          title: "Thành công",
          description: `Đã nhận diện: ${user.name}`,
          variant: "success",
        });
      }
    }
  } catch (error) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.QR_PROCESSING_ERROR,
      variant: "destructive",
    });
  } finally {
    isLoadingUser.value = false;
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
        description: MAINTENANCE_MESSAGES.SCAN_DEVICE_ERROR,
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
        description: MAINTENANCE_MESSAGES.DEVICE_ALREADY_ADDED,
        variant: "destructive",
      });
      return;
    }

    const deviceDetails = await deviceService.getDeviceMaintenanceById(
      deviceId,
      storedUserInfo.value?.lab.id || ""
    );

    if (deviceDetails.currentStatus === DeviceStatus.MAINTAINING) {
      toast({
        title: "Lỗi",
        description: "Thiết bị đang trong quá trình sửa chữa khác",
        variant: "destructive",
      });
      return;
    }

    if (!validStatuses.includes(deviceDetails.status)) {
      console.log(deviceDetails);
      toast({
        title: "Lỗi",
        description: `Tình trạng thiết bị không hợp lệ: ${statusMap[deviceDetails.status]}`,
        variant: "destructive",
      });
      return;
    }

    if (mode.value === "idle") {
      if (userInfo.value && storedUserInfo.value?.lab?.id) {
        try {
          const maintenanceDevices = [
            ...pendingDevices.value.map((pd) => ({
              id: pd.id,
              maintenanceOutcome: pd.maintenanceOutcome,
              prevStatus: pd.status,
            })),
            {
              id: deviceId,
              maintenanceOutcome: deviceDetails.outcome || DeviceStatus.HEALTHY,
              prevStatus: deviceDetails.status,
            },
          ];

          const result = await maintenanceService.createMaintenance({
            technicianId: userInfo.value.id,
            location: storedUserInfo.value.lab.id,
            notes: notes.value || undefined,
            devices: maintenanceDevices,
          });

          maintenanceId.value = result.id;
          mode.value = "maintenance";
          pendingDevices.value = [];
        } catch (error) {
          toast({
            title: "Lỗi",
            description: MAINTENANCE_MESSAGES.MAINTENANCE_ERROR,
            variant: "destructive",
          });
          return;
        }
      } else {
        pendingDevices.value.push({
          id: deviceId,
          status: deviceDetails.status,
          maintenanceOutcome: deviceDetails.outcome || DeviceStatus.HEALTHY,
        });

        const existingDevice = devices.value.find(
          (d) => d.code === deviceKindId
        );
        if (existingDevice) {
          existingDevice.items.push({
            id: deviceId,
            status: deviceDetails.status,
            maintenanceOutcome: deviceDetails.outcome || DeviceStatus.HEALTHY,
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
                maintenanceOutcome:
                  deviceDetails.outcome || DeviceStatus.HEALTHY,
              },
            ],
            isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly,
          });
        }

        updateMaintenanceCounts();

        toast({
          title: "Thiết bị đã được thêm",
          description: "Vui lòng quét mã người dùng để bắt đầu sửa chữa",
          variant: "success",
        });
        return;
      }
    } else if (mode.value === "maintenance") {
      try {
        if (continueMode.value && maintenanceId.value) {
          await maintenanceService.addDeviceToMaintenance(
            maintenanceId.value,
            deviceId,
            deviceDetails.status,
            deviceDetails.outcome
          );
        }
      } catch (error) {
        toast({
          title: "Lỗi",
          description:
            "Không thể cập nhật thông tin thiết bị vào phiên sửa chữa",
          variant: "destructive",
        });
        return;
      }
    }

    const newItem: MaintenanceDeviceItem = {
      id: deviceId,
      status: deviceDetails.status,
      maintenanceOutcome: deviceDetails.outcome || DeviceStatus.HEALTHY,
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

    updateMaintenanceCounts();

    toast({
      title: "Thành công",
      description: MAINTENANCE_MESSAGES.DEVICE_ADDED_SUCCESS,
      variant: "success",
    });
  } catch (error) {
    if (error instanceof Error) {
      if (error.message === "Device not found") {
        toast({
          title: "Lỗi",
          description: MAINTENANCE_MESSAGES.DEVICE_NOT_FOUND,
          variant: "destructive",
        });
      } else if (error.message === "Device does not belong to this lab") {
        toast({
          title: "Lỗi",
          description: MAINTENANCE_MESSAGES.DEVICE_WRONG_LAB,
          variant: "destructive",
        });
      } else if (error.message === "Missing device ID") {
        toast({
          title: "Lỗi",
          description: MAINTENANCE_MESSAGES.MISSING_INFO,
          variant: "destructive",
        });
      } else {
        toast({
          title: "Lỗi",
          description: MAINTENANCE_MESSAGES.DEVICE_PROCESS_ERROR,
          variant: "destructive",
        });
      }
    } else {
      toast({
        title: "Lỗi",
        description: MAINTENANCE_MESSAGES.DEVICE_PROCESS_ERROR,
        variant: "destructive",
      });
    }
  } finally {
    isLoadingDeviceScan.value = false;
  }
};

const updateMaintenanceCounts = () => {
  let fixed = 0;
  let partiallyFixed = 0;
  let needsReplacement = 0;
  let unrepairable = 0;

  devices.value.forEach((device) => {
    device.items.forEach((item) => {
      const outcome = item.maintenanceOutcome;

      if (outcome === "healthy") fixed++;
      else if (outcome === "broken") partiallyFixed++;
      else if (outcome === "assessing") needsReplacement++;
      else if (outcome === "discarded") unrepairable++;
    });
  });

  maintenanceDetails.value.repairStatus = {
    fixed,
    partiallyFixed,
    needsReplacement,
    unrepairable,
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

const toggleDevice = (device: MaintenanceDevice) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded;
  }
};

const removeDeviceItem = async (device: MaintenanceDevice, itemId: string) => {
  try {
    device.items = device.items.filter((item) => item.id !== itemId);
    device.quantity = device.items.length;

    if (device.items.length === 0) {
      devices.value = devices.value.filter((d) => d.code !== device.code);
    }

    if (mode.value === "idle") {
      pendingDevices.value = pendingDevices.value.filter(
        (pd) => pd.id !== itemId
      );
    } else if (mode.value === "maintenance" && maintenanceId.value) {
      await maintenanceService.removeDeviceFromMaintenance(
        maintenanceId.value,
        itemId
      );
    }

    if (devices.value.length === 0 && pendingDevices.value.length === 0) {
      mode.value = "idle";
    }

    updateMaintenanceCounts();
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể xóa thiết bị khỏi phiên sửa chữa",
      variant: "destructive",
    });
  }
};

const updateMaintenanceOutcome = async (
  item: MaintenanceDeviceItem,
  outcome: (typeof DeviceStatus)[keyof typeof DeviceStatus]
) => {
  try {
    item.maintenanceOutcome = outcome;

    if (mode.value === "idle") {
      const pendingDevice = pendingDevices.value.find(
        (pd) => pd.id === item.id
      );
      if (pendingDevice) {
        pendingDevice.maintenanceOutcome = outcome;
      }
    } else if (mode.value === "maintenance" && maintenanceId.value) {
      await maintenanceService.updateDeviceCondition(
        maintenanceId.value,
        item.id,
        outcome
      );
    }

    updateMaintenanceCounts();
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể cập nhật tình trạng thiết bị",
      variant: "destructive",
    });
  }
};

const completeMaintenance = async () => {
  if (!userInfo.value) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.USER_SCAN_REQUIRED,
      variant: "destructive",
    });
    return;
  }

  if (devices.value.length === 0) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.DEVICE_SCAN_REQUIRED,
      variant: "destructive",
    });
    return;
  }

  isConfirming.value = true;
  try {
    const maintenanceToComplete =
      continueMode.value && maintenanceId.value
        ? maintenanceId.value
        : maintenanceId.value;

    if (maintenanceToComplete) {
      const deviceUpdates = devices.value.flatMap((device) =>
        device.items.map((item) => ({
          id: item.id,
          condition: item.maintenanceOutcome,
        }))
      );

      if (deviceUpdates.length > 0) {
        await maintenanceService.updateListDeviceConditions(
          maintenanceToComplete,
          deviceUpdates
        );
      }

      await maintenanceService.completeMaintenance(
        maintenanceToComplete,
        notes.value
      );
    }

    successMessage.value = MAINTENANCE_MESSAGES.MAINTENANCE_SUCCESS;
    showSuccessModal.value = true;
    continueMode.value = false;
  } catch (error) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.MAINTENANCE_ERROR,
      variant: "destructive",
    });
  } finally {
    isConfirming.value = false;
  }
};

async function fetchMaintenanceSessions() {
  loadingMaintenanceSessions.value = true;
  try {
    if (!storedUserInfo.value?.lab?.id) {
      toast({
        title: "Lỗi",
        description: MAINTENANCE_MESSAGES.LAB_ID_ERROR,
        variant: "destructive",
      });
      return;
    }
    const result = await maintenanceService.getIncompleteMaintenance(
      storedUserInfo.value.lab.id
    );
    maintenanceSessions.value = result;
  } catch (error) {
    toast({
      title: "Lỗi",
      description: MAINTENANCE_MESSAGES.MAINTENANCE_LIST_ERROR,
      variant: "destructive",
    });
  } finally {
    loadingMaintenanceSessions.value = false;
  }
}

function openMaintenanceSessionsModal() {
  fetchMaintenanceSessions();
  showMaintenanceSessionsModal.value = true;
}

function clearMaintenanceState() {
  devices.value = [];
  userInfo.value = null;
  mode.value = "idle";
  continueMode.value = false;
  pendingDevices.value = [];
  notes.value = "";
  maintenanceId.value = "";
  selectedSession.value = null;
}

async function cancelMaintenance(maintenanceId: string) {
  try {
    await maintenanceService.cancelMaintenance(maintenanceId);

    clearMaintenanceState();

    toast({
      title: "Thành công",
      description: "Đã hủy phiên sửa chữa",
      variant: "success",
    });

    showMaintenanceSessionsModal.value = false;
    await fetchMaintenanceSessions();
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể hủy phiên sửa chữa",
      variant: "destructive",
    });
  }
}

async function continueMaintenance() {
  if (!selectedSession.value) {
    return;
  }
  loadingMaintenanceSessions.value = true;
  try {
    const userMeta = await userService.getUserById(
      selectedSession.value.maintainerId
    );
    if (!userMeta) {
      toast({
        title: "Lỗi",
        description: MAINTENANCE_MESSAGES.USER_NOT_FOUND,
        variant: "destructive",
      });
      loadingMaintenanceSessions.value = false;
      return;
    }

    userInfo.value = {
      id: userMeta.id,
      name: userMeta.name,
      avatar: userMeta.avatar,
      roles: userMeta.roles,
    };

    maintenanceDetails.value.location = `${selectedSession.value.labRoom}, ${selectedSession.value.labBranch}`;

    mode.value = "maintenance";
    continueMode.value = true;
    maintenanceId.value = selectedSession.value.id;

    showMaintenanceSessionsModal.value = false;

    toast({
      title: "Thành công",
      description: `Tiếp tục phiên sửa chữa: ${selectedSession.value.id}`,
      variant: "success",
    });

    const deviceIdList = Array.isArray(selectedSession.value.deviceIds)
      ? [...selectedSession.value.deviceIds].map((id) => String(id))
      : [];

    if (deviceIdList.length > 0) {
      await loadMaintenanceDevices(deviceIdList);
    } else {
      toast({
        title: "Cảnh báo",
        description: "Không tìm thấy thiết bị nào trong phiên sửa chữa này",
        variant: "destructive",
      });
    }
  } catch (error) {
    if (error instanceof Error) {
      toast({
        title: "Lỗi",
        description: MAINTENANCE_MESSAGES.MAINTENANCE_CONTINUE_ERROR,
        variant: "destructive",
      });
    } else {
      toast({
        title: "Lỗi",
        description: MAINTENANCE_MESSAGES.MAINTENANCE_CONTINUE_ERROR,
        variant: "destructive",
      });
    }
  } finally {
    loadingMaintenanceSessions.value = false;
  }
}

async function loadMaintenanceDevices(deviceIds: string[]) {
  isLoadingDeviceScan.value = true;
  try {
    devices.value = [];

    for (const deviceId of deviceIds) {
      try {
        const deviceDetail =
          await deviceService.getDeviceMaintenanceById(deviceId);

        const newItem: MaintenanceDeviceItem = {
          id: deviceId,
          status: deviceDetail.status || DeviceStatus.HEALTHY,
          maintenanceOutcome: deviceDetail.outcome || DeviceStatus.HEALTHY,
        };

        const existingDevice = devices.value.find(
          (d) => d.code === deviceDetail.kind
        );
        if (existingDevice) {
          existingDevice.items.push(newItem);
          existingDevice.quantity = existingDevice.items.length;
        } else {
          devices.value.push({
            code: deviceDetail.kind,
            name: deviceDetail.deviceName,
            image: deviceDetail.image,
            quantity: 1,
            unit: deviceDetail.unit,
            expanded: true,
            items: [newItem],
            isBorrowableLabOnly: deviceDetail.isBorrowableLabOnly,
          });
        }
      } catch (deviceError) {
        toast({
          title: "Lỗi",
          description: `Không thể tải thiết bị ${deviceId}: ${deviceError}`,
          variant: "destructive",
        });
      }
    }

    updateMaintenanceCounts();

    if (devices.value.length === 0) {
      toast({
        title: "Thông báo",
        description: "Không có thiết bị nào được hiển thị",
        variant: "destructive",
      });
    }
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể tải thông tin thiết bị cho phiên sửa chữa này",
      variant: "destructive",
    });
  } finally {
    isLoadingDeviceScan.value = false;
  }
}

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
      Sử dụng máy scan quét mã QR thiết bị/người dùng để ghi nhận sửa chữa
    </p>

    <div class="grid grid-cols-3 gap-6">
      <div
        class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
      >
        <div
          class="p-4 border-b border-gray-200 flex justify-between items-center"
        >
          <h2 class="text-lg font-semibold flex items-center gap-2">
            <WrenchIcon class="h-5 w-5" />
            {{ leftColumnTitle }}
          </h2>

          <Button
            variant="outline"
            size="sm"
            class="flex items-center gap-1 text-blue-600 border-blue-200 hover:bg-blue-50"
            @click="openMaintenanceSessionsModal"
          >
            <ListIcon class="h-4 w-4" />
            Danh sách các phiên sửa chữa
          </Button>
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
            <h3 class="text-lg font-medium mb-1">
              {{ PAGE_TITLES.NO_RECORDS }}
            </h3>
            <p class="text-sm text-gray-500 max-w-xs">
              {{
                isLoadingDeviceScan
                  ? PROCESSING_MESSAGES.DEVICE
                  : PROCESSING_MESSAGES.SCAN_DEVICE
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
                      :class="statusColorMap[item.status]"
                      variant="outline"
                      class="h-8 text-sm font-semibold w-fit"
                    >
                      {{ statusMap[item.status] }}
                    </Badge>
                    <span class="text-gray-400">→</span>
                    <div class="w-32">
                      <Select
                        v-model="item.maintenanceOutcome"
                        class="flex-grow"
                        @update:modelValue="
                          updateMaintenanceOutcome(
                            item,
                            item.maintenanceOutcome
                          )
                        "
                      >
                        <SelectTrigger
                          class="h-8 text-sm bg-white font-semibold w-fit"
                          :class="
                            item.maintenanceOutcome
                              ? statusColorMap[item.maintenanceOutcome]
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
                              Đã sửa
                            </Badge>
                          </SelectItem>
                          <SelectItem value="broken" class="cursor-pointer">
                            <Badge
                              :class="statusColorMap['broken']"
                              variant="outline"
                            >
                              Sửa một phần
                            </Badge>
                          </SelectItem>
                          <SelectItem value="discarded" class="cursor-pointer">
                            <Badge
                              :class="statusColorMap['discarded']"
                              variant="outline"
                            >
                              Không sửa được
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
                    ? PROCESSING_MESSAGES.USER
                    : PROCESSING_MESSAGES.SCAN_USER
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
            v-if="mode === 'maintenance' && devices.length > 0"
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
                  <span class="text-green-600 text-sm bg-white font-semibold">
                    Đã sửa
                  </span>
                  {{ maintenanceDetails.repairStatus.fixed }}
                </div>
                <div class="flex justify-between">
                  <span class="text-red-600 text-sm bg-white font-semibold">
                    Sửa một phần
                  </span>
                  {{ maintenanceDetails.repairStatus.partiallyFixed }}
                </div>
                <div class="flex justify-between">
                  <span class="text-amber-800 text-sm bg-white font-semibold">
                    Không sửa được
                  </span>
                  {{ maintenanceDetails.repairStatus.unrepairable }}
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
                  {{ maintenanceDetails.location }}
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
                  {{ maintenanceDetails.maintenanceDate }}
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
                placeholder="Thêm ghi chú về nội dung sửa chữa hoặc chi tiết linh kiện thay thế..."
                class="min-h-[80px]"
              />
            </div>

            <Button
              :disabled="isConfirming || !userInfo || devices.length === 0"
              class="w-full mt-4 bg-blue-600 hover:bg-blue-700"
              @click="completeMaintenance"
            >
              <LoaderIcon
                v-if="isConfirming"
                class="h-5 w-5 mr-2 animate-spin"
              />
              <WrenchIcon v-else class="h-5 w-5 mr-2" />
              Hoàn tất sửa chữa
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
          <p v-if="maintenanceId" class="text-base text-gray-600 mb-6">
            Mã sửa chữa: {{ maintenanceId.slice(0, 8) }}
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

    <div
      v-if="showMaintenanceSessionsModal"
      class="fixed inset-0 flex items-center justify-center z-50"
    >
      <div
        class="fixed inset-0 bg-black bg-opacity-60"
        @click="showMaintenanceSessionsModal = false"
      ></div>

      <div
        class="bg-white rounded-lg shadow-xl z-10 max-w-3xl w-full mx-4 overflow-hidden"
      >
        <div
          class="flex justify-between items-center border-b border-gray-200 px-6 py-4"
        >
          <h2 class="text-xl font-semibold leading-6 text-gray-900">
            Danh sách các phiên sửa chữa
          </h2>
          <Button
            variant="ghost"
            size="icon"
            @click="showMaintenanceSessionsModal = false"
            class="rounded-full"
          >
            <XIcon class="h-5 w-5" />
          </Button>
        </div>

        <div class="p-4 max-h-[70vh] overflow-y-auto">
          <div
            v-if="loadingMaintenanceSessions"
            class="flex flex-col items-center justify-center py-10"
          >
            <LoaderIcon class="h-8 w-8 text-blue-600 animate-spin mb-4" />
            <p class="text-gray-500">Đang tải danh sách phiên sửa chữa...</p>
          </div>

          <div
            v-else-if="maintenanceSessions.length === 0"
            class="flex flex-col items-center justify-center py-10 text-center"
          >
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <WrenchIcon class="h-8 w-8 text-gray-400" />
            </div>
            <p class="text-sm text-gray-500 max-w-xs">
              Không có phiên sửa chữa nào đang tiến hành.
            </p>
          </div>

          <div v-else>
            <div class="space-y-3">
              <div
                v-for="session in maintenanceSessions"
                :key="session.id"
                class="border border-gray-200 rounded-lg p-4 cursor-pointer transition-colors hover:bg-blue-50"
                :class="{
                  'border-blue-400 bg-blue-50':
                    selectedSession?.id === session.id,
                }"
                @click="selectedSession = session"
              >
                <div class="flex justify-between">
                  <div>
                    <p class="font-medium text-gray-900 flex items-center">
                      <span class="mr-2">Mã sửa chữa:</span>
                      <span class="font-semibold">{{
                        session.id.slice(0, 8)
                      }}</span>
                      <Badge
                        :class="{
                          'bg-blue-100 text-blue-700':
                            session.status === MaintenanceStatus.MAINTAINING,
                          'bg-green-100 text-green-700':
                            session.status === MaintenanceStatus.COMPLETED,
                          'bg-gray-100 text-gray-700':
                            session.status === MaintenanceStatus.CANCELLED,
                        }"
                        class="ml-2"
                      >
                        {{
                          session.status === MaintenanceStatus.MAINTAINING
                            ? "Đang sửa chữa"
                            : session.status === MaintenanceStatus.COMPLETED
                              ? "Hoàn thành"
                              : "Đã hủy"
                        }}
                      </Badge>
                    </p>
                    <p class="text-sm text-gray-500 mt-1">
                      Người sửa chữa:
                      {{ session.maintainerName || session.maintainerId }}
                    </p>
                    <p class="text-sm text-gray-500">
                      Số thiết bị: {{ session.deviceCount }}
                    </p>
                    <p class="text-sm text-gray-500">
                      Thời gian tạo:
                      {{ new Date(session.createdAt).toLocaleString("vi-VN") }}
                    </p>
                    <p v-if="session.finishedAt" class="text-sm text-gray-500">
                      Thời gian hoàn thành:
                      {{ new Date(session.finishedAt).toLocaleString("vi-VN") }}
                    </p>
                  </div>

                  <div class="flex flex-col gap-2">
                    <Button
                      v-if="
                        selectedSession?.id === session.id &&
                        session.status === MaintenanceStatus.MAINTAINING
                      "
                      variant="destructive"
                      size="sm"
                      class="text-xs"
                      @click.stop="cancelMaintenance(session.id)"
                    >
                      Huỷ sửa chữa
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>

        <div
          class="px-6 py-4 bg-gray-50 border-t border-gray-200 flex justify-between items-center gap-3"
        >
          <Button
            class="w-40"
            variant="outline"
            @click="showMaintenanceSessionsModal = false"
          >
            Đóng
          </Button>

          <Button
            class="w-40"
            variant="default"
            @click="continueMaintenance"
            :disabled="
              !selectedSession ||
              loadingMaintenanceSessions ||
              selectedSession?.status !== MaintenanceStatus.MAINTAINING
            "
          >
            <LoaderIcon
              v-if="loadingMaintenanceSessions"
              class="h-5 w-5 animate-spin"
            />
            <CheckIcon v-else class="h-5 w-5" />
            {{
              loadingMaintenanceSessions
                ? "Đang tải dữ liệu..."
                : "Tiếp tục sửa chữa"
            }}
          </Button>
        </div>
      </div>
    </div>
  </div>
</template>
