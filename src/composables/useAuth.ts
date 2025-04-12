import { computed, ref } from "vue";
import { useRouter } from "vue-router";

export function useAuth() {
  const router = useRouter();
  const deviceCode = ref("");
  const isLoading = ref(false);
  const error = ref<string | null>(null);
  const isAuthenticated = computed(() => !!localStorage.getItem("auth_token"));

  const generateDeviceCode = () => {
    const randomCode = Math.floor(100000 + Math.random() * 900000).toString();
    deviceCode.value = randomCode.slice(0, 3) + " " + randomCode.slice(3);
    return deviceCode.value;
  };

  const checkDeviceCode = async (_code: string) => {
    isLoading.value = true;
    error.value = null;

    try {
      await new Promise((resolve) => setTimeout(resolve, 1500));

      const token = "demo_auth_token_" + Date.now();
      localStorage.setItem("auth_token", token);
      localStorage.setItem(
        "user_info",
        JSON.stringify({
          name: "Demo User",
          role: "Lab Manager",
          lab: "601 H6, Dĩ An",
        })
      );

      router.push({ name: "home" });
      return true;
    } catch (err) {
      error.value = "Failed to authenticate. Please try again.";
      return false;
    } finally {
      isLoading.value = false;
    }
  };

  const logout = () => {
    localStorage.removeItem("auth_token");
    localStorage.removeItem("user_info");
    router.push({ name: "login" });
  };

  const getUserInfo = () => {
    const userInfo = localStorage.getItem("user_info");
    return userInfo ? JSON.parse(userInfo) : null;
  };

  return {
    deviceCode,
    isLoading,
    error,
    isAuthenticated,
    generateDeviceCode,
    checkDeviceCode,
    logout,
    getUserInfo,
  };
}
