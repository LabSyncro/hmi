<script setup lang="ts">
import {
  ChevronDownIcon,
  ChevronUpIcon,
  ClipboardCheckIcon,
  LoaderIcon,
  PackageIcon,
  SearchIcon,
  TruckIcon,
  UserIcon,
  WrenchIcon,
} from "lucide-vue-next";

const deviceService = {
  getDeviceInventoryByKindId: async (_kindId: string, _labId: string) => {
    console.log("Mock getDeviceInventoryByKindId", _kindId, _labId);
    return [];
  },
  getDeviceReceiptById: async (_id: string, _labId: string | string[]) => {
    console.log("Mock getDeviceReceiptById", _id);
    return {
      fullId: "device-123",
      status: DeviceStatus.HEALTHY,
      image: null,
      unit: "EA",
      deviceName: "Mock Device",
      allowedBorrowRoles: ["admin", "user"],
      allowedViewRoles: ["admin", "user"],
      brand: "Mock Brand",
      manufacturer: "Mock Manufacturer",
      description: "This is a mock device",
      isBorrowableLabOnly: false,
      categoryName: "Electronics",
      labRoom: "A101",
      labBranch: "Main",
      kind: "laptop",
    };
  },
};

const mode = ref<"idle" | "device" | "user">("idle");

const showMore = ref(false);
const showAccessories = ref(false);
const route = useRoute();
const deviceDetail = ref<DeviceDetail | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const retrying = ref(false);
const inventory = ref<Array<any>>([]);
const loadingInventory = ref(true);

const activeTab = ref("inventory");
const borrowedDevices = ref<Array<any>>([]);
const maintenanceDevices = ref<Array<any>>([]);
const transportDevices = ref<Array<any>>([]);
const auditDevices = ref<Array<any>>([]);
const loadingBorrowedItems = ref(false);
const loadingMaintenanceItems = ref(false);
const loadingTransportItems = ref(false);
const loadingAuditItems = ref(false);

const userInfo = ref<UserInfo | null>(null);
const storedUserInfo = ref<{
  id: string;
  lab: { id: string; room: string; branch: string };
} | null>(null);

const isLoadingDeviceScan = ref(false);
const isLoadingUser = ref(false);
const isConfirming = ref(false);

const pageTitle = computed(() => {
  if (mode.value === "device") return "THÔNG TIN THIẾT BỊ";
  if (mode.value === "user") return "THÔNG TIN NGƯỜI DÙNG";
  return "TRA CỨU NHANH";
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

    mode.value = "user";

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
    isLoadingUser.value = false;
    isConfirming.value = false;
  }
}

async function handleDeviceScan(input: string) {
  isLoadingDeviceScan.value = true;
  try {
    const deviceKindId = input.match(/\/devices\/([a-fA-F0-9]+)/)?.[1];
    const deviceId = input.match(/[?&]id=([a-fA-F0-9]+)/)?.[1];

    if (!deviceId || !deviceKindId) {
      toast({
        title: "Lỗi",
        description: "Không thể trích xuất ID thiết bị từ mã QR",
        variant: "destructive",
      });
      return;
    }

    await loadDeviceDetailsById(deviceId, deviceKindId);
    mode.value = "device";
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể tải thông tin thiết bị",
      variant: "destructive",
    });
  } finally {
    isLoadingDeviceScan.value = false;
  }
}

async function handleOneTimeQRScan(input: string) {
  isLoadingUser.value = true;
  try {
    const oneTimeQRService = useOneTimeQR();
    const result = await oneTimeQRService.verifyScannedQrCode(input);

    if (
      result &&
      typeof result === "object" &&
      "user" in result &&
      result.user
    ) {
      const { user } = result;
      userInfo.value = {
        id: user.id,
        name: user.name,
        avatar: user.avatar,
        roles: user.roles,
      };

      mode.value = "user";

      toast({
        title: "Thành công",
        description: `Đã nhận diện: ${user.name}`,
        variant: "success",
      });
      return;
    }

    toast({
      title: "Lỗi",
      description: "Mã QR không hợp lệ hoặc đã hết hạn",
      variant: "destructive",
    });
  } catch (error) {
    toast({
      title: "Lỗi xử lý mã QR",
      description: "Vui lòng thử lại",
      variant: "destructive",
    });
  } finally {
    isLoadingUser.value = false;
  }
}

const handleVirtualKeyboardDetection = async (
  input: string,
  type?: "userId" | "device" | "oneTimeQR"
) => {
  if (type === "device") {
    await handleDeviceScan(input);
  } else if (type === "userId") {
    await handleUserCodeChange(input);
  } else if (type === "oneTimeQR") {
    await handleOneTimeQRScan(input);
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

async function loadDeviceDetailsById(id: string, kindId: string) {
  loading.value = true;
  error.value = null;
  try {
    const labId = storedUserInfo.value?.lab?.id || "";
    deviceDetail.value = await deviceService.getDeviceReceiptById(id, [labId]);
    if (!deviceDetail.value) {
      error.value = "Device not found";
    } else {
      await loadInventoryData(kindId || deviceDetail.value.kind);
    }
  } catch (e) {
    error.value =
      e instanceof Error ? e.message : "Failed to load device details";
  } finally {
    loading.value = false;
    retrying.value = false;
  }
}

async function loadDeviceDetails() {
  loading.value = true;
  error.value = null;
  try {
    const id = route.params.id as string;
    const labId = storedUserInfo.value?.lab?.id || "";
    deviceDetail.value = await deviceService.getDeviceReceiptById(id, [labId]);
    if (!deviceDetail.value) {
      error.value = "Device not found";
    } else {
      const kindId =
        (route.query.deviceKindId as string) || deviceDetail.value.kind;
      await loadInventoryData(kindId);
    }
  } catch (e) {
    error.value =
      e instanceof Error ? e.message : "Failed to load device details";
  } finally {
    loading.value = false;
    retrying.value = false;
  }
}

async function loadInventoryData(kindId: string) {
  loadingInventory.value = true;
  try {
    if (deviceDetail.value) {
      const inventoryData = await deviceService.getDeviceInventoryByKindId(
        kindId,
        storedUserInfo.value?.lab?.id || ""
      );

      inventory.value = [
        {
          id: "inv-001",
          fullId: "LAP-001",
          deviceName: "Dell XPS 13",
          status: "HEALTHY",
          location: "Phòng 601 H6",
        },
        {
          id: "inv-002",
          fullId: "LAP-002",
          deviceName: "HP Elitebook",
          status: "HEALTHY",
          location: "Phòng 601 H6",
        },
        {
          id: "inv-003",
          fullId: "LAP-003",
          deviceName: "Macbook Pro",
          status: "BROKEN",
          location: "Phòng 601 H6",
        },
        {
          id: "inv-004",
          fullId: "LAP-004",
          deviceName: "Lenovo ThinkPad",
          status: "HEALTHY",
          location: "Phòng 602 H6",
        },
        {
          id: "inv-005",
          fullId: "LAP-005",
          deviceName: "Asus ZenBook",
          status: "HEALTHY",
          location: "Phòng 602 H6",
        },
        {
          id: "inv-006",
          fullId: "LAP-006",
          deviceName: "Microsoft Surface",
          status: "BROKEN",
          location: "Phòng 701 H6",
        },
        {
          id: "inv-007",
          fullId: "LAP-007",
          deviceName: "Acer Predator",
          status: "HEALTHY",
          location: "Phòng 701 H6",
        },
        {
          id: "inv-008",
          fullId: "LAP-008",
          deviceName: "Dell Inspiron",
          status: "HEALTHY",
          location: "Phòng 701 H6",
        },
        {
          id: "inv-009",
          fullId: "LAP-009",
          deviceName: "HP Pavilion",
          status: "BROKEN",
          location: "Phòng 701 H6",
        },
      ];

      if (inventoryData && inventory.value.length === 0) {
        if (Array.isArray(inventoryData)) {
          inventory.value = inventoryData;
        } else if (typeof inventoryData === "object") {
          const devices = (inventoryData as any).devices;
          if (Array.isArray(devices)) {
            inventory.value = devices;
          } else {
            inventory.value = [inventoryData];
          }
        } else {
          inventory.value = [];
        }
      }

      await loadBorrowedDevices();
      await loadMaintenanceDevices();
      await loadTransportDevices();
      await loadAuditDevices();
    }
  } catch (error) {
    console.error("Error loading inventory data", error);
  } finally {
    loadingInventory.value = false;
  }
}

async function loadBorrowedDevices() {
  loadingBorrowedItems.value = true;
  try {
    // This would be replaced with a real API call
    // For now, simulating with a timeout and mock data
    await new Promise((resolve) => setTimeout(resolve, 500));
    // In a real implementation, you would fetch borrowed devices from a service
    // Example: const data = await deviceService.getBorrowedDevices(deviceDetail.value?.kind, storedUserInfo.value?.lab?.id || "");
    borrowedDevices.value = [
      {
        id: "1",
        fullId: "Thiết bị 001",
        status: "ON_TIME",
        borrower: { name: "Nguyễn Văn A" },
        borrowDate: "2025-04-22T10:00:00Z",
        expectedReturnAt: "2025-05-06T10:00:00Z",
      },
      {
        id: "2",
        fullId: "Thiết bị 002",
        status: "NEAR_DUE",
        borrower: { name: "Trần Thị B" },
        borrowDate: "2025-04-15T10:00:00Z",
        expectedReturnAt: "2025-05-01T10:00:00Z",
      },
      {
        id: "3",
        fullId: "Thiết bị 003",
        status: "OVERDUE",
        borrower: { name: "Lê Văn C" },
        borrowDate: "2025-03-15T10:00:00Z",
        expectedReturnAt: "2025-04-15T10:00:00Z",
      },
    ];
  } catch (error) {
    console.error("Error loading borrowed devices", error);
  } finally {
    loadingBorrowedItems.value = false;
  }
}

async function loadMaintenanceDevices() {
  loadingMaintenanceItems.value = true;
  try {
    // This would be replaced with a real API call
    // For now, simulating with a timeout and mock data
    await new Promise((resolve) => setTimeout(resolve, 500));
    // In a real implementation, you would fetch maintenance devices from a service
    // Example: const data = await deviceService.getMaintenanceDevices(deviceDetail.value?.kind, storedUserInfo.value?.lab?.id || "");
    maintenanceDevices.value = [
      {
        id: "1",
        fullId: "Thiết bị 004",
        maintenanceReason: "Sửa chữa môđun hiển thị",
        technician: { name: "Kỹ thuật viên Minh" },
        maintenanceStartDate: "2025-04-20T10:00:00Z",
        expectedCompletionDate: "2025-05-05T10:00:00Z",
      },
      {
        id: "2",
        fullId: "Thiết bị 005",
        maintenanceReason: "Bảo dưỡng định kỳ",
        technician: { name: "Kỹ thuật viên Hưng" },
        maintenanceStartDate: "2025-04-25T10:00:00Z",
        expectedCompletionDate: "2025-04-30T10:00:00Z",
      },
    ];
  } catch (error) {
    console.error("Error loading maintenance devices", error);
  } finally {
    loadingMaintenanceItems.value = false;
  }
}

async function loadTransportDevices() {
  loadingTransportItems.value = true;
  try {
    // This would be replaced with a real API call
    // For now, simulating with a timeout and mock data
    await new Promise((resolve) => setTimeout(resolve, 500));
    // In a real implementation, you would fetch transport devices from a service
    // Example: const data = await deviceService.getTransportDevices(deviceDetail.value?.kind, storedUserInfo.value?.lab?.id || "");
    transportDevices.value = [
      {
        id: "1",
        fullId: "Thiết bị 006",
        sourceLocation: "Kho trung tâm",
        destinationLocation: "Phòng TN 601 H6",
        transportDate: "2025-04-28T10:00:00Z",
        status: "Đang vận chuyển",
      },
      {
        id: "2",
        fullId: "Thiết bị 007",
        sourceLocation: "Phòng TN 605 H6",
        destinationLocation: "Phòng TN 812 H6",
        transportDate: "2025-04-27T10:00:00Z",
        status: "Đang vận chuyển",
      },
    ];
  } catch (error) {
    console.error("Error loading transport devices", error);
  } finally {
    loadingTransportItems.value = false;
  }
}

async function loadAuditDevices() {
  loadingAuditItems.value = true;
  try {
    // This would be replaced with a real API call
    // For now, simulating with a timeout and mock data
    await new Promise((resolve) => setTimeout(resolve, 500));
    // In a real implementation, you would fetch audit devices from a service
    // Example: const data = await deviceService.getAuditDevices(deviceDetail.value?.kind, storedUserInfo.value?.lab?.id || "");
    auditDevices.value = [
      {
        id: "1",
        fullId: "Thiết bị 008",
        auditor: { name: "Người kiểm đếm Hóa" },
        auditDate: "2025-04-25T10:00:00Z",
        auditResult: "Hoàn thành",
        notes: "Thiết bị hoạt động tốt",
      },
      {
        id: "2",
        fullId: "Thiết bị 009",
        auditor: { name: "Người kiểm đếm Tuấn" },
        auditDate: "2025-04-26T10:00:00Z",
        auditResult: "Đang kiểm đếm",
        notes: "",
      },
    ];
  } catch (error) {
    console.error("Error loading audit devices", error);
  } finally {
    loadingAuditItems.value = false;
  }
}

const groupByLocation = (devices: any[]) => {
  const grouped: Record<string, any[]> = {};

  devices.forEach((device) => {
    if (!grouped[device.location]) {
      grouped[device.location] = [];
    }
    grouped[device.location].push(device);
  });

  return grouped;
};

const countDevicesByStatus = (devices: any[], status: string) => {
  return devices.filter((device) => device.status === status).length;
};

const getBorrowedCountForLocation = (
  _locationName: string,
  _isGoodCondition: boolean
) => {
  return 10;
};

const getBorrowedStatusClass = (item: any) => {
  return {
    "bg-green-500": item.status === "ON_TIME",
    "bg-yellow-500": item.status === "NEAR_DUE",
    "bg-red-500": item.status === "OVERDUE",
  };
};

const getBorrowedProgressClass = (item: any) => {
  return {
    "bg-green-50 text-green-800": item.status === "ON_TIME",
    "bg-yellow-50 text-yellow-800": item.status === "NEAR_DUE",
    "bg-red-50 text-red-800": item.status === "OVERDUE",
  };
};

const calculateReturnProgress = (date: string) => {
  if (!date) return "N/A";

  const dueDate = new Date(date);
  const today = new Date();
  const daysLeft = Math.ceil(
    (dueDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24)
  );

  if (daysLeft > 5) return "Còn " + daysLeft + " ngày";
  if (daysLeft > 0) return "Sắp đến hạn: " + daysLeft + " ngày";
  if (daysLeft === 0) return "Đến hạn hôm nay";
  return "Quá hạn " + Math.abs(daysLeft) + " ngày";
};

const formatDate = (dateString: string) => {
  if (!dateString) return "N/A";

  const date = new Date(dateString);
  return date.toLocaleDateString("vi-VN", {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  });
};

const getMaintenanceStatusClass = () => {
  return "bg-amber-500";
};

const resetToIdle = () => {
  mode.value = "idle";
  activeTab.value = "inventory";
  deviceDetail.value = null;
};

function retryLoading() {
  retrying.value = true;
  loadDeviceDetailsById(route.params.id as string, "");
}

onMounted(() => {
  const stored = localStorage.getItem("user_info");
  if (stored) {
    const ui = JSON.parse(stored) as {
      id: string;
      lab: { id: string; room: string; branch: string };
    };
    storedUserInfo.value = ui;
  }

  if (route.params.id) {
    loadDeviceDetails();
    mode.value = "device";
  }
});

watch(
  () => [route.params.id, route.query.deviceKindId],
  () => {
    if (route.params.id) {
      loadDeviceDetails();
      mode.value = "device";
    }
  }
);
</script>

<template>
  <div>
    <h1 class="text-2xl font-bold text-center">{{ pageTitle }}</h1>
    <p class="text-center text-gray-500 mb-2">
      Sử dụng máy scan quét mã QR thiết bị/người dùng để tra cứu thông tin
      nhanh.
    </p>

    <div v-if="mode === 'idle'" class="text-center">
      <div class="max-w-sm mx-auto bg-white rounded-full shadow p-16">
        <div
          class="rounded-full bg-gray-100 mx-auto w-36 h-36 flex items-center justify-center mb-4"
        >
          <SearchIcon class="h-12 w-12 text-gray-400" />
        </div>
        <h2 class="text-xl font-semibold mb-2">CHƯA CÓ THÔNG TIN</h2>
        <p class="text-gray-600">
          Quét mã QR thiết bị / người dùng<br />để hiển thị thông tin chi tiết
        </p>
      </div>
    </div>

    <div
      v-else-if="isLoadingDeviceScan || isLoadingUser || loading"
      class="text-center py-12 bg-white rounded-lg shadow p-8"
    >
      <div class="max-w-md mx-auto">
        <div class="flex justify-center mb-4">
          <LoaderIcon class="h-10 w-10 animate-spin text-blue-500" />
        </div>
        <p class="mt-2 text-sm text-gray-600">
          {{
            isLoadingDeviceScan
              ? "Đang tải thông tin thiết bị..."
              : isLoadingUser
                ? "Đang tải thông tin người dùng..."
                : "Đang tải thông tin..."
          }}
        </p>
      </div>
    </div>

    <div v-else-if="error" class="bg-red-50 p-4 rounded-md">
      <p class="text-red-700">{{ error }}</p>
      <button
        @click="retryLoading"
        class="mt-3 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
        :disabled="retrying"
      >
        <span v-if="retrying">Đang thử lại...</span>
        <span v-else>Thử lại</span>
      </button>
    </div>

    <div v-else-if="mode === 'device' && deviceDetail">
      <div>
        <div class="grid grid-cols-3 gap-6">
          <div
            class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
          >
            <Tabs v-model="activeTab" class="w-full">
              <div class="border-b border-gray-200">
                <TabsList class="bg-transparent p-0 w-full flex">
                  <TabsTrigger
                    value="inventory"
                    class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                  >
                    <div class="flex items-center justify-center gap-1 w-full">
                      <div class="rounded-full bg-blue-50 p-1">
                        <PackageIcon class="h-4 w-4 text-blue-600" />
                      </div>
                      <span class="whitespace-nowrap">TỒN KHO</span>
                      <span
                        class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                      >
                        {{ inventory.length || 0 }}
                      </span>
                    </div>
                  </TabsTrigger>
                  <TabsTrigger
                    value="borrowed"
                    class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                  >
                    <div class="flex items-center justify-center gap-1 w-full">
                      <div class="rounded-full bg-violet-50 p-1">
                        <UserIcon class="h-4 w-4 text-violet-600" />
                      </div>
                      <span class="whitespace-nowrap">ĐANG MƯỢN</span>
                      <span
                        class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                      >
                        {{ borrowedDevices.length || 0 }}
                      </span>
                    </div>
                  </TabsTrigger>
                  <TabsTrigger
                    value="maintenance"
                    class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                  >
                    <div class="flex items-center justify-center gap-1 w-full">
                      <div class="rounded-full bg-amber-50 p-1">
                        <WrenchIcon class="h-4 w-4 text-amber-600" />
                      </div>
                      <span class="whitespace-nowrap">BẢO TRÌ</span>
                      <span
                        class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                      >
                        {{ maintenanceDevices.length || 0 }}
                      </span>
                    </div>
                  </TabsTrigger>
                  <TabsTrigger
                    value="transport"
                    class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                  >
                    <div class="flex items-center justify-center gap-1 w-full">
                      <div class="rounded-full bg-emerald-50 p-1">
                        <TruckIcon class="h-4 w-4 text-emerald-600" />
                      </div>
                      <span class="whitespace-nowrap">VẬN CHUYỂN</span>
                      <span
                        class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                      >
                        {{ transportDevices.length || 0 }}
                      </span>
                    </div>
                  </TabsTrigger>
                  <TabsTrigger
                    value="audit"
                    class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                  >
                    <div class="flex items-center justify-center gap-1 w-full">
                      <div class="rounded-full bg-rose-50 p-1">
                        <ClipboardCheckIcon class="h-4 w-4 text-rose-600" />
                      </div>
                      <span class="whitespace-nowrap">KIỂM ĐẾM</span>
                      <span
                        class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                      >
                        {{ auditDevices.length || 0 }}
                      </span>
                    </div>
                  </TabsTrigger>
                </TabsList>
              </div>

              <TabsContent
                value="inventory"
                class="p-4 h-[calc(100vh-16rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingInventory">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="inventory && inventory.length">
                  <div class="mb-3">
                    <div class="text-sm text-gray-500">
                      Số lượng còn lại: {{ inventory.length }} thiết bị
                    </div>
                  </div>

                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                            rowspan="2"
                          >
                            PHÒNG THÍ NGHIỆM
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-2 text-center text-sm font-semibold text-blue-900 border"
                            colspan="2"
                          >
                            TẠI PHÒNG
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-2 text-center text-sm font-semibold text-blue-900 border"
                            colspan="2"
                          >
                            ĐANG MƯỢN
                          </th>
                        </tr>
                        <tr>
                          <th
                            class="bg-blue-50 px-4 py-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Tốt
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Hư
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Tốt
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Hư
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="(group, location) in groupByLocation(
                            inventory
                          )"
                          :key="location"
                          class="hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            {{ location }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ countDevicesByStatus(group, "HEALTHY") }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ countDevicesByStatus(group, "BROKEN") }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ getBorrowedCountForLocation(location, true) }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ getBorrowedCountForLocation(location, false) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                <div v-else class="flex flex-col items-center py-6">
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <PackageIcon class="size-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Không có thiết bị nào trong kho
                  </p>
                </div>
              </TabsContent>

              <TabsContent
                value="borrowed"
                class="p-4 h-[calc(100vh-16rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingBorrowedItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="borrowedDevices && borrowedDevices.length">
                  <div class="mb-3">
                    <div class="text-sm text-gray-500">
                      Số lượng đang mượn: {{ borrowedDevices.length }} thiết bị
                    </div>
                  </div>

                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            THIẾT BỊ
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGƯỜI MƯỢN
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY MƯỢN
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            HẠN TRẢ
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TRẠNG THÁI
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in borrowedDevices"
                          :key="item.id"
                          class="hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center">
                              <div
                                class="size-3 rounded-full mr-2"
                                :class="getBorrowedStatusClass(item)"
                              ></div>
                              <span>{{ item.fullId }}</span>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.borrower?.name || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.borrowDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.expectedReturnAt) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full"
                              :class="getBorrowedProgressClass(item)"
                            >
                              {{
                                calculateReturnProgress(item.expectedReturnAt)
                              }}
                            </span>
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                <div v-else class="flex flex-col items-center py-6">
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <UserIcon class="size-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Không có thiết bị nào đang được mượn
                  </p>
                </div>
              </TabsContent>

              <TabsContent
                value="maintenance"
                class="p-4 h-[calc(100vh-16rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingMaintenanceItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div
                  v-else-if="maintenanceDevices && maintenanceDevices.length"
                >
                  <div class="mb-3">
                    <div class="text-sm text-gray-500">
                      Số lượng đang bảo trì:
                      {{ maintenanceDevices.length }} thiết bị
                    </div>
                  </div>

                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            THIẾT BỊ
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            LÝ DO
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            KỸ THUẬT VIÊN
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY BẮT ĐẦU
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            DỰ KIẾN HOÀN THÀNH
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in maintenanceDevices"
                          :key="item.id"
                          class="hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center">
                              <div
                                class="size-3 rounded-full mr-2"
                                :class="getMaintenanceStatusClass()"
                              ></div>
                              <span>{{ item.fullId }}</span>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.maintenanceReason || "Đang bảo trì" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.technician?.name || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.maintenanceStartDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.expectedCompletionDate) }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                <div v-else class="flex flex-col items-center py-6">
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <WrenchIcon class="size-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Không có thiết bị nào đang bảo trì
                  </p>
                </div>
              </TabsContent>

              <TabsContent
                value="transport"
                class="p-4 h-[calc(100vh-16rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingTransportItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="transportDevices && transportDevices.length">
                  <div class="mb-3">
                    <div class="text-sm text-gray-500">
                      Số lượng đang vận chuyển:
                      {{ transportDevices.length }} thiết bị
                    </div>
                  </div>

                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            THIẾT BỊ
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TỪ ĐỊA ĐIỂM
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            ĐẾN ĐỊA ĐIỂM
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY VẬN CHUYỂN
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TRẠNG THÁI
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in transportDevices"
                          :key="item.id"
                          class="hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center">
                              <div
                                class="size-3 rounded-full mr-2 bg-emerald-500"
                              ></div>
                              <span>{{ item.fullId }}</span>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.sourceLocation || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.destinationLocation || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.transportDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full bg-emerald-50 text-emerald-800"
                            >
                              {{ item.status || "Đang vận chuyển" }}
                            </span>
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                <div v-else class="flex flex-col items-center py-6">
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <TruckIcon class="size-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Không có thiết bị nào đang vận chuyển
                  </p>
                </div>
              </TabsContent>

              <TabsContent
                value="audit"
                class="p-4 h-[calc(100vh-16rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingAuditItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="auditDevices && auditDevices.length">
                  <div class="mb-3">
                    <div class="text-sm text-gray-500">
                      Số lượng đang kiểm đếm: {{ auditDevices.length }} thiết bị
                    </div>
                  </div>

                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            THIẾT BỊ
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGƯỜI KIỂM ĐẾm
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY KIỂM ĐẾm
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            KẾT QUẢ
                          </th>
                          <th
                            class="bg-blue-50 px-4 py-3 text-left text-sm font-semibold text-blue-900 border"
                          >
                            GHI CHÚ
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in auditDevices"
                          :key="item.id"
                          class="hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center">
                              <div
                                class="size-3 rounded-full mr-2 bg-rose-500"
                              ></div>
                              <span>{{ item.fullId }}</span>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.auditor?.name || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.auditDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full bg-blue-50 text-blue-800"
                            >
                              {{ item.auditResult || "Đang kiểm đếm" }}
                            </span>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.notes || "—" }}
                          </td>
                        </tr>
                      </tbody>
                    </table>
                  </div>
                </div>
                <div v-else class="flex flex-col items-center py-6">
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <ClipboardCheckIcon class="size-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Không có thiết bị nào đang kiểm đếm
                  </p>
                </div>
              </TabsContent>
            </Tabs>
          </div>

          <div
            class="bg-white h-[calc(100vh-12rem)] overflow-y-auto rounded-lg shadow-sm border border-gray-200"
          >
            <div class="p-4 border-b border-gray-200">
              <h2 class="text-xl font-semibold text-gray-700">
                THÔNG TIN THIẾT BỊ
              </h2>
            </div>

            <div class="p-4">
              <div class="flex items-center mb-4">
                <div
                  class="flex-shrink-0 mr-4 w-20 h-20 bg-gray-100 rounded border"
                >
                  <img
                    :src="deviceDetail.image?.mainImage || '/device-image.svg'"
                    :alt="deviceDetail.deviceName || 'Device Image'"
                    class="w-full h-full object-cover"
                  />
                </div>

                <div class="flex-grow">
                  <div class="text-sm text-gray-500 mb-1">
                    Mã thiết bị: {{ deviceDetail.fullId || "device-123" }}
                  </div>
                  <h1 class="text-xl font-medium text-gray-900">
                    {{ deviceDetail.deviceName || "Mock Device" }}
                  </h1>
                </div>
              </div>

              <div class="mb-6 space-y-2">
                <div class="grid grid-cols-[120px_1fr]">
                  <div class="text-base text-gray-500">Tình trạng</div>
                  <div class="text-base font-medium text-red-600">Hư</div>
                </div>

                <div class="grid grid-cols-[120px_1fr]">
                  <div class="text-base text-gray-500">Hoạt động</div>
                  <div class="text-base text-gray-900">Sửa chữa: Đang chờ</div>
                </div>

                <div class="grid grid-cols-[120px_1fr]">
                  <div class="text-base text-gray-500">Nơi chứa</div>
                  <div class="text-base text-gray-900">
                    {{
                      deviceDetail.labRoom && deviceDetail.labBranch
                        ? deviceDetail.labRoom.split("-")[1] +
                          " " +
                          deviceDetail.labRoom.split("-")[0] +
                          ", " +
                          deviceDetail.labBranch
                        : "A101, Main"
                    }}
                  </div>
                </div>
              </div>

              <div class="border rounded-md">
                <div
                  class="p-4 flex justify-between items-center cursor-pointer"
                  @click="showMore = !showMore"
                >
                  <h3 class="text-base text-blue-600 font-medium">Chi tiết</h3>
                  <ChevronDownIcon
                    v-if="!showMore"
                    class="h-5 w-5 text-blue-500"
                  />
                  <ChevronUpIcon v-else class="h-5 w-5 text-blue-500" />
                </div>

                <div v-if="showMore" class="p-4 border-t">
                  <div class="space-y-3">
                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Phân loại</div>
                      <div class="flex-1 text-sm text-gray-900">
                        {{ deviceDetail.categoryName || "Máy hàn khô" }}
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Thương hiệu</div>
                      <div class="flex-1 text-sm text-gray-900">
                        {{ deviceDetail.brand || "OEM" }}
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Quyền mượn</div>
                      <div class="flex-1 text-sm text-gray-900">
                        {{
                          deviceDetail.allowedBorrowRoles?.join(", ") ||
                          "Sinh viên, Giảng viên"
                        }}
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Môn đang học</div>
                      <div class="flex-1 text-sm text-gray-900">
                        Kỹ thuật Lập trình<br />
                        Lập trình ứng dụng IoT<br />
                        Đồ án Đa ngành
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Môn đã học</div>
                      <div class="flex-1 text-sm text-gray-900">
                        Hệ thống số
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              <div class="border rounded-md mt-3">
                <div
                  class="p-4 flex justify-between items-center cursor-pointer"
                  @click="showAccessories = !showAccessories"
                >
                  <h3 class="text-base text-blue-600 font-medium">
                    Dụng cụ đi kèm
                  </h3>
                  <ChevronDownIcon
                    v-if="!showAccessories"
                    class="h-5 w-5 text-blue-500"
                  />
                  <ChevronUpIcon v-else class="h-5 w-5 text-blue-500" />
                </div>
                <div v-if="showAccessories" class="p-4 border-t">
                  <p class="text-sm text-gray-500">Không có dụng cụ đi kèm</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="mode === 'user' && userInfo"
      class="bg-white rounded-lg shadow p-6"
    >
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div class="md:col-span-1">
          <div class="flex items-center mb-4">
            <img
              :src="userInfo.avatar || '/user-image.svg'"
              :alt="userInfo.name || 'Unknown User'"
              class="h-16 w-16 rounded-full bg-gray-100 object-cover"
            />
          </div>
        </div>
        <div class="md:col-span-2">
          <div class="space-y-4">
            <div class="col-span-2">
              <div class="text-sm text-gray-500">MÃ SỐ: {{ userInfo.id }}</div>
              <div class="text-2xl font-bold">{{ userInfo.name }}</div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="col-span-2">
                <div class="text-sm font-medium text-gray-500">Vai trò</div>
                <div class="mt-1 text-sm text-gray-900">
                  {{ userInfo.roles?.[0] || "Sinh viên" }}
                </div>
              </div>
              <div class="col-span-2">
                <div class="text-sm font-medium text-gray-500">Trạng thái</div>
                <div class="mt-1 text-sm text-green-600 font-medium">
                  Hoạt động
                </div>
              </div>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="col-span-2">
                <div class="text-sm font-medium text-gray-500">Email</div>
                <div class="mt-1 text-sm text-gray-900">
                  {{ userInfo.id }}@stu.edu.vn
                </div>
              </div>
              <div class="col-span-2">
                <div class="text-sm font-medium text-gray-500">
                  Số điện thoại
                </div>
                <div class="text-sm text-gray-900">
                  {{ "(Chưa cập nhật)" }}
                </div>
              </div>
            </div>
          </div>

          <div class="mt-6">
            <button
              type="button"
              class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
              @click="resetToIdle"
            >
              Trở về
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
