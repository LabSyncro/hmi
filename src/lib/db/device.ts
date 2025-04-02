import { db } from './client'
import { DeviceStatus } from '@/types/db/generated'

export type DeviceDetail = {
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
  kind: string;
  borrower?: {
    id: string;
    name: string;
    image: string | null;
  } | null;
  borrowedAt?: Date | null;
  expectedReturnAt?: Date | null;
  borrowedLab?: string | null;
  expectedReturnLab?: string | null;
  receiptId?: string | null;
}

export type DeviceInventory = {
  branch: string;
  room: string;
  borrowingQuantity: number;
  availableQuantity: number;
}

export const deviceService = {
  async getDeviceById(id: string): Promise<DeviceDetail | null> {
    if (!id) {
      return null
    }

    try {
      const sql = `
        SELECT 
          d.full_id,
          d.status,
          d.kind,
          dk.image,
          dk.name AS device_name,
          dk.allowed_borrow_roles,
          dk.allowed_view_roles,
          dk.brand,
          dk.manufacturer,
          dk.description,
          c.name AS category_name,
          l.room,
          l.branch,
          r.id AS receipt_id,
          a.created_at AS borrowed_at,
          rd.expected_returned_at,
          bl.room || ', ' || bl.branch AS borrowed_lab,
          rl.room || ', ' || rl.branch AS expected_return_lab,
          u.id AS borrower_id,
          u.name AS borrower_name,
          u.image AS borrower_image
        FROM 
          devices d
          LEFT JOIN device_kinds dk ON d.kind = dk.id
          LEFT JOIN labs l ON d.lab_id = l.id
          LEFT JOIN categories c ON dk.category_id = c.id
          LEFT JOIN receipts_devices rd ON d.id = rd.device_id AND rd.return_id IS NULL
          LEFT JOIN receipts r ON rd.receipt_id = r.id
          LEFT JOIN users u ON r.borrower_id = u.id
          LEFT JOIN labs bl ON r.borrowed_lab_id = bl.id
          LEFT JOIN labs rl ON rd.expected_returned_lab_id = rl.id
          LEFT JOIN activities a ON rd.borrow_id = a.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [id]
      });

      if (results.length === 0) {
        return null
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
        kind: row.kind as string,
        receiptId: row.receiptId as string | null,
        borrower: row.borrowerId ? {
          id: row.borrowerId as string,
          name: row.borrowerName as string,
          image: row.borrowerImage as string | null,
        } : null,
        borrowedAt: row.borrowedAt ? new Date(row.borrowedAt as string) : null,
        expectedReturnAt: row.expectedReturnedAt ? new Date(row.expectedReturnedAt as string) : null,
        borrowedLab: row.borrowedLab as string | null,
        expectedReturnLab: row.expectedReturnLab as string | null,
      };

      return deviceDetail;
    } catch (error) {
      throw error;
    }
  },

  async getDeviceInventoryByKindId(kindId: string): Promise<DeviceInventory[]> {
    try {
      const sql = `
        SELECT 
          l.branch,
          l.room,
          SUM(CASE WHEN d.status = 'borrowing' THEN 1 ELSE 0 END)::int as borrowing_quantity,
          SUM(CASE WHEN d.status IN ('healthy', 'borrowing') THEN 1 ELSE 0 END)::int as available_quantity
        FROM 
          labs l
          JOIN devices d ON l.id = d.lab_id
          JOIN device_kinds dk ON d.kind = dk.id
        WHERE 
          dk.id = $1
          AND d.deleted_at IS NULL
        GROUP BY
          l.id,
          l.branch,
          l.room
        ORDER BY
          l.branch,
          l.room
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [kindId]
      });

      return results.map(row => ({
        branch: row.branch as string,
        room: row.room as string,
        borrowingQuantity: row.borrowingQuantity as number,
        availableQuantity: row.availableQuantity as number
      }));
    } catch (error) {
      throw error;
    }
  },

  async getDeviceStatusById(id: string): Promise<DeviceStatus | null> {
    if (!id) {
      return null
    }
    try {
      const results = await db.query<{ status: DeviceStatus }>({
        table: 'devices',
        columns: ['status'],
        conditions: [
          ['id', id],
          ['deleted_at', null]
        ],
        limit: 1
      })

      if (results.length > 0) {
        const status = results[0].status
        if (!Object.values(DeviceStatus).includes(status)) {
          return null
        }
        return status
      } else {
        return null
      }
    } catch (error) {
      return null
    }
  }
} 