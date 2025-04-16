import { hmiService } from "@/lib/db";
import { computed, ref } from "vue";
import { useRouter } from "vue-router";

const POLLING_INTERVAL_MS = 3000;
const POLLING_TIMEOUT_MS = 5 * 60 * 1000;

type LoginStatus =
  | "idle"
  | "polling"
  | "awaiting_lab"
  | "success"
  | "error"
  | "timeout";

interface LabInfo {
  id: string;
  name?: string;
  room?: string;
  branch?: string;
}

interface UserInfo {
  id: string;
  email: string;
  name: string;
  avatar: string;
  roles: { name: string; key: string }[];
}

interface AuthResponse {
  status: "success" | "pending" | "error";
  token?: string;
  lab?: LabInfo;
  user?: UserInfo;
  message?: string;
}

function isPolling(status: LoginStatus): boolean {
  return status === "polling";
}

export function useAuth() {
  const router = useRouter();
  const hmiCode = ref("");
  const isLoading = ref(false);
  const isGeneratingCode = ref(false);
  const error = ref<string | null>(null);
  const isAuthenticated = computed(() => !!localStorage.getItem("auth_token"));
  const labInfo = ref<LabInfo | null>(null);
  const userInfo = ref<UserInfo | null>(null);

  const loginStatus = ref<LoginStatus>("idle");
  const pollingIntervalId = ref<number | null>(null);
  const pollingTimeoutId = ref<number | null>(null);

  const generateHMICode = async () => {
    loginStatus.value = "idle";
    error.value = null;
    hmiCode.value = "";
    isGeneratingCode.value = true;
    isLoading.value = true;

    try {
      const hmiDetail = await hmiService.generateHMICode();
      if (hmiDetail) {
        const code = hmiDetail.hmiCode;
        hmiCode.value = code.slice(0, 3) + " " + code.slice(3);
      } else {
        throw new Error("Không thể tạo mã HMI từ service.");
      }
    } catch (err) {
      error.value =
        err instanceof Error
          ? err.message
          : "Lỗi không xác định khi tạo mã HMI.";
      hmiCode.value = "";
    } finally {
      isGeneratingCode.value = false;
      if (!isPolling(loginStatus.value)) {
        isLoading.value = false;
      }
    }
    return hmiCode.value;
  };

  const handleSuccessfulLogin = (
    token: string,
    userData: UserInfo,
    labData?: LabInfo
  ) => {
    localStorage.setItem("auth_token", token);

    const userStorage = {
      ...userData,
      lab: labData || null,
    };
    localStorage.setItem("user_info", JSON.stringify(userStorage));

    userInfo.value = userData;
    labInfo.value = labData || null;
  };

  const checkLoginStatus = async (codeWithoutSpace: string) => {
    const apiBaseUrl =
      import.meta.env.VITE_API_BASE_URL || "http://localhost:3000";
    const url = `${apiBaseUrl}/api/auth/status/${encodeURIComponent(codeWithoutSpace)}`;

    try {
      const response = await fetch(url, {
        method: "GET",
        headers: {
          Accept: "application/json",
        },
      });

      if (!response.ok) {
        let errorMsg = `HTTP error! Status: ${response.status}`;
        try {
          const errorData = await response.json();
          errorMsg = errorData.message || errorMsg;
        } catch (jsonError) {
          // Ignore if response body is not valid JSON
        }
        throw new Error(errorMsg);
      }

      const data = (await response.json()) as AuthResponse;

      switch (data.status) {
        case "success":
          if (!data.token || !data.user) {
            throw new Error("Lỗi xác thực: Thiếu thông tin người dùng.");
          }
          if (!data.lab) {
            loginStatus.value = "awaiting_lab";
            isLoading.value = true;
            return;
          }
          loginStatus.value = "success";
          handleSuccessfulLogin(data.token, data.user, data.lab);
          stopLoginPolling();
          break;
        case "pending":
          if (loginStatus.value !== "awaiting_lab") {
            loginStatus.value = "polling";
          }
          break;
        case "error":
          error.value =
            data.message || "Đã xảy ra lỗi trong quá trình đăng nhập.";
          loginStatus.value = "error";
          stopLoginPolling();
          break;
        default:
          break;
      }
    } catch (fetchError) {
      error.value =
        fetchError instanceof Error
          ? fetchError.message
          : "Lỗi kết nối. Vui lòng kiểm tra kết nối mạng hoặc máy chủ.";
      loginStatus.value = "error";
      stopLoginPolling();
    }
  };

  const startLoginPolling = (codeWithSpace: string) => {
    if (pollingIntervalId.value || pollingTimeoutId.value) {
      stopLoginPolling();
    }
    if (!codeWithSpace) {
      return;
    }

    const codeWithoutSpace = codeWithSpace.replace(/\s/g, "");

    loginStatus.value = "polling";
    isLoading.value = true;
    error.value = null;

    checkLoginStatus(codeWithoutSpace);
    pollingIntervalId.value = window.setInterval(() => {
      if (
        isPolling(loginStatus.value) ||
        loginStatus.value === "awaiting_lab"
      ) {
        checkLoginStatus(codeWithoutSpace);
      } else {
        stopLoginPolling();
      }
    }, POLLING_INTERVAL_MS);

    pollingTimeoutId.value = window.setTimeout(() => {
      if (
        isPolling(loginStatus.value) ||
        loginStatus.value === "awaiting_lab"
      ) {
        error.value = "Login timed out. Please try generating a new code.";
        loginStatus.value = "timeout";
        stopLoginPolling();
      }
    }, POLLING_TIMEOUT_MS);
  };

  const stopLoginPolling = () => {
    if (pollingIntervalId.value) {
      window.clearInterval(pollingIntervalId.value);
      pollingIntervalId.value = null;
    }
    if (pollingTimeoutId.value) {
      window.clearTimeout(pollingTimeoutId.value);
      pollingTimeoutId.value = null;
    }
    if (
      isLoading.value &&
      loginStatus.value !== "success" &&
      loginStatus.value !== "awaiting_lab"
    ) {
      isLoading.value = false;
    }
  };

  const logout = () => {
    localStorage.removeItem("auth_token");
    localStorage.removeItem("user_info");
    router.push({ name: "login" });
    stopLoginPolling();
    loginStatus.value = "idle";
    labInfo.value = null;
    userInfo.value = null;
  };

  const getUserInfo = () => {
    const userInfoStr = localStorage.getItem("user_info");
    return userInfoStr ? JSON.parse(userInfoStr) : null;
  };

  return {
    hmiCode,
    isLoading,
    isGeneratingCode,
    error,
    isAuthenticated,
    loginStatus,
    labInfo,
    userInfo,
    generateHMICode,
    startLoginPolling,
    stopLoginPolling,
    logout,
    getUserInfo,
  };
}
