import { createRouter, createWebHistory } from 'vue-router'
import DeviceDetail from '@/components/app/device/DeviceDetail.vue'
import BorrowForm from '@/components/app/device/BorrowForm.vue'
import ConfirmBorrowForm from '@/components/app/device/ConfirmBorrowForm.vue'
import BorrowInvoice from '@/components/app/device/BorrowInvoice.vue'
import ReturnForm from '@/components/app/device/ReturnForm.vue'
import ConfirmReturnForm from '@/components/app/device/ConfirmReturnForm.vue'
import ReturnInvoice from '@/components/app/device/ReturnInvoice.vue'
import BorrowReturn from '@/pages/borrow-return/index.vue'
import RecordPage from '@/pages/borrow-return/RecordPage.vue'
import AuditPage from '@/pages/audit/AuditPage.vue'
import MaintenancePage from '@/pages/maintenance/MaintenancePage.vue'
import TransportPage from '@/pages/transport/TransportPage.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: RecordPage
    },
    {
      path: '/borrow-return',
      name: 'borrow-return',
      component: BorrowReturn
    },
    {
      path: '/audit',
      name: 'audit',
      component: AuditPage
    },
    {
      path: '/maintenance',
      name: 'maintenance',
      component: MaintenancePage
    },
    {
      path: '/transport',
      name: 'transport',
      component: TransportPage
    },
    {
      path: '/device/:id',
      name: 'device-detail',
      component: DeviceDetail
    },
    {
      path: '/device/:id/borrow',
      name: 'device-borrow',
      component: BorrowForm
    },
    {
      path: '/device/:id/return',
      name: 'device-return',
      component: ReturnForm
    },
    {
      path: '/device/:id/borrow/confirm',
      name: 'confirm-borrow',
      component: ConfirmBorrowForm
    },
    {
      path: '/device/borrow-invoice',
      name: 'borrow-invoice',
      component: BorrowInvoice
    },
    {
      path: '/device/:id/return/confirm',
      name: 'confirm-return',
      component: ConfirmReturnForm
    },
    {
      path: '/device/return-invoice',
      name: 'return-invoice',
      component: ReturnInvoice
    }
  ]
})

export default router 