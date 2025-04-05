import { DeviceStatus } from '@/lib/db'

export const statusMap: Record<DeviceStatus, string> = {
  'healthy': 'Tốt',
  'broken': 'Hư hỏng',
  'borrowing': 'Đang mượn',
  'discarded': 'Đã bỏ',
  'assessing': 'Đang đánh giá',
  'maintaining': 'Đang bảo trì',
  'shipping': 'Đang giao hàng',
  'lost': 'Đã mất'
}

export const statusColorMap: Record<DeviceStatus, string> = {
  'healthy': 'text-green-600',
  'broken': 'text-red-600',
  'borrowing': 'text-blue-600',
  'discarded': 'text-gray-600',
  'assessing': 'text-yellow-600',
  'maintaining': 'text-orange-600',
  'shipping': 'text-purple-600',
  'lost': 'text-black'
}

export type UserInfo = {
  id: string
  name: string
  avatar: string
  roles: { name: string; key: string }[]
}

export type DeviceItem = {
  id: string
  status: DeviceStatus
  returnCondition?: DeviceStatus
}

export type Device = {
  code: string
  name: string,
  image: string,
  quantity: number
  unit: string
  expanded: boolean
  items: DeviceItem[]
}