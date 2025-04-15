import { hmiService } from "@/lib/db";
import { computed, ref } from "vue";
import { useRouter } from "vue-router";

const POLLING_INTERVAL_MS = 3000;
const POLLING_TIMEOUT_MS = 5 * 60 * 1000;

type LoginStatus =
  | "idle"
  | "polling"
  | "success"
  | "error"
  | "cancelled"
  | "timeout";

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

  const handleSuccessfulLogin = (token: string) => {
    localStorage.setItem("auth_token", token);
    localStorage.setItem(
      "user_info",
      JSON.stringify({
        name: "Authenticated User",
        role: "Lab User",
        lab: "Determined by Auth",
      })
    );
    loginStatus.value = "success";
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

      const data = await response.json();

      switch (data.status) {
        case "success":
          if (!data.token) {
            throw new Error("Success status received but no token provided.");
          }
          handleSuccessfulLogin(data.token);
          stopLoginPolling();
          break;
        case "pending":
          break;
        case "error":
          error.value =
            data.message || "An unknown error occurred during login.";
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
          : "Polling failed. Check connection or server.";
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
      if (isPolling(loginStatus.value)) {
        checkLoginStatus(codeWithoutSpace);
      } else {
        stopLoginPolling();
      }
    }, POLLING_INTERVAL_MS);

    pollingTimeoutId.value = window.setTimeout(() => {
      if (isPolling(loginStatus.value)) {
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
    if (isLoading.value && loginStatus.value !== "success") {
      isLoading.value = false;
    }
    if (loginStatus.value === "polling" || loginStatus.value === "timeout") {
      loginStatus.value = "idle";
    }
  };

  const logout = () => {
    localStorage.removeItem("auth_token");
    localStorage.removeItem("user_info");
    router.push({ name: "login" });
    stopLoginPolling();
    loginStatus.value = "idle";
  };

  const getUserInfo = () => {
    const userInfo = localStorage.getItem("user_info");
    return userInfo ? JSON.parse(userInfo) : null;
  };

  return {
    hmiCode,
    isLoading,
    isGeneratingCode,
    error,
    isAuthenticated,
    loginStatus,
    generateHMICode,
    startLoginPolling,
    stopLoginPolling,
    logout,
    getUserInfo,
  };
}
