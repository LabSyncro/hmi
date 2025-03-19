import { createRouter, createWebHistory } from 'vue-router'
import Home from '../components/Home.vue'
import DeviceDetail from '../components/device/DeviceDetail.vue'
import BorrowForm from '../components/device/BorrowForm.vue'
import ConfirmBorrowForm from '../components/device/ConfirmBorrowForm.vue'
import BorrowInvoice from '../components/device/BorrowInvoice.vue'
import ReturnForm from '../components/device/ReturnForm.vue'
import ConfirmReturnForm from '../components/device/ConfirmReturnForm.vue'
import ReturnInvoice from '../components/device/ReturnInvoice.vue'
import BorrowReturn from '../pages/borrow-return/index.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Home
    },
    {
      path: '/borrow-return',
      name: 'borrow-return',
      component: BorrowReturn
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