import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";
import AutoImport from "unplugin-auto-import/vite";
import { ElementPlusResolver } from "unplugin-vue-components/resolvers";
import Components from "unplugin-vue-components/vite";
import { defineConfig } from "vite";

// @ts-ignore
const host = process.env.TAURI_DEV_HOST;

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    AutoImport({
      imports: [
        "vue",
        "vue-router",
        "vue-i18n",
        "pinia",
        "@vueuse/core",
        "vee-validate",
        {
          axios: [["default", "axios"]],
          "@josempgon/vue-keycloak": [
            "useKeycloak",
            ["getToken", "getKeycloakToken"],
          ],
        },
        {
          from: "vue-router",
          imports: ["RouteLocationRaw"],
          type: true,
        },
        { from: "@/components/ui/toast", imports: ["toast"] },
        {
          from: "@/composables",
          imports: ["useOneTimeQR", "useVirtualKeyboardDetection"],
        },
        {
          from: "@/lib/db",
          imports: [
            "receiptService",
            "userService",
            "deviceService",
            "auditService",
            "maintenanceService",
            "shipmentService",
            "labService",
            "searchService",
          ],
        },
        {
          from: "@/types/db/generated",
          imports: [
            "DeviceStatus",
            "AssessmentStatus",
            "MaintenanceStatus",
            "ShipmentStatus",
          ],
        },
        {
          from: "@/lib/utils",
          imports: ["cn"],
        },
        {
          from: "@/types",
          imports: [
            "statusMap",
            "statusColorMap",
            "qualityMap",
            "qualityColorMap",
          ],
        },
        {
          from: "@/types",
          imports: [
            "Device",
            "DeviceItem",
            "QualityDeviceItem",
            "UserInfo",
            "AuditDevice",
            "AuditDeviceItem",
            "IncompleteAudit",
            "MaintenanceDevice",
            "MaintenanceDeviceItem",
            "MaintenanceSession",
            "ShipmentDevice",
            "ShipmentDeviceItem",
            "Accessory",
            "UserBorrowHistoryItem",
            "UserActivityItem",
          ],
          type: true,
        },
      ],
      dts: true,
      vueTemplate: true,
    }),
    Components({
      resolvers: [
        ElementPlusResolver(),
        (componentName) => {
          if (["ValidationForm", "ValidationField"].includes(componentName))
            return {
              name: componentName.split("Validation")[1],
              from: "vee-validate",
            };
        },
      ],
      dts: true,
    }),
  ],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == "windows" ? "chrome105" : "safari13",
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
});
