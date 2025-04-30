<script setup lang="ts">
import {
  CheckCircleIcon,
  ChevronDownIcon,
  ChevronUpIcon,
  ClipboardCheckIcon,
  ClockIcon,
  LoaderIcon,
  PackageIcon,
  SearchIcon,
  TruckIcon,
  UserIcon,
  WrenchIcon,
} from "lucide-vue-next";

const mode = ref<"idle" | "device" | "user">("idle");

const showMore = ref(false);
const showAccessories = ref(false);
const showUserDetails = ref(false);
const deviceDetail = ref<DeviceItem | null>(null);
const currentDeviceId = ref<string>("");
const loading = ref(false);
const error = ref<string | null>(null);
const retrying = ref(false);
const inventory = ref<Array<any>>([]);
const loadingInventory = ref(true);
const accessories = ref<Accessory[]>([]);
const loadingAccessories = ref(false);

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

const showUserInfo = ref(false);
const userBorrowedDevices = ref<UserBorrowHistoryItem[]>([]);
const loadingUserBorrowedItems = ref(false);
const groupedBorrowedDevices = ref<GroupedDevice[]>([]);

const userActivities = ref<UserActivityItem[]>([]);
const loadingUserActivities = ref(false);
const userActiveTab = ref("borrowed");

type GroupedDevice = {
  kindId: string;
  deviceName: string;
  image: { mainImage: string | null };
  labBranch: string;
  labRoom: string;
  quantity: number;
  expanded: boolean;
  deviceBorrowableLabOnly: boolean;
  items: UserBorrowHistoryItem[];
};

function groupUserBorrowedDevices() {
  const groups: Record<string, GroupedDevice> = {};

  userBorrowedDevices.value.forEach((item: UserBorrowHistoryItem) => {
    const kindId = item.deviceKindId;
    if (!groups[kindId]) {
      groups[kindId] = {
        kindId: kindId,
        deviceName: item.deviceName,
        image: item.deviceImage,
        labBranch: item.labBranch,
        labRoom: item.labRoom,
        deviceBorrowableLabOnly: item.deviceBorrowableLabOnly,
        quantity: 0,
        expanded: false,
        items: [],
      };
    }
    groups[kindId].items.push(item);
    groups[kindId].quantity = groups[kindId].items.length;
  });

  groupedBorrowedDevices.value = Object.values(groups);
}

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
      email: userMeta.email,
    };

    mode.value = "user";
    showUserInfo.value = true;

    loadingUserBorrowedItems.value = true;
    try {
      userBorrowedDevices.value =
        await userService.getBorrowedHistoryByUser(userId);
      groupUserBorrowedDevices();
    } catch (historyError) {
      toast({
        title: "Lỗi",
        description: "Không thể tải lịch sử mượn",
        variant: "destructive",
      });
      userBorrowedDevices.value = [];
      groupedBorrowedDevices.value = [];
    } finally {
      loadingUserBorrowedItems.value = false;
    }

    loadingUserActivities.value = true;
    try {
      userActivities.value = await userService.getUserActivitiesHistory(userId);
    } catch (activitiesError) {
      toast({
        title: "Lỗi",
        description: "Không thể tải hoạt động người dùng",
        variant: "destructive",
      });
      userActivities.value = [];
    } finally {
      loadingUserActivities.value = false;
    }

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
        email: user.email,
      };

      mode.value = "user";
      showUserInfo.value = true;

      loadingUserBorrowedItems.value = true;
      try {
        userBorrowedDevices.value = await userService.getBorrowedHistoryByUser(
          user.id
        );
        groupUserBorrowedDevices();
      } catch (historyError) {
        toast({
          title: "Lỗi",
          description: "Không thể tải lịch sử mượn",
          variant: "destructive",
        });
        userBorrowedDevices.value = [];
        groupedBorrowedDevices.value = [];
      } finally {
        loadingUserBorrowedItems.value = false;
      }

      loadingUserActivities.value = true;
      try {
        userActivities.value = await userService.getUserActivitiesHistory(
          user.id
        );
      } catch (activitiesError) {
        toast({
          title: "Lỗi",
          description: "Không thể tải hoạt động người dùng",
          variant: "destructive",
        });
        userActivities.value = [];
      } finally {
        loadingUserActivities.value = false;
      }

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
    const labId = storedUserInfo.value?.lab?.id;
    currentDeviceId.value = id;
    deviceDetail.value = await deviceService.getDeviceById(id, labId);
    if (!deviceDetail.value) {
      error.value = "Device not found";
    } else {
      const effectiveKindId = kindId || deviceDetail.value.kind;
      await loadInventoryData(effectiveKindId);
      await loadAccessoriesData(effectiveKindId, labId);
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
      const inventorySummary =
        await deviceService.getDeviceInventoryByKind(kindId);
      inventory.value = inventorySummary;

      if (currentDeviceId.value) {
        await loadBorrowedDevices(currentDeviceId.value);
        await loadMaintenanceDevices(currentDeviceId.value);
        await loadTransportDevices(currentDeviceId.value);
        await loadAuditDevices(currentDeviceId.value);
      }
    }
  } catch (error) {
    throw error;
  } finally {
    loadingInventory.value = false;
  }
}

async function loadBorrowedDevices(deviceId: string) {
  loadingBorrowedItems.value = true;
  try {
    const borrowHistory = await deviceService.getDeviceBorrowHistory(deviceId);
    borrowedDevices.value = borrowHistory;
  } catch (error) {
    borrowedDevices.value = [];
    throw error;
  } finally {
    loadingBorrowedItems.value = false;
  }
}

async function loadMaintenanceDevices(deviceId: string) {
  loadingMaintenanceItems.value = true;
  try {
    const maintenanceHistory =
      await deviceService.getDeviceMaintenanceHistory(deviceId);
    maintenanceDevices.value = maintenanceHistory;
  } catch (error) {
    maintenanceDevices.value = [];
    throw error;
  } finally {
    loadingMaintenanceItems.value = false;
  }
}

async function loadTransportDevices(deviceId: string) {
  loadingTransportItems.value = true;
  try {
    const transportHistory =
      await deviceService.getDeviceTransportHistory(deviceId);
    transportDevices.value = transportHistory;
  } catch (error) {
    transportDevices.value = [];
    throw error;
  } finally {
    loadingTransportItems.value = false;
  }
}

async function loadAuditDevices(deviceId: string) {
  loadingAuditItems.value = true;
  try {
    const auditHistory = await deviceService.getDeviceAuditHistory(deviceId);
    auditDevices.value = auditHistory;
  } catch (error) {
    auditDevices.value = [];
    throw error;
  } finally {
    loadingAuditItems.value = false;
  }
}

const getBorrowedProgressClass = (item: any) => {
  return {
    "bg-green-50 text-green-800": item.status === "ON_TIME",
    "bg-yellow-50 text-yellow-800": item.status === "NEAR_DUE",
    "bg-red-50 text-red-800": item.status === "OVERDUE",
    "bg-blue-50 text-blue-800": item.hasBeenReturned,
  };
};

const calculateReturnProgress = (date: string, hasBeenReturned: boolean) => {
  if (hasBeenReturned) {
    return "Đã trả";
  }

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

const getAuditStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    assessing: "Đang kiểm đếm",
    cancelled: "Đã hủy",
    completed: "Hoàn thành",
  };
  return statusMap[status] || status || "Đang kiểm đếm";
};

const getMaintenanceStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    maintaining: "Đang bảo trì",
    cancelled: "Đã hủy",
    completed: "Hoàn thành",
  };
  return statusMap[status] || status || "Đang bảo trì";
};

const getTransportStatusText = (status: string) => {
  const statusMap: Record<string, string> = {
    shipping: "Đang vận chuyển",
    cancelled: "Đã hủy",
    completed: "Hoàn thành",
  };
  return statusMap[status] || status || "Đang vận chuyển";
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

async function loadAccessoriesData(kindId: string, labId?: string) {
  if (!kindId) return;

  loadingAccessories.value = true;
  accessories.value = [];

  try {
    const accessoriesData = await searchService.getAccessoriesForDeviceKind(
      kindId,
      labId
    );
    accessories.value = accessoriesData;
    if (accessories.value.length > 0) {
      showAccessories.value = true;
    }
  } catch (err) {
    throw err;
  } finally {
    loadingAccessories.value = false;
  }
}

function retryLoading() {
  retrying.value = true;
  if (currentDeviceId.value) {
    loadDeviceDetailsById(currentDeviceId.value, "");
  }
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
});

const getUserBorrowStatusClass = (status: string) => {
  return {
    "bg-green-50 text-green-800": status === "ON_TIME",
    "bg-yellow-50 text-yellow-800": status === "NEAR_DUE",
    "bg-red-50 text-red-800": status === "OVERDUE",
  };
};

const getUserBorrowStatusText = (
  status: string,
  expectedReturnDate: string
) => {
  if (!expectedReturnDate) return "N/A";

  const dueDate = new Date(expectedReturnDate);
  const today = new Date();
  const daysLeft = Math.ceil(
    (dueDate.getTime() - today.getTime()) / (1000 * 60 * 60 * 24)
  );

  switch (status) {
    case "ON_TIME":
      return `Còn ${daysLeft} ngày`;
    case "NEAR_DUE":
      return `Sắp hết hạn (${daysLeft} ngày)`;
    case "OVERDUE":
      return `Quá hạn ${Math.abs(daysLeft)} ngày`;
    default:
      return "Không rõ";
  }
};

function toggleDeviceGroup(group: GroupedDevice) {
  group.expanded = !group.expanded;
}

const getActivityTypeClass = (type: string) => {
  return {
    "bg-green-50 text-green-800": type === "AUDIT",
    "bg-amber-50 text-amber-800": type === "MAINTENANCE",
    "bg-blue-50 text-blue-800": type === "TRANSPORT",
    "bg-purple-50 text-purple-800": type === "RETURNED",
  };
};

const getActivityTypeText = (type: string) => {
  switch (type) {
    case "AUDIT":
      return "Kiểm đếm";
    case "MAINTENANCE":
      return "Bảo trì";
    case "TRANSPORT":
      return "Vận chuyển";
    case "RETURNED":
      return "Mượn trả";
    default:
      return type;
  }
};

const getActivityStatusClass = (status: string) => {
  return {
    "bg-blue-50 text-blue-800": [
      "assessing",
      "maintaining",
      "shipping",
      "returned",
    ].includes(status),
    "bg-red-50 text-red-800": status === "cancelled",
    "bg-green-50 text-green-800": status === "completed",
  };
};

const getActivityStatusText = (status: string) => {
  switch (status) {
    case "returned":
      return "Đã trả";
    case "assessing":
      return "Đang kiểm đếm";
    case "maintaining":
      return "Đang bảo trì";
    case "shipping":
      return "Đang vận chuyển";
    case "cancelled":
      return "Đã hủy";
    case "completed":
      return "Hoàn thành";
    default:
      return status;
  }
};
</script>

<template>
  <div>
    <div v-if="mode === 'idle'" class="text-center py-12">
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
                      <span class="whitespace-nowrap">MƯỢN TRẢ</span>
                      <span
                        class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                      >
                        {{ borrowedDevices.length || 0 }}
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
                </TabsList>
              </div>
              <TabsContent
                value="inventory"
                class="p-4 h-[calc(100vh-10rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingInventory">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="inventory && inventory.length">
                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                            rowspan="1"
                          >
                            PHÒNG THÍ NGHIỆM
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Tốt
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Hư
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Loại bỏ
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-center text-sm font-semibold text-blue-900 border"
                          >
                            Đã mất
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in inventory"
                          :key="item.location"
                          class="hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            {{ item.location }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ item.healthy }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ item.broken }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ item.discarded }}
                          </td>
                          <td class="px-4 py-3 text-sm text-center border">
                            {{ item.lost }}
                          </td>
                        </tr>
                        <tr class="bg-gray-50 font-medium">
                          <td class="px-4 py-3 text-sm border font-semibold">
                            TỔNG CỘNG
                          </td>
                          <td
                            class="px-4 py-3 text-sm text-center border font-semibold"
                          >
                            {{
                              inventory.reduce(
                                (sum, item) =>
                                  sum + (parseInt(item.healthy) || 0),
                                0
                              )
                            }}
                          </td>
                          <td
                            class="px-4 py-3 text-sm text-center border font-semibold"
                          >
                            {{
                              inventory.reduce(
                                (sum, item) =>
                                  sum + (parseInt(item.broken) || 0),
                                0
                              )
                            }}
                          </td>
                          <td
                            class="px-4 py-3 text-sm text-center border font-semibold"
                          >
                            {{
                              inventory.reduce(
                                (sum, item) =>
                                  sum + (parseInt(item.discarded) || 0),
                                0
                              )
                            }}
                          </td>
                          <td
                            class="px-4 py-3 text-sm text-center border font-semibold"
                          >
                            {{
                              inventory.reduce(
                                (sum, item) => sum + (parseInt(item.lost) || 0),
                                0
                              )
                            }}
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
                class="p-4 h-[calc(100vh-10rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingBorrowedItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="borrowedDevices && borrowedDevices.length">
                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr class="whitespace-nowrap">
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGƯỜI MƯỢN
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY MƯỢN
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            HẠN TRẢ
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TRẠNG THÁI
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            GHI CHÚ
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in borrowedDevices"
                          :key="item.id"
                          class="whitespace-nowrap hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center gap-2">
                              <div
                                class="relative flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full overflow-hidden"
                              >
                                <img
                                  :src="item.borrower?.avatar || 'User Avatar'"
                                  :alt="item.borrower?.name || 'User'"
                                  class="w-full h-full object-cover"
                                />
                              </div>
                              <div>
                                <span class="font-medium">{{
                                  item.borrower?.name || "N/A"
                                }}</span>
                                <div class="text-xs text-gray-500">
                                  {{ item.borrower?.id || "" }} |
                                  {{ item.borrower?.email || "" }}
                                </div>
                              </div>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.borrowDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.expectedReturnedAt) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full"
                              :class="getBorrowedProgressClass(item)"
                            >
                              {{
                                calculateReturnProgress(
                                  item.expectedReturnedAt,
                                  item.hasBeenReturned
                                )
                              }}
                            </span>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.returnedNote || "—" }}
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
                value="audit"
                class="p-4 h-[calc(100vh-10rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingAuditItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="auditDevices && auditDevices.length">
                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr class="whitespace-nowrap">
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGƯỜI KIỂM ĐẾM
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY KIỂM ĐẾM
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            KẾT QUẢ
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            GHI CHÚ
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in auditDevices"
                          :key="item.id"
                          class="whitespace-nowrap hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center gap-2">
                              <div
                                class="relative flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full overflow-hidden"
                              >
                                <img
                                  :src="item.auditor?.avatar || 'User Avatar'"
                                  :alt="item.auditor?.name || 'Auditor'"
                                  class="w-full h-full object-cover"
                                />
                              </div>
                              <div>
                                <span class="font-medium">{{
                                  item.auditor?.name || "N/A"
                                }}</span>
                                <div class="text-xs text-gray-500">
                                  {{ item.auditor?.id || "" }} |
                                  {{ item.auditor?.email || "" }}
                                </div>
                              </div>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.auditDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full"
                              :class="{
                                'bg-yellow-50 text-yellow-800':
                                  item.auditResult === 'assessing',
                                'bg-red-50 text-red-800':
                                  item.auditResult === 'cancelled',
                                'bg-green-50 text-green-800':
                                  item.auditResult === 'completed',
                              }"
                            >
                              {{ getAuditStatusText(item.auditResult) }}
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
              <TabsContent
                value="maintenance"
                class="p-4 h-[calc(100vh-10rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingMaintenanceItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div
                  v-else-if="maintenanceDevices && maintenanceDevices.length"
                >
                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr class="whitespace-nowrap">
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            KỸ THUẬT VIÊN
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY BẮT ĐẦU
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            DỰ KIẾN HOÀN THÀNH
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TRẠNG THÁI
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            LÝ DO
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in maintenanceDevices"
                          :key="item.id"
                          class="whitespace-nowrap hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            <div class="flex items-center gap-2">
                              <div
                                class="relative flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full overflow-hidden"
                              >
                                <img
                                  :src="
                                    item.technician?.avatar || 'User Avatar'
                                  "
                                  :alt="item.technician?.name || 'Technician'"
                                  class="w-full h-full object-cover"
                                />
                              </div>
                              <div>
                                <span class="font-medium">{{
                                  item.technician?.name || "N/A"
                                }}</span>
                                <div class="text-xs text-gray-500">
                                  {{ item.technician?.id || "" }} |
                                  {{ item.technician?.email || "" }}
                                </div>
                              </div>
                            </div>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.maintenanceStartDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.expectedCompletionDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full"
                              :class="{
                                'bg-yellow-50 text-yellow-800':
                                  item.status === 'maintaining',
                                'bg-red-50 text-red-800':
                                  item.status === 'cancelled',
                                'bg-green-50 text-green-800':
                                  item.status === 'completed',
                              }"
                            >
                              {{ getMaintenanceStatusText(item.status) }}
                            </span>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.maintenanceReason || "Bảo trì định kỳ" }}
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
                class="p-4 h-[calc(100vh-10rem)] overflow-y-auto mt-0"
              >
                <div v-if="loadingTransportItems">
                  <div class="flex justify-center items-center p-8">
                    <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                  </div>
                </div>
                <div v-else-if="transportDevices && transportDevices.length">
                  <div class="overflow-x-auto">
                    <table class="min-w-full divide-y divide-gray-200 border">
                      <thead>
                        <tr class="whitespace-nowrap">
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TỪ ĐỊA ĐIỂM
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            ĐẾN ĐỊA ĐIỂM
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGƯỜI XÁC NHẬN VẬN CHUYỂN
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGƯỜI XÁC NHẬN NHẬN VỀ
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            NGÀY VẬN CHUYỂN
                          </th>
                          <th
                            class="bg-blue-50 p-2 text-left text-sm font-semibold text-blue-900 border"
                          >
                            TRẠNG THÁI
                          </th>
                        </tr>
                      </thead>
                      <tbody class="divide-y divide-gray-200 bg-white">
                        <tr
                          v-for="item in transportDevices"
                          :key="item.id"
                          class="whitespace-nowrap hover:bg-gray-50"
                        >
                          <td class="px-4 py-3 text-sm border">
                            {{ item.sourceLocation || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ item.destinationLocation || "N/A" }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <div
                              class="flex items-center gap-2"
                              v-if="item.sender"
                            >
                              <div
                                class="relative flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full overflow-hidden"
                              >
                                <img
                                  :src="item.sender?.avatar || 'User Avatar'"
                                  :alt="item.sender?.name || 'Sender'"
                                  class="w-full h-full object-cover"
                                />
                              </div>
                              <div>
                                <span class="font-medium">{{
                                  item.sender?.name || "N/A"
                                }}</span>
                                <div class="text-xs text-gray-500">
                                  {{ item.sender?.id || "" }} |
                                  {{ item.sender?.email || "" }}
                                </div>
                              </div>
                            </div>
                            <span v-else>—</span>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <div
                              class="flex items-center gap-2"
                              v-if="item.receiver"
                            >
                              <div
                                class="relative flex-shrink-0 w-8 h-8 bg-gray-100 rounded-full overflow-hidden"
                              >
                                <img
                                  :src="item.receiver?.avatar || 'User Avatar'"
                                  :alt="item.receiver?.name || 'Receiver'"
                                  class="w-full h-full object-cover"
                                />
                              </div>
                              <div>
                                <span class="font-medium">{{
                                  item.receiver?.name || "N/A"
                                }}</span>
                                <div class="text-xs text-gray-500">
                                  {{ item.receiver?.id || "" }} |
                                  {{ item.receiver?.email || "" }}
                                </div>
                              </div>
                            </div>
                            <span v-else>—</span>
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            {{ formatDate(item.transportDate) }}
                          </td>
                          <td class="px-4 py-3 text-sm border">
                            <span
                              class="text-xs px-2 py-1 rounded-full"
                              :class="{
                                'bg-yellow-50 text-yellow-800':
                                  item.status === 'shipping',
                                'bg-red-50 text-red-800':
                                  item.status === 'cancelled',
                                'bg-green-50 text-green-800':
                                  item.status === 'completed',
                              }"
                            >
                              {{ getTransportStatusText(item.status) }}
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
            </Tabs>
          </div>

          <div
            class="bg-white h-[calc(100vh-6rem)] overflow-y-auto rounded-lg shadow-sm border border-gray-200"
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
                    :src="deviceDetail.image?.mainImage"
                    :alt="deviceDetail.deviceName"
                    class="w-full h-full object-cover"
                  />
                </div>

                <div class="flex-grow">
                  <div class="text-sm text-gray-500 mb-1">
                    Mã thiết bị: {{ deviceDetail.id }}
                  </div>
                  <h1 class="text-xl font-medium text-gray-900">
                    {{ deviceDetail.deviceName }}
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
                        : "N/A"
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
                          /*deviceDetail.allowedBorrowRoles?.join(", ") ||*/ "Sinh viên, Giảng viên"
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
                  <div
                    v-if="loadingAccessories"
                    class="flex justify-center items-center py-4"
                  >
                    <LoaderIcon class="animate-spin h-6 w-6 text-gray-400" />
                  </div>
                  <div
                    v-else-if="accessories.length === 0"
                    class="text-sm text-gray-500"
                  >
                    Không có dụng cụ đi kèm
                  </div>
                  <div v-else class="space-y-4">
                    <div
                      v-for="accessory in accessories"
                      :key="accessory.id"
                      class="flex items-center space-x-3 border-b border-gray-100 pb-3"
                    >
                      <div
                        class="flex-shrink-0 w-12 h-12 bg-gray-100 border rounded flex items-center justify-center overflow-hidden"
                      >
                        <img
                          v-if="accessory.image"
                          :src="
                            accessory.image?.mainImage || '/placeholder.svg'
                          "
                          :alt="accessory.name"
                          class="w-full h-full object-contain"
                        />
                        <PackageIcon v-else class="h-6 w-6 text-gray-400" />
                      </div>
                      <div class="flex-1">
                        <div class="text-sm font-medium text-gray-900">
                          {{ accessory.name }}
                        </div>
                        <div class="text-xs text-gray-500">
                          {{ accessory.brand || "Không có thương hiệu" }}
                        </div>
                      </div>
                      <div class="text-sm font-medium text-gray-900">
                        x {{ accessory.quantity }} {{ accessory.unit || "EA" }}
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

    <div v-if="mode === 'user' && userInfo">
      <div class="grid grid-cols-3 gap-6">
        <div
          class="col-span-2 bg-white rounded-lg shadow-sm border border-gray-200"
        >
          <Tabs v-model="userActiveTab" class="w-full">
            <div class="border-b border-gray-200">
              <TabsList class="bg-transparent p-0 w-full flex">
                <TabsTrigger
                  value="borrowed"
                  class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                >
                  <div class="flex items-center justify-center gap-1 w-full">
                    <div class="rounded-full bg-violet-50 p-1">
                      <PackageIcon class="h-4 w-4 text-violet-600" />
                    </div>
                    <span class="whitespace-nowrap">THIẾT BỊ ĐANG MƯỢN</span>
                    <span
                      class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                    >
                      {{ groupedBorrowedDevices.length || 0 }}
                    </span>
                  </div>
                </TabsTrigger>
                <TabsTrigger
                  value="activities"
                  class="text-xs flex-1 px-4 py-3 rounded-none border-b-2 data-[state=active]:border-blue-500 data-[state=active]:shadow-none data-[state=active]:bg-transparent data-[state=active]:text-blue-600"
                >
                  <div class="flex items-center justify-center gap-1 w-full">
                    <div class="rounded-full bg-emerald-50 p-1">
                      <ClipboardCheckIcon class="h-4 w-4 text-emerald-600" />
                    </div>
                    <span class="whitespace-nowrap">CÁC HOẠT ĐỘNG KHÁC</span>
                    <span
                      class="ml-auto px-2 py-0.5 bg-blue-100 text-blue-800 text-xs rounded-full min-w-[20px] text-center"
                    >
                      {{ userActivities.length || 0 }}
                    </span>
                  </div>
                </TabsTrigger>
              </TabsList>
            </div>

            <div class="h-[calc(100vh-10rem)] overflow-y-auto">
              <TabsContent value="borrowed" class="p-4 mt-0">
                <div
                  v-if="loadingUserBorrowedItems"
                  class="flex justify-center items-center p-8"
                >
                  <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                </div>
                <div
                  v-else-if="groupedBorrowedDevices.length > 0"
                  class="divide-y divide-gray-200"
                >
                  <div
                    v-for="group in groupedBorrowedDevices"
                    :key="group.kindId"
                    class="divide-y divide-gray-100"
                  >
                    <div
                      class="p-4 hover:bg-gray-50 cursor-pointer"
                      @click="toggleDeviceGroup(group)"
                    >
                      <div class="grid grid-cols-12 items-center">
                        <div class="col-span-10 flex items-center gap-3">
                          <img
                            :src="group.image?.mainImage || '/placeholder.svg'"
                            alt="Device image"
                            class="h-12 w-12 rounded-full object-cover"
                          />
                          <div>
                            <div class="flex items-center gap-2 mb-0.5">
                              <h3 class="font-medium text-gray-900 text-sm">
                                Mã loại:
                                <span class="font-bold text-base">{{
                                  group.kindId
                                }}</span>
                              </h3>
                              <Badge
                                v-if="group.deviceBorrowableLabOnly"
                                variant="outline"
                                class="text-blue-600 border-blue-200 bg-blue-50 text-xs"
                              >
                                Không mượn về
                              </Badge>
                            </div>
                            <p class="text-base text-gray-900 font-medium">
                              {{ group.deviceName }}
                            </p>
                          </div>
                        </div>
                        <div class="col-span-2 flex items-center">
                          <span
                            class="text-base text-gray-900 font-medium w-full"
                          >
                            SL: {{ group.quantity }}
                          </span>
                          <ChevronDownIcon
                            class="h-5 w-5 text-gray-400 transition-transform justify-self-end"
                            :class="{ 'rotate-180': group.expanded }"
                          />
                        </div>
                      </div>
                    </div>

                    <div v-if="group.expanded" class="bg-gray-50">
                      <div
                        class="grid grid-cols-12 px-4 py-2 text-sm font-medium text-gray-500 border-b border-gray-200"
                      >
                        <div class="col-span-5">THIẾT BỊ GHI NHẬN</div>
                        <div class="col-span-5">TIẾN ĐỘ TRẢ</div>
                        <div class="col-span-2">HẸN TRẢ</div>
                      </div>
                      <div
                        v-for="item in group.items"
                        :key="item.receiptId"
                        class="grid grid-cols-12 items-center px-4 py-3 border-b border-gray-100 last:border-b-0"
                      >
                        <div
                          class="col-span-5 text-sm font-medium text-gray-900"
                        >
                          {{ group.kindId }}/{{ item.deviceId }}
                        </div>
                        <div
                          class="col-span-5 text-sm"
                          :class="
                            item.status === 'OVERDUE'
                              ? 'text-red-600'
                              : 'text-gray-600'
                          "
                        >
                          <span
                            :class="getUserBorrowStatusClass(item.status)"
                            class="px-2 py-0.5 rounded-full text-xs font-medium"
                          >
                            {{
                              getUserBorrowStatusText(
                                item.status,
                                item.expectedReturnedAt
                              )
                            }}
                          </span>
                        </div>
                        <div class="col-span-2 text-sm text-gray-600">
                          {{ formatDate(item.expectedReturnedAt) || "---" }}
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div
                  v-else
                  class="flex flex-col items-center justify-center py-20 text-center"
                >
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <PackageIcon class="h-6 w-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Người dùng này chưa mượn thiết bị nào.
                  </p>
                </div>
              </TabsContent>

              <TabsContent value="activities" class="p-4 mt-0">
                <div
                  v-if="loadingUserActivities"
                  class="flex justify-center items-center p-8"
                >
                  <LoaderIcon class="animate-spin h-8 w-8 text-gray-400" />
                </div>
                <div
                  v-else-if="userActivities.length > 0"
                  class="space-y-4 relative before:absolute before:inset-0 before:left-9 before:ml-0.5 before:border-l-2 before:border-gray-200"
                >
                  <div
                    v-for="activity in userActivities"
                    :key="activity.id"
                    class="relative pl-10"
                  >
                    <div
                      class="absolute left-0 top-4 flex h-7 w-7 items-center justify-center rounded-full bg-white border border-gray-200 shadow"
                    >
                      <div
                        class="rounded-full p-1.5"
                        :class="{
                          'bg-green-50': activity.type === 'AUDIT',
                          'bg-amber-50': activity.type === 'MAINTENANCE',
                          'bg-blue-50': activity.type === 'TRANSPORT',
                          'bg-purple-50': activity.type === 'RETURNED',
                        }"
                      >
                        <ClipboardCheckIcon
                          v-if="activity.type === 'AUDIT'"
                          class="h-3 w-3 text-green-600"
                        />
                        <WrenchIcon
                          v-else-if="activity.type === 'MAINTENANCE'"
                          class="h-3 w-3 text-amber-600"
                        />
                        <TruckIcon
                          v-else-if="activity.type === 'TRANSPORT'"
                          class="h-3 w-3 text-blue-600"
                        />
                        <CheckCircleIcon
                          v-else-if="activity.type === 'RETURNED'"
                          class="h-3 w-3 text-purple-600"
                        />
                      </div>
                    </div>

                    <div
                      class="bg-white p-3 border border-gray-100 shadow-sm hover:bg-gray-50 transition-colors"
                    >
                      <div class="grid grid-cols-12 gap-3">
                        <div class="col-span-7 flex gap-3">
                          <img
                            :src="
                              activity.deviceImage?.mainImage || 'Device Image'
                            "
                            alt="Device image"
                            class="h-16 w-16 rounded-md object-cover border border-gray-200 flex-shrink-0"
                          />
                          <div class="flex-grow">
                            <div
                              class="text-sm font-medium text-gray-900 mb-0.5"
                            >
                              {{ activity.deviceName }}
                            </div>
                            <div class="text-xs text-gray-500 mb-1">
                              <span class="font-medium">Mã thiết bị:</span>
                              {{ activity.deviceKindId }}/{{
                                activity.deviceId
                              }}
                            </div>
                            <div class="text-xs text-gray-600">
                              <span class="font-medium">Địa điểm:</span>
                              {{ activity.location }}
                            </div>
                            <div
                              v-if="activity.note"
                              class="mt-1 text-xs text-gray-600 italic"
                            >
                              <span class="font-medium">Ghi chú:</span>
                              {{ activity.note }}
                            </div>
                          </div>
                        </div>

                        <div class="col-span-5 flex flex-col justify-between">
                          <div class="flex flex-wrap gap-1.5 mb-2">
                            <span
                              class="text-xs font-medium px-2 py-0.5 rounded-full"
                              :class="getActivityTypeClass(activity.type)"
                            >
                              {{ getActivityTypeText(activity.type) }}
                            </span>
                            <span
                              class="text-xs font-medium px-2 py-0.5 rounded-full"
                              :class="getActivityStatusClass(activity.status)"
                            >
                              {{ getActivityStatusText(activity.status) }}
                            </span>
                          </div>

                          <div
                            class="flex items-center text-xs text-gray-500 mt-auto"
                          >
                            <ClockIcon class="mr-1 h-3 w-3" />
                            <time :datetime="activity.date">
                              {{ formatDate(activity.date) }}
                            </time>
                          </div>

                          <div class="mt-2 text-xs">
                            <div
                              v-if="activity.type === 'AUDIT'"
                              class="text-gray-600"
                            >
                              <span class="font-medium">Kiểm đếm:</span> Kiểm
                              thường niên
                            </div>
                            <div
                              v-if="activity.type === 'MAINTENANCE'"
                              class="text-gray-600"
                            >
                              <span class="font-medium">Bảo trì:</span>
                              {{
                                activity.status === "completed"
                                  ? "Hoàn thành"
                                  : "Đang thực hiện"
                              }}
                            </div>
                            <div
                              v-if="activity.type === 'TRANSPORT'"
                              class="text-gray-600"
                            >
                              <span class="font-medium">Vận chuyển:</span>
                              {{
                                activity.status === "completed"
                                  ? "Đã nhận"
                                  : "Đang giao"
                              }}
                            </div>
                            <div
                              v-if="activity.type === 'RETURNED'"
                              class="text-gray-600"
                            >
                              <span class="font-medium">Trạng thái:</span>
                              {{
                                activity.prevQuality === activity.afterQuality
                                  ? "Trả đúng chất lượng"
                                  : "Chất lượng đã thay đổi"
                              }}
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
                <div
                  v-else
                  class="flex flex-col items-center justify-center py-20 text-center"
                >
                  <div class="rounded-full bg-gray-100 p-3 mb-3">
                    <ClipboardCheckIcon class="h-6 w-6 text-gray-400" />
                  </div>
                  <p class="text-sm text-gray-500">
                    Không có hoạt động nào được ghi nhận.
                  </p>
                </div>
              </TabsContent>
            </div>
          </Tabs>
        </div>

        <div
          class="bg-white h-[calc(100vh-6rem)] overflow-y-auto rounded-lg shadow-sm border border-gray-200"
        >
          <div class="border-b border-gray-200 p-4">
            <h2 class="text-xl font-semibold text-gray-700">
              THÔNG TIN NGƯỜI DÙNG
            </h2>
          </div>

          <div>
            <div class="space-y-4 bg-gray-50 rounded-lg p-2">
              <div class="rounded-lg px-4 py-1">
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

            <div class="space-y-4 p-4 border-t border-gray-200">
              <div class="grid grid-cols-[100px_1fr]">
                <div class="text-sm text-gray-500">Tài khoản</div>
                <div class="text-sm text-green-600">Hoạt động</div>
              </div>

              <div class="grid grid-cols-[100px_1fr]">
                <div class="text-sm text-gray-500">Tình trạng</div>
                <div class="text-sm text-red-600">
                  Cấm hoạt động (đến 05/05/2025)
                </div>
              </div>

              <div class="border rounded-md">
                <div
                  class="p-4 flex justify-between items-center cursor-pointer"
                  @click="showUserDetails = !showUserDetails"
                >
                  <h3 class="text-base text-blue-600 font-medium">Chi tiết</h3>
                  <ChevronDownIcon
                    v-if="!showUserDetails"
                    class="h-5 w-5 text-blue-500"
                  />
                  <ChevronUpIcon v-else class="h-5 w-5 text-blue-500" />
                </div>

                <div v-if="showUserDetails" class="p-4 border-t">
                  <div class="space-y-3">
                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Nguyên nhân</div>
                      <div class="flex-1 text-sm text-gray-900">
                        Làm mất thiết bị
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Môn đang học</div>
                      <div class="flex-1 text-sm text-gray-900">
                        Hệ thống số<br />
                        Điện - Điện tử<br />
                        Thiết kế vi mạch
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">Môn đã học</div>
                      <div class="flex-1 text-sm text-gray-900">
                        Hệ thống số
                      </div>
                    </div>

                    <div class="flex">
                      <div class="w-32 text-sm text-gray-500">
                        Phòng có phép
                      </div>
                      <div class="flex-1 text-sm text-gray-900">
                        605 H6, Dĩ An<br />
                        601 H6, Dĩ An
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div class="px-4 pb-4">
              <Button class="w-full bg-blue-600 hover:bg-blue-700">
                Cho phép mượn
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
