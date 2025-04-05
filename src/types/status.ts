import { DeviceQuality, DeviceStatus } from '@/lib/db'

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

export const qualityMap: Record<DeviceQuality, string> = {
  'healthy': 'Tốt',
  'broken': 'Hư hỏng',
  'needs_fixing': 'Cần sửa chữa',
  'lost': 'Đã mất'
}

export const statusColorMap: Record<DeviceStatus, string> = {
  'healthy': 'text-green-600 bg-green-50 border-green-600',
  'broken': 'text-red-600 bg-red-50 border-red-600',
  'borrowing': 'text-blue-600 bg-blue-50 border-blue-600',
  'discarded': 'text-gray-600 bg-gray-50 border-gray-600',
  'assessing': 'text-yellow-600 bg-yellow-50 border-yellow-600',
  'maintaining': 'text-orange-600 bg-orange-50 border-orange-600',
  'shipping': 'text-purple-600 bg-purple-50 border-purple-600',
  'lost': 'text-black bg-gray-50 border-black'
}

export const qualityColorMap: Record<DeviceQuality, string> = {
  'healthy': 'text-green-600 bg-green-50 border-green-600',
  'broken': 'text-red-600 bg-red-50 border-red-600',
  'needs_fixing': 'text-yellow-600 bg-yellow-50 border-yellow-600',
  'lost': 'text-black bg-gray-50 border-black'
}

export type UserInfo = {
  id: string
  name: string
  avatar: string
  roles: { name: string; key: string }[]
}

// Base device item type with common fields
export type BaseDeviceItem = {
  id: string
  status: DeviceStatus
}

// Workflow-specific types
export type AuditDeviceItem = BaseDeviceItem & {
  auditCondition: DeviceStatus
}

export type MaintenanceDeviceItem = BaseDeviceItem & {
  maintenanceOutcome: DeviceStatus
}

export type ReturnDeviceItem = BaseDeviceItem & {
  returnCondition: DeviceStatus
  prevQuality?: DeviceQuality
}

export type TransportDeviceItem = BaseDeviceItem & {
  transportDestination: string
}

// Generic device type that can be used for any workflow
export type Device = {
  code: string
  name: string
  image: any
  quantity: number
  unit: string
  expanded: boolean
  items: BaseDeviceItem[]
}

// Workflow-specific device types
export type AuditDevice = Omit<Device, 'items'> & {
  items: AuditDeviceItem[]
}

export type MaintenanceDevice = Omit<Device, 'items'> & {
  items: MaintenanceDeviceItem[]
}