import { DatabaseClient } from '@/lib/db/client'
import type { DeviceStatus } from '@/types/db/generated'

export interface DeviceDetail {
  fullId: string;
  status: DeviceStatus | null;
  image: any;
  deviceName: string;
  allowedBorrowRoles: string[];
  allowedViewRoles: string[];
  brand: string | null;
  manufacturer: string | null;
  description: string | null;
  categoryName: string;
  labRoom: string | null;
  labBranch: string | null;
}

export async function getDeviceById(id: string): Promise<DeviceDetail | null> {
  try {
    const db = DatabaseClient.getInstance();
    const sql = `
      SELECT 
        d.full_id,
        d.status,
        dk.image,
        dk.name AS device_name,
        dk.allowed_borrow_roles,
        dk.allowed_view_roles,
        dk.brand,
        dk.manufacturer,
        dk.description,
        c.name AS category_name,
        l.room,
        l.branch
      FROM devices d
      LEFT JOIN device_kinds dk ON d.kind = dk.id
      LEFT JOIN labs l ON d.lab_id = l.id
      LEFT JOIN categories c ON dk.category_id = c.id
      WHERE d.id = $1
    `;

    const results = await db.queryRaw<Record<string, unknown>>({
      sql,
      params: [id]
    });

    if (results.length === 0) {
      return null;
    }

    const row = results[0];

    const deviceDetail: DeviceDetail = {
      fullId: row.fullId as string,
      status: row.status as DeviceStatus | null,
      image: row.image,
      deviceName: row.deviceName as string,
      allowedBorrowRoles: row.allowedBorrowRoles as string[],
      allowedViewRoles: row.allowedViewRoles as string[],
      brand: row.brand as string | null,
      manufacturer: row.manufacturer as string | null,
      description: row.description as string | null,
      categoryName: row.categoryName as string,
      labRoom: row.room as string | null,
      labBranch: row.branch as string | null,
    };

    return deviceDetail;
  } catch (error) {
    throw error;
  }
} 