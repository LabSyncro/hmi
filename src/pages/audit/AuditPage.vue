<script setup lang="ts">
import {
  BoxIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  ClipboardCheckIcon,
  ListIcon,
  LoaderIcon,
  MapPinIcon,
  PackageIcon,
  TrashIcon,
  UserIcon,
  XIcon,
} from "lucide-vue-next";

interface PendingDevice {
  id: string;
  status: (typeof DeviceStatus)[keyof typeof DeviceStatus];
  auditCondition?: (typeof DeviceStatus)[keyof typeof DeviceStatus];
}

const mode = ref<"idle" | "audit">("idle");
const pendingDevices = ref<PendingDevice[]>([]);

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
const isLoadingDeviceScan = ref(false);
const isLoadingUser = ref(false);
const showSuccessModal = ref(false);
const successMessage = ref("");
const auditId = ref("");

const { verifyScannedQrCode } = useOneTimeQR();

const validStatuses = [
  DeviceStatus.HEALTHY,
  DeviceStatus.BROKEN,
  DeviceStatus.DISCARDED,
  DeviceStatus.LOST,
];

const showIncompleteAuditsModal = ref(false);
const incompleteAudits = ref<IncompleteAudit[]>([]);
const selectedAudit = ref<IncompleteAudit | null>(null);
const loadingIncompleteAudits = ref(false);
const continueMode = ref(false);

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

  isLoadingUser.value = true;
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
    isLoadingUser.value = false;
    isConfirming.value = false;
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

    try {
      const deviceDetails = await deviceService.getDeviceAuditById(
        deviceId,
        storedUserInfo.value?.lab.id || ""
      )!;

      if (deviceDetails.currentStatus === DeviceStatus.ASSESSING) {
        toast({
          title: "Lỗi",
          description: "Thiết bị đang trong quá trình kiểm đếm khác",
          variant: "destructive",
        });
        return;
      }

      if (!validStatuses.includes(deviceDetails.status)) {
        toast({
          title: "Lỗi",
          description: `Tình trạng thiết bị không hợp lệ: ${statusMap[deviceDetails.status]}`,
          variant: "destructive",
        });
        return;
      }

      const inventoryData = await deviceService.getDeviceInventoryInAudit(
        deviceKindId!,
        storedUserInfo.value?.lab.id || ""
      );
      const expectedQuantity = inventoryData.availableQuantity || 0;
      const unscannedDeviceIds = inventoryData.unscannedDeviceIds || [];

      if (mode.value === "idle") {
        if (userInfo.value && storedUserInfo.value?.lab?.id) {
          try {
            const auditResult = await auditService.createAudit({
              auditorId: userInfo.value.id,
              location: storedUserInfo.value.lab.id,
              devices: [
                ...pendingDevices.value.map((pd) => ({
                  id: pd.id,
                  condition: pd.auditCondition || DeviceStatus.HEALTHY,
                  prevStatus: pd.status,
                })),
                {
                  id: deviceId,
                  condition:
                    deviceDetails.auditCondition || DeviceStatus.HEALTHY,
                  prevStatus: deviceDetails.status,
                },
              ],
              status: AssessmentStatus.ASSESSING,
            });

            auditId.value = auditResult.id;
            mode.value = "audit";
            pendingDevices.value = [];
          } catch (error) {
            toast({
              title: "Lỗi",
              description: "Không thể tạo phiên kiểm đếm mới",
              variant: "destructive",
            });
            return;
          }
        } else {
          pendingDevices.value.push({
            id: deviceId,
            status: deviceDetails.status,
            auditCondition:
              deviceDetails.auditCondition || DeviceStatus.HEALTHY,
          });

          const existingDeviceIndex = devices.value.findIndex(
            (d) => d.code === deviceKindId
          );

          if (existingDeviceIndex === -1) {
            devices.value.push({
              code: deviceKindId!,
              name: deviceDetails.deviceName || "Unknown",
              image: deviceDetails.image || "",
              quantity: 1,
              unit: deviceDetails.unit || "Cái",
              isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly || false,
              expectedQuantity: expectedQuantity,
              items: [
                {
                  id: deviceId,
                  status: deviceDetails.status,
                  auditCondition:
                    deviceDetails.auditCondition || DeviceStatus.HEALTHY,
                },
              ],
              expanded: true,
              unscannedDeviceIds: unscannedDeviceIds.filter(
                (id) => id !== deviceId
              ),
              unscannedCondition: DeviceStatus.LOST,
            });
          } else {
            devices.value[existingDeviceIndex].items.push({
              id: deviceId,
              status: deviceDetails.status,
              auditCondition:
                deviceDetails.auditCondition || DeviceStatus.HEALTHY,
            });
            const unscannedIndex =
              devices.value[existingDeviceIndex].unscannedDeviceIds?.indexOf(
                deviceId
              );
            if (unscannedIndex !== undefined && unscannedIndex > -1) {
              devices.value[existingDeviceIndex].unscannedDeviceIds?.splice(
                unscannedIndex,
                1
              );
            }
          }

          updateAuditCounts();

          toast({
            title: "Thiết bị đã được thêm",
            description: "Vui lòng quét mã người dùng để bắt đầu kiểm đếm",
            variant: "success",
          });
          return;
        }
      } else if (mode.value === "audit") {
        try {
          const currentAuditId =
            continueMode.value && selectedAudit.value
              ? selectedAudit.value.id
              : auditId.value;

          await auditService.addDeviceToAudit(
            currentAuditId,
            deviceId,
            deviceDetails.status,
            deviceDetails.auditCondition
          );
        } catch (error) {
          toast({
            title: "Lỗi",
            description:
              "Không thể cập nhật thông tin thiết bị vào phiên kiểm đếm",
            variant: "destructive",
          });
          return;
        }
      }

      const newItem: AuditDeviceItem = {
        id: deviceId,
        status: deviceDetails.status,
        auditCondition: deviceDetails.auditCondition || DeviceStatus.HEALTHY,
      };

      const existingDevice = devices.value.find((d) => d.code === deviceKindId);
      if (existingDevice) {
        if (!existingDevice.items.some((item) => item.id === deviceId)) {
          existingDevice.items.push(newItem);
          existingDevice.quantity = existingDevice.items.length;
          existingDevice.unscannedDeviceIds =
            existingDevice.unscannedDeviceIds.filter(
              (id: string) => id !== deviceId
            );
        }
      } else {
        const newDevice: AuditDevice = {
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
          unscannedDeviceIds: unscannedDeviceIds.filter(
            (id: string) => id !== deviceId
          ),
          unscannedItemConditions: {},
        };
        devices.value.push(newDevice);
      }

      updateAuditCounts();

      toast({
        title: "Thành công",
        description: "Đã thêm thiết bị vào danh sách kiểm đếm",
        variant: "success",
      });
    } catch (error) {
      if (error instanceof Error) {
        if (
          error.message === "Device not found" ||
          error.message === "Device inventory not found"
        ) {
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
        } else if (
          error.message === "Missing device ID or lab ID" ||
          error.message === "Missing kind ID or lab ID"
        ) {
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

const updateAuditCounts = () => {
  let good = 0;
  let damaged = 0;
  let missing = 0;
  let discarded = 0;

  devices.value.forEach((device) => {
    device.items.forEach((item) => {
      if (item.auditCondition === DeviceStatus.HEALTHY) good++;
      else if (item.auditCondition === DeviceStatus.BROKEN) damaged++;
      else if (item.auditCondition === DeviceStatus.LOST) missing++;
      else if (item.auditCondition === DeviceStatus.DISCARDED) discarded++;
    });

    const unscannedItems = getUnscannedItems(device);
    unscannedItems.forEach((item) => {
      if (item.auditCondition === DeviceStatus.HEALTHY) good++;
      else if (item.auditCondition === DeviceStatus.BROKEN) damaged++;
      else if (item.auditCondition === DeviceStatus.LOST) missing++;
      else if (item.auditCondition === DeviceStatus.DISCARDED) discarded++;
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

      if (pendingDevices.value.length > 0 && storedUserInfo.value?.lab?.id) {
        try {
          const auditResult = await auditService.createAudit({
            auditorId: user.id,
            location: storedUserInfo.value.lab.id,
            devices: pendingDevices.value.map((pd) => ({
              id: pd.id,
              condition: pd.auditCondition || DeviceStatus.HEALTHY,
              prevStatus: pd.status,
            })),
            status: AssessmentStatus.ASSESSING,
          });

          auditId.value = auditResult.id;
          mode.value = "audit";
          pendingDevices.value = [];

          toast({
            title: "Thành công",
            description: `Đã nhận diện: ${user.name} và bắt đầu kiểm đếm`,
            variant: "success",
          });
        } catch (error) {
          toast({
            title: "Lỗi",
            description: "Không thể tạo phiên kiểm đếm mới",
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

const toggleDevice = (device: AuditDevice) => {
  if (device.items.length > 0) {
    device.expanded = !device.expanded;
  }
};

const removeDeviceItem = async (device: AuditDevice, itemId: string) => {
  try {
    device.items = device.items.filter((item) => item.id !== itemId);
    device.quantity = device.items.length;

    if (device.items.length === 0) {
      devices.value = devices.value.filter((d) => d.code !== device.code);
    }

    if (devices.value.length === 0) {
      mode.value = "idle";
    }

    updateAuditCounts();

    if (mode.value === "audit") {
      const currentAuditId =
        continueMode.value && selectedAudit.value
          ? selectedAudit.value.id
          : auditId.value;

      await auditService.removeDeviceFromAudit(currentAuditId, itemId);
    }
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể xóa thiết bị khỏi phiên kiểm đếm",
      variant: "destructive",
    });
  }
};

const updateDeviceCondition = async (
  item: AuditDeviceItem,
  condition: (typeof DeviceStatus)[keyof typeof DeviceStatus]
) => {
  try {
    item.auditCondition = condition;

    const deviceWithItem = devices.value.find((d) =>
      d.unscannedDeviceIds?.includes(item.id)
    );

    if (deviceWithItem) {
      deviceWithItem.unscannedItemConditions = {
        ...deviceWithItem.unscannedItemConditions,
        [item.id]: condition,
      };
    } else if (mode.value === "audit") {
      const currentAuditId =
        continueMode.value && selectedAudit.value
          ? selectedAudit.value.id
          : auditId.value;

      await auditService.updateDeviceCondition(
        currentAuditId,
        item.id,
        condition
      );
    }

    updateAuditCounts();
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể cập nhật tình trạng thiết bị",
      variant: "destructive",
    });
  }
};

const goToHome = () => {
  showSuccessModal.value = false;
  mode.value = "idle";
  devices.value = [];
  notes.value = "";
  userInfo.value = null;
  router.push("/audit");
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
    let result;

    const auditToComplete =
      continueMode.value && selectedAudit.value
        ? selectedAudit.value.id
        : auditId.value;

    const scannedDeviceUpdates: {
      id: string;
      condition: (typeof DeviceStatus)[keyof typeof DeviceStatus];
    }[] = [];

    devices.value.forEach((device) => {
      device.items.forEach((item) => {
        scannedDeviceUpdates.push({
          id: item.id,
          condition: item.auditCondition || DeviceStatus.HEALTHY,
        });
      });
    });

    if (scannedDeviceUpdates.length > 0) {
      await auditService.updateListDeviceConditions(
        auditToComplete,
        scannedDeviceUpdates
      );
    }

    const unscannedItemsForBulk: {
      deviceId: string;
      condition: (typeof DeviceStatus)[keyof typeof DeviceStatus];
    }[] = [];

    devices.value.forEach((device) => {
      const unscannedItems = getUnscannedItems(device);

      if (unscannedItems.length > 0) {
        unscannedItems.forEach((item) => {
          unscannedItemsForBulk.push({
            deviceId: item.id,
            condition: item.auditCondition || DeviceStatus.LOST,
          });
        });
      }
    });

    if (unscannedItemsForBulk.length > 0) {
      await auditService.addUnscannedDevices(
        auditToComplete,
        unscannedItemsForBulk
      );
    }

    await auditService.completeAudit(auditToComplete, notes.value);
    result = { success: true, id: auditToComplete };

    successMessage.value = "Đã hoàn tất kiểm đếm thiết bị";
    auditId.value = result.id;
    showSuccessModal.value = true;
    continueMode.value = false;
    selectedAudit.value = null;
  } catch (error) {
    console.error(error);
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
  return Array.from({ length: Math.max(0, count) }).map((_, index) => {
    const id = device.unscannedDeviceIds[index];
    const storedCondition =
      device.unscannedItemConditions?.[id] ||
      device.unscannedCondition ||
      DeviceStatus.LOST;

    return {
      id: id,
      displayName: `${device.code} / ${id}`,
      status: DeviceStatus.LOST,
      auditCondition: storedCondition || DeviceStatus.LOST,
    };
  });
};

async function fetchIncompleteAudits() {
  loadingIncompleteAudits.value = true;
  try {
    if (!storedUserInfo.value?.lab?.id) {
      toast({
        title: "Lỗi",
        description: "Không tìm thấy thông tin phòng lab",
        variant: "destructive",
      });
      return;
    }
    const result = await auditService.getIncompleteAudits(
      storedUserInfo.value.lab.id
    );
    incompleteAudits.value = result as IncompleteAudit[];
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể tải danh sách kiểm đếm chưa hoàn thành",
      variant: "destructive",
    });
  } finally {
    loadingIncompleteAudits.value = false;
  }
}

function openIncompleteAuditsModal() {
  fetchIncompleteAudits();
  showIncompleteAuditsModal.value = true;
}

function clearAuditState() {
  devices.value = [];
  userInfo.value = null;
  mode.value = "idle";
  continueMode.value = false;
  pendingDevices.value = [];
  notes.value = "";
  auditId.value = "";
  selectedAudit.value = null;
}

async function cancelAudit(auditId: string) {
  try {
    await auditService.cancelAudit(auditId);

    clearAuditState();

    toast({
      title: "Thành công",
      description: "Đã hủy phiên kiểm đếm",
      variant: "success",
    });

    showIncompleteAuditsModal.value = false;
    await fetchIncompleteAudits();
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể hủy phiên kiểm đếm",
      variant: "destructive",
    });
  }
}

async function continueAudit() {
  if (!selectedAudit.value) {
    return;
  }
  loadingIncompleteAudits.value = true;
  try {
    const userMeta = await userService.getUserById(
      selectedAudit.value.accountantId
    );
    if (userMeta) {
      userInfo.value = {
        id: userMeta.id,
        name: userMeta.name,
        avatar: userMeta.avatar,
        roles: userMeta.roles,
      };
    }

    auditDetails.value.location = `${selectedAudit.value.labRoom}, ${selectedAudit.value.labBranch}`;

    mode.value = "audit";
    continueMode.value = true;

    showIncompleteAuditsModal.value = false;

    toast({
      title: "Thành công",
      description: `Tiếp tục kiểm đếm: ${selectedAudit.value.id}`,
      variant: "success",
    });
    const deviceIdList = Array.isArray(selectedAudit.value.deviceIds)
      ? [...selectedAudit.value.deviceIds].map((id) => String(id))
      : [];

    if (deviceIdList.length > 0) {
      await loadAuditDevices(deviceIdList);
    } else {
      toast({
        title: "Cảnh báo",
        description: "Không tìm thấy thiết bị nào trong phiên kiểm đếm này",
        variant: "destructive",
      });
    }
  } catch (error) {
    toast({
      title: "Lỗi",
      description: "Không thể tiếp tục kiểm đếm này",
      variant: "destructive",
    });
  } finally {
    loadingIncompleteAudits.value = false;
  }
}

async function loadAuditDevices(deviceIds: string[]) {
  isLoadingDeviceScan.value = true;
  try {
    devices.value = [];

    for (const deviceId of deviceIds) {
      try {
        const deviceDetails = await deviceService.getDeviceAuditById(
          deviceId,
          selectedAudit.value?.labId || ""
        );
        auditId.value = selectedAudit.value?.id || "";

        const kindId = deviceDetails.kind;

        if (!validStatuses.includes(deviceDetails.status)) {
          continue;
        }

        const inventoryData = await deviceService.getDeviceInventoryInAudit(
          kindId,
          selectedAudit.value?.labId || ""
        );

        const expectedQuantity = inventoryData.availableQuantity || 0;
        const unscannedDeviceIds = inventoryData.unscannedDeviceIds || [];

        const newItem: AuditDeviceItem = {
          id: deviceId,
          status: deviceDetails.status,
          auditCondition: deviceDetails.auditCondition || DeviceStatus.HEALTHY,
        };

        const existingDevice = devices.value.find((d) => d.code === kindId);
        if (existingDevice) {
          if (!existingDevice.items.some((item) => item.id === deviceId)) {
            existingDevice.items.push(newItem);
            existingDevice.quantity = existingDevice.items.length;
            existingDevice.unscannedDeviceIds =
              existingDevice.unscannedDeviceIds.filter(
                (id: string) => id !== deviceId
              );
          }
        } else {
          const newDevice: AuditDevice = {
            code: kindId,
            name: deviceDetails.deviceName,
            image: deviceDetails.image,
            quantity: 1,
            unit: deviceDetails.unit,
            expanded: true,
            items: [newItem],
            isBorrowableLabOnly: deviceDetails.isBorrowableLabOnly || false,
            expectedQuantity,
            unscannedCondition: DeviceStatus.LOST,
            unscannedDeviceIds: unscannedDeviceIds.filter(
              (id: string) => id !== deviceId
            ),
            unscannedItemConditions: {},
          };
          devices.value.push(newDevice);
        }
      } catch (error) {
        toast({
          title: "Lỗi",
          description: `Không thể tải thông tin thiết bị ${deviceId}`,
          variant: "destructive",
        });
      }
    }

    updateAuditCounts();

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
      description: "Không thể tải thông tin thiết bị cho kiểm đếm này",
      variant: "destructive",
    });
  } finally {
    isLoadingDeviceScan.value = false;
  }
}

const updateDeviceUnscannedItems = (device: AuditDevice) => {
  if (!device.unscannedCondition) return;

  const unscannedItems = getUnscannedItems(device);
  unscannedItems.forEach((item) => {
    if (device.unscannedCondition) {
      item.auditCondition = device.unscannedCondition;
      updateDeviceCondition(item, device.unscannedCondition);
    }
  });

  updateAuditCounts();
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
        <div
          class="p-4 border-b border-gray-200 flex justify-between items-center"
        >
          <h2 class="text-sm font-semibold flex items-center gap-1">
            <ClipboardCheckIcon class="h-4 w-4" />
            DANH SÁCH GHI NHẬN
          </h2>

          <Button
            variant="outline"
            size="sm"
            class="flex items-center gap-1 text-blue-600 border-blue-200 hover:bg-blue-50 text-xs py-1 px-2"
            @click="openIncompleteAuditsModal"
          >
            <ListIcon class="h-3 w-3" />
            Kiểm đếm chưa hoàn thành
          </Button>
        </div>

        <div class="h-[calc(100vh-10rem)] overflow-y-auto">
          <div
            v-if="devices.length === 0"
            class="flex flex-col items-center justify-center py-12 text-center"
          >
            <div class="rounded-full bg-gray-100 p-2 mb-3">
              <LoaderIcon
                v-if="isLoadingDeviceScan"
                class="h-5 w-5 text-blue-500 animate-spin"
              />
              <PackageIcon v-else class="h-5 w-5 text-gray-400" />
            </div>
            <p class="text-xs text-gray-500">
              {{
                isLoadingDeviceScan
                  ? "Đang xử lý thiết bị..."
                  : "Quét mã QR thiết bị để ghi nhận kiểm đếm thiết bị"
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
                class="p-3 hover:bg-gray-50 cursor-pointer"
                @click="toggleDevice(device)"
              >
                <div class="grid grid-cols-10 items-center">
                  <div class="flex items-center col-span-7 gap-2">
                    <img
                      :src="device.image.mainImage"
                      alt="Device image"
                      class="h-10 w-10 rounded-full object-cover"
                    />
                    <div>
                      <div class="flex items-center gap-1 mb-0.5">
                        <h3 class="font-medium text-gray-900 text-xs">
                          Mã loại:
                          <span class="font-bold text-sm whitespace-nowrap">{{
                            device.code
                          }}</span>
                        </h3>
                        <Badge
                          v-if="device.isBorrowableLabOnly"
                          variant="outline"
                          class="text-blue-600 border-blue-200 bg-blue-50 text-xs py-0 px-1"
                        >
                          Không mượn về
                        </Badge>
                      </div>
                      <p
                        class="text-sm text-gray-900 font-medium whitespace-nowrap"
                      >
                        {{ device.name }}
                      </p>
                    </div>
                  </div>
                  <div class="col-span-3 text-center flex items-center">
                    <span
                      class="text-sm text-gray-900 font-medium w-full text-start"
                    >
                      SL: {{ device.quantity }} /
                      {{ device.expectedQuantity || device.quantity }}
                    </span>
                    <ChevronDownIcon
                      class="h-4 w-4 text-gray-400 transition-transform"
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
                      <Badge
                        :class="statusColorMap[item.status]"
                        variant="outline"
                        class="h-8 text-sm font-semibold w-fit whitespace-nowrap"
                      >
                        {{ statusMap[item.status] }}
                      </Badge>
                      <span class="text-gray-400">→</span>
                      <div class="w-28">
                        <Select
                          v-model="item.auditCondition"
                          class="flex-grow"
                          @update:modelValue="
                            updateDeviceCondition(item, item.auditCondition)
                          "
                        >
                          <SelectTrigger
                            class="h-7 text-xs bg-white font-semibold w-fit"
                            :class="
                              item.auditCondition
                                ? statusColorMap[item.auditCondition]
                                : 'text-gray-900'
                            "
                          >
                            <SelectValue placeholder="Chọn tình trạng" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem
                              :value="DeviceStatus.HEALTHY"
                              class="cursor-pointer"
                            >
                              <Badge
                                :class="statusColorMap[DeviceStatus.HEALTHY]"
                                variant="outline"
                              >
                                {{ statusMap[DeviceStatus.HEALTHY] }}
                              </Badge>
                            </SelectItem>
                            <SelectItem
                              :value="DeviceStatus.BROKEN"
                              class="cursor-pointer"
                            >
                              <Badge
                                :class="statusColorMap[DeviceStatus.BROKEN]"
                                variant="outline"
                              >
                                {{ statusMap[DeviceStatus.BROKEN] }}
                              </Badge>
                            </SelectItem>
                            <SelectItem
                              :value="DeviceStatus.LOST"
                              class="cursor-pointer"
                            >
                              <Badge
                                :class="statusColorMap[DeviceStatus.LOST]"
                                variant="outline"
                              >
                                {{ statusMap[DeviceStatus.LOST] }}
                              </Badge>
                            </SelectItem>
                            <SelectItem
                              :value="DeviceStatus.DISCARDED"
                              class="cursor-pointer"
                            >
                              <Badge
                                :class="statusColorMap[DeviceStatus.DISCARDED]"
                                variant="outline"
                              >
                                {{ statusMap[DeviceStatus.DISCARDED] }}
                              </Badge>
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
                          v-model="device.unscannedCondition"
                          @update:modelValue="
                            updateDeviceUnscannedItems(device)
                          "
                          class="flex-grow"
                        >
                          <SelectTrigger
                            class="h-8 text-sm bg-white font-semibold w-fit"
                            :class="
                              device.unscannedCondition
                                ? statusColorMap[device.unscannedCondition]
                                : 'text-gray-900'
                            "
                          >
                            <SelectValue placeholder="Chọn tình trạng" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem
                              :value="DeviceStatus.LOST"
                              class="cursor-pointer"
                            >
                              <Badge
                                :class="statusColorMap[DeviceStatus.LOST]"
                                variant="outline"
                              >
                                {{ statusMap[DeviceStatus.LOST] }}
                              </Badge>
                            </SelectItem>
                            <SelectItem
                              :value="DeviceStatus.DISCARDED"
                              class="cursor-pointer"
                            >
                              <Badge
                                :class="statusColorMap[DeviceStatus.DISCARDED]"
                                variant="outline"
                              >
                                {{ statusMap[DeviceStatus.DISCARDED] }}
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
        <div class="p-3 border-b border-gray-200">
          <h2 class="text-sm font-semibold flex items-center gap-1">
            <UserIcon class="h-4 w-4" />
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

            <div v-else class="rounded-lg px-3 py-1">
              <div class="flex items-center">
                <img
                  :src="userInfo.avatar || undefined"
                  alt="User avatar"
                  class="h-10 w-10 rounded-full object-cover"
                />
                <div class="ml-2">
                  <h4 class="text-xs font-medium text-gray-500">
                    Mã số:
                    <span class="text-gray-500 font-semibold">{{
                      userInfo.id
                    }}</span>
                    <span class="text-xs text-gray-500 italic font-semibold">
                      ({{
                        userInfo.roles?.map((r) => r.name).join(", ") ||
                        "Không có vai trò"
                      }})
                    </span>
                  </h4>
                  <p class="text-sm font-semibold text-gray-900">
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
            <div class="flex items-center gap-2">
              <div class="rounded-full bg-blue-50 p-1.5">
                <BoxIcon class="h-3.5 w-3.5 text-blue-600" />
              </div>
              <div class="flex justify-between w-full">
                <p class="text-xs text-gray-500">Tổng thiết bị</p>
                <p class="font-medium text-blue-600 text-sm">
                  {{ auditDetails.totalDevices }}
                </p>
              </div>
            </div>

            <div class="flex gap-4">
              <div class="w-px bg-gray-200 relative ml-12">
                <!-- Dots or markers can be added here if needed -->
              </div>
              <div class="space-y-2 flex-1">
                <div class="flex justify-between">
                  <span
                    :class="
                      statusColorMap[DeviceStatus.HEALTHY] +
                      ' text-sm bg-white font-semibold'
                    "
                  >
                    {{ statusMap[DeviceStatus.HEALTHY] }}
                  </span>
                  {{ auditDetails.deviceConditions.good }}
                </div>
                <div class="flex justify-between">
                  <span
                    :class="
                      statusColorMap[DeviceStatus.BROKEN] +
                      ' text-sm bg-white font-semibold'
                    "
                  >
                    {{ statusMap[DeviceStatus.BROKEN] }}
                  </span>
                  {{ auditDetails.deviceConditions.damaged }}
                </div>
                <div class="flex justify-between">
                  <span
                    :class="
                      statusColorMap[DeviceStatus.LOST] +
                      ' text-sm bg-white font-semibold'
                    "
                  >
                    {{ statusMap[DeviceStatus.LOST] }}
                  </span>
                  {{ auditDetails.deviceConditions.missing }}
                </div>
                <div class="flex justify-between">
                  <span
                    :class="
                      statusColorMap[DeviceStatus.DISCARDED] +
                      ' text-sm bg-white font-semibold'
                    "
                  >
                    {{ statusMap[DeviceStatus.DISCARDED] }}
                  </span>
                  {{ auditDetails.deviceConditions.discarded }}
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
              <LoaderIcon
                v-if="isConfirming"
                class="mr-3 -ml-1 size-5 animate-spin text-white"
              />
              <ClipboardCheckIcon v-else class="h-5 w-5" />
              Hoàn tất kiểm đếm
            </button>
          </div>
        </div>
      </div>
    </div>

    <div
      v-if="showIncompleteAuditsModal"
      class="fixed inset-0 flex items-center justify-center z-50"
    >
      <div
        class="fixed inset-0 bg-black bg-opacity-60"
        @click="showIncompleteAuditsModal = false"
      ></div>

      <div
        class="bg-white rounded-lg shadow-xl z-10 max-w-3xl w-full mx-4 overflow-hidden"
      >
        <div
          class="flex justify-between items-center border-b border-gray-200 px-4 py-3"
        >
          <h2 class="text-base font-semibold leading-6 text-gray-900">
            Kiểm đếm chưa hoàn thành
          </h2>
          <Button
            variant="ghost"
            size="icon"
            @click="showIncompleteAuditsModal = false"
            class="rounded-full"
          >
            <XIcon class="h-4 w-4" />
          </Button>
        </div>

        <div class="p-4 max-h-[70vh] overflow-y-auto">
          <div
            v-if="loadingIncompleteAudits"
            class="flex flex-col items-center justify-center py-10"
          >
            <LoaderIcon class="h-8 w-8 text-blue-600 animate-spin mb-4" />
            <p class="text-gray-500">Đang tải danh sách kiểm đếm...</p>
          </div>

          <div
            v-else-if="incompleteAudits.length === 0"
            class="flex flex-col items-center justify-center py-10 text-center"
          >
            <div class="rounded-full bg-gray-100 p-3 mb-4">
              <ClipboardCheckIcon class="h-8 w-8 text-gray-400" />
            </div>
            <p class="text-sm text-gray-500 max-w-xs">
              Không có phiên kiểm đếm nào chưa hoàn thành.
            </p>
          </div>

          <div v-else>
            <div class="space-y-3">
              <div
                v-for="audit in incompleteAudits"
                :key="audit.id"
                class="border border-gray-200 rounded-lg p-4 cursor-pointer transition-colors hover:bg-blue-50"
                :class="{
                  'border-blue-400 bg-blue-50': selectedAudit?.id === audit.id,
                }"
                @click="selectedAudit = audit"
              >
                <div class="flex justify-between">
                  <div>
                    <p class="font-medium text-gray-900 flex items-center">
                      <span class="mr-2">Mã kiểm đếm:</span>
                      <span class="font-semibold">{{
                        audit.id.slice(0, 8)
                      }}</span>
                      <Badge
                        :class="{
                          'bg-blue-100 text-blue-700':
                            audit.status === AssessmentStatus.ASSESSING,
                        }"
                        class="ml-2"
                      >
                        Đang ghi nhận
                      </Badge>
                    </p>
                    <p class="text-sm text-gray-500 mt-1">
                      Người kiểm đếm:
                      {{ audit.accountantName || audit.accountantId }}
                    </p>
                    <p class="text-sm text-gray-500">
                      Phòng lab: {{ audit.labRoom }}, {{ audit.labBranch }}
                    </p>
                    <p class="text-sm text-gray-500">
                      Số thiết bị: {{ audit.deviceIds?.length }}
                    </p>
                    <p class="text-sm text-gray-500">
                      Thời gian tạo:
                      {{
                        audit.createdAt
                          ? new Date(audit.createdAt).toLocaleString("vi-VN")
                          : "N/A"
                      }}
                    </p>
                  </div>

                  <div class="flex flex-col gap-2">
                    <Button
                      v-if="selectedAudit?.id === audit.id"
                      variant="destructive"
                      size="sm"
                      class="text-xs"
                      @click.stop="cancelAudit(audit.id)"
                    >
                      Hủy kiểm đếm
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
            @click="showIncompleteAuditsModal = false"
          >
            Đóng
          </Button>

          <Button
            class="w-40"
            variant="default"
            @click="continueAudit"
            :disabled="!selectedAudit || loadingIncompleteAudits"
          >
            <LoaderIcon
              v-if="loadingIncompleteAudits"
              class="h-5 w-5 animate-spin"
            />
            <CheckIcon v-else class="h-5 w-5" />
            {{
              loadingIncompleteAudits
                ? "Đang tải dữ liệu..."
                : "Tiếp tục kiểm đếm"
            }}
          </Button>
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
            Mã kiểm đếm: {{ auditId.slice(0, 8) }}
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
