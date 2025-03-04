import { createRouter, createWebHistory } from 'vue-router'
import Home from '../components/Home.vue'
import DeviceDetail from '../components/device/DeviceDetail.vue'
import BorrowForm from '../components/device/BorrowForm.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: Home
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
    }
  ]
})

export default router 