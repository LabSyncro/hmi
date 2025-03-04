import { createRouter, createWebHistory } from 'vue-router'
import DeviceDetail from '../components/device/DeviceDetail.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/',
      name: 'home',
      component: () => import('../components/Home.vue')
    },
    {
      path: '/device/:id',
      name: 'device-detail',
      component: DeviceDetail
    }
  ]
})

export default router 