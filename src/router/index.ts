import BorrowForm from "@/components/app/device/BorrowForm.vue";
import BorrowInvoice from "@/components/app/device/BorrowInvoice.vue";
import ConfirmBorrowForm from "@/components/app/device/ConfirmBorrowForm.vue";
import ConfirmReturnForm from "@/components/app/device/ConfirmReturnForm.vue";
import ReturnForm from "@/components/app/device/ReturnForm.vue";
import ReturnInvoice from "@/components/app/device/ReturnInvoice.vue";
import AuthLayout from "@/layouts/AuthLayout.vue";
import AuditPage from "@/pages/audit/AuditPage.vue";
import LoginPage from "@/pages/auth/LoginPage.vue";
import BorrowReturn from "@/pages/borrow-return/index.vue";
import RecordPage from "@/pages/borrow-return/RecordPage.vue";
import DetailInfoSearch from "@/pages/detail/DetailInfoSearch.vue";
import MaintenancePage from "@/pages/maintenance/MaintenancePage.vue";
import TransportPage from "@/pages/transport/TransportPage.vue";
import { createRouter, createWebHistory } from "vue-router";

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: "/login",
      name: "login",
      component: LoginPage,
      meta: { layout: AuthLayout },
    },
    {
      path: "/",
      name: "home",
      component: RecordPage,
      meta: { requiresAuth: true },
    },
    {
      path: "/borrow-return",
      name: "borrow-return",
      component: BorrowReturn,
      meta: { requiresAuth: true },
    },
    {
      path: "/audit",
      name: "audit",
      component: AuditPage,
      meta: { requiresAuth: true },
    },
    {
      path: "/maintenance",
      name: "maintenance",
      component: MaintenancePage,
      meta: { requiresAuth: true },
    },
    {
      path: "/transport",
      name: "transport",
      component: TransportPage,
      meta: { requiresAuth: true },
    },
    {
      path: "/search",
      name: "search",
      component: DetailInfoSearch,
      meta: { requiresAuth: true },
    },
    {
      path: "/device/:id/borrow",
      name: "device-borrow",
      component: BorrowForm,
      meta: { requiresAuth: true },
    },
    {
      path: "/device/:id/return",
      name: "device-return",
      component: ReturnForm,
      meta: { requiresAuth: true },
    },
    {
      path: "/device/:id/borrow/confirm",
      name: "confirm-borrow",
      component: ConfirmBorrowForm,
      meta: { requiresAuth: true },
    },
    {
      path: "/device/borrow-invoice",
      name: "borrow-invoice",
      component: BorrowInvoice,
      meta: { requiresAuth: true },
    },
    {
      path: "/device/:id/return/confirm",
      name: "confirm-return",
      component: ConfirmReturnForm,
      meta: { requiresAuth: true },
    },
    {
      path: "/device/return-invoice",
      name: "return-invoice",
      component: ReturnInvoice,
      meta: { requiresAuth: true },
    },
  ],
});

router.beforeEach((to, _from, next) => {
  const isAuthenticated = localStorage.getItem("auth_token");

  if (
    to.matched.some((record) => record.meta.requiresAuth) &&
    !isAuthenticated
  ) {
    next({ name: "login" });
  } else if (to.name === "login" && isAuthenticated) {
    next({ name: "home" });
  } else {
    next();
  }
});

export default router;
