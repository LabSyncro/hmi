import { DeviceQuality, DeviceStatus } from "@/types/db/generated";
import { db } from "./client";

interface DeviceItem {
  id: string;
  returnCondition?: string;
  prevQuality?: string;
  expectedReturnedLabId?: string;
  expectedReturnedAt: Date;
}

export interface Receipt {
  id: string;
  borrowerId: string;
  borrowCheckerId: string;
  borrowedLabId: string;
  createdAt: Date;
  updatedAt: Date;
}

interface CreateReceiptParams {
  id: string;
  borrowerId: string;
  borrowCheckerId: string;
  borrowedLabId: string;
  devices: DeviceItem[];
}

export const receiptService = {
  async fetchReadyBorrowDevices(
    offset: number,
    limit: number,
    options: { desc?: boolean; sortField?: string }
  ) {
    try {
      const orderClause = options.sortField
        ? `ORDER BY ${options.sortField} ${options.desc ? "DESC" : "ASC"}`
        : "ORDER BY device_kinds.name ASC";

      const sql = `
                SELECT 
                    device_kinds.id as kind,
                    device_kinds.name,
                    device_kinds.image,
                    COUNT(*) as quantity,
                    labs.name as place,
                    COUNT(*) OVER() as total_count
                FROM 
                    devices
                    JOIN device_kinds ON devices.kind = device_kinds.id
                    LEFT JOIN labs ON devices.lab_id = labs.id
                WHERE
                    devices.status::text = $1
                    AND devices.deleted_at IS NULL
                GROUP BY 
                    device_kinds.id,
                    device_kinds.name,
                    device_kinds.image,
                    labs.name
                ${orderClause}
                LIMIT $2 
                OFFSET $3
            `;

      const result = await db.queryRaw<{
        kind: string;
        name: string;
        image: any;
        quantity: number;
        place: string;
        totalCount: number;
      }>({
        sql,
        params: [DeviceStatus.HEALTHY, limit, offset],
      });

      const totalCount = result[0]?.totalCount || 0;

      return {
        data: result.map(({ totalCount, ...rest }) => rest),
        totalPages: Math.ceil(totalCount / limit),
        totalCount: totalCount,
      };
    } catch (error) {
      throw error;
    }
  },

  async fetchBorrowingDevices(
    offset: number,
    limit: number,
    options: { desc?: boolean; sortField?: string }
  ) {
    try {
      const orderClause = options.sortField
        ? `ORDER BY ${options.sortField} ${options.desc ? "DESC" : "ASC"}`
        : "ORDER BY activities.created_at DESC";

      const sql = `
                SELECT 
                    receipts.id as receipt_code,
                    users.name as borrower_name,
                    users.image as borrower_image,
                    COUNT(*) as total_qty,
                    COUNT(CASE WHEN receipts_devices.return_checker_id IS NOT NULL THEN 1 END) as returned_qty,
                    labs.name as borrowed_place,
                    activities.created_at as borrowed_at,
                    receipts_devices.expected_returned_at,
                    CASE 
                        WHEN receipts_devices.expected_returned_at < CURRENT_TIMESTAMP THEN 'late'
                        ELSE 'on_time'
                    END as status,
                    CASE 
                        WHEN receipts_devices.return_checker_id IS NULL THEN 'borrowing'
                        ELSE 'returned'
                    END as borrow_state,
                    COUNT(*) OVER() as total_count
                FROM 
                    receipts_devices
                    JOIN receipts ON receipts_devices.receipt_id = receipts.id
                    JOIN users ON receipts.borrower_id = users.id
                    JOIN labs ON receipts.borrowed_lab_id = labs.id
                    JOIN activities ON receipts_devices.borrow_id = activities.id
                WHERE 
                    users.deleted_at IS NULL
                    AND receipts_devices.return_id IS NULL
                GROUP BY 
                    receipts.id,
                    users.name,
                    users.image,
                    labs.name,
                    activities.created_at,
                    receipts_devices.expected_returned_at,
                    receipts_devices.return_checker_id
                ${orderClause}
                LIMIT $1 OFFSET $2
            `;

      const result = await db.queryRaw<{
        receiptCode: string;
        borrowerName: string;
        borrowerImage: string;
        totalQty: number;
        returnedQty: number;
        borrowedPlace: string;
        borrowedAt: Date;
        expectedReturnedAt: Date;
        status: "late" | "on_time";
        borrowState: "borrowing" | "returned";
        totalCount: number;
      }>({
        sql,
        params: [limit, offset],
      });

      const totalCount = result[0]?.totalCount || 0;

      return {
        data: result.map(({ totalCount, ...rest }) => rest),
        totalPages: Math.ceil(totalCount / limit),
        totalCount: totalCount,
      };
    } catch (error) {
      throw error;
    }
  },

  async fetchReturnedDevices(
    offset: number,
    limit: number,
    options: { desc?: boolean; sortField?: string }
  ) {
    try {
      const orderClause = options.sortField
        ? `ORDER BY ${options.sortField} ${options.desc ? "DESC" : "ASC"}`
        : "ORDER BY activities.created_at DESC";

      const sql = `
                SELECT 
                    receipts.id as receipt_code,
                    users.name as returned_name,
                    users.image as returned_image,
                    COUNT(*) as quantity,
                    labs.name as returned_place,
                    activities.created_at as returned_at,
                    CASE 
                        WHEN receipts_devices.expected_returned_at < activities.created_at THEN 'late'
                        ELSE 'on_time'
                    END as status,
                    receipts_devices.note,
                    COUNT(*) OVER() as total_count
                FROM 
                    receipts_devices
                    JOIN receipts ON receipts_devices.receipt_id = receipts.id
                    JOIN users ON receipts_devices.return_checker_id = users.id
                    JOIN labs ON receipts.borrowed_lab_id = labs.id
                    JOIN activities ON receipts_devices.return_id = activities.id
                WHERE 
                    users.deleted_at IS NULL
                    AND receipts_devices.return_id IS NOT NULL
                GROUP BY 
                    receipts.id,
                    users.name,
                    users.image,
                    labs.name,
                    activities.created_at,
                    receipts_devices.expected_returned_at,
                    receipts_devices.note
                ${orderClause}
                LIMIT $1 OFFSET $2
            `;

      const result = await db.queryRaw<{
        receiptCode: string;
        returnedName: string;
        returnedImage: string;
        quantity: number;
        returnedPlace: string;
        returnedAt: Date;
        status: "late" | "on_time";
        note: string;
        totalCount: number;
      }>({
        sql,
        params: [limit, offset],
      });

      const totalCount = result[0]?.totalCount || 0;

      return {
        data: result.map(({ totalCount, ...rest }) => rest),
        totalPages: Math.ceil(totalCount / limit),
        totalCount: totalCount,
      };
    } catch (error) {
      throw error;
    }
  },

  async fetchReadyBorrowCount(): Promise<number> {
    const sql = `
            SELECT COUNT(*) OVER() as count
            FROM (
                SELECT device_kinds.id
                FROM devices
                JOIN device_kinds ON devices.kind = device_kinds.id
                LEFT JOIN labs ON devices.lab_id = labs.id
                WHERE devices.status::text = $1
                    AND devices.deleted_at IS NULL
                GROUP BY 
                    device_kinds.id,
                    device_kinds.name,
                    device_kinds.image,
                    labs.name
            ) subquery
        `;

    const result = await db.queryRaw<{ count: number }>({
      sql,
      params: [DeviceStatus.HEALTHY],
    });

    return result[0]?.count ?? 0;
  },

  async fetchBorrowingCount(): Promise<number> {
    const sql = `
            SELECT COUNT(*) OVER() as count
            FROM (
                SELECT receipts.id
                FROM receipts_devices
                JOIN receipts ON receipts_devices.receipt_id = receipts.id
                JOIN users ON receipts.borrower_id = users.id
                JOIN labs ON receipts.borrowed_lab_id = labs.id
                JOIN activities ON receipts_devices.borrow_id = activities.id
                WHERE users.deleted_at IS NULL
                    AND receipts_devices.return_id IS NULL
                GROUP BY 
                    receipts.id,
                    users.name,
                    users.image,
                    labs.name,
                    activities.created_at,
                    receipts_devices.expected_returned_at,
                    receipts_devices.return_checker_id
            ) subquery
        `;

    const result = await db.queryRaw<{ count: number }>({
      sql,
      params: [],
    });

    return result[0]?.count ?? 0;
  },

  async fetchReturnedCount(): Promise<number> {
    const sql = `
            SELECT COUNT(*) OVER() as count
            FROM (
                SELECT receipts.id
                FROM receipts_devices
                JOIN receipts ON receipts_devices.receipt_id = receipts.id
                JOIN users ON receipts_devices.return_checker_id = users.id
                JOIN labs ON receipts.borrowed_lab_id = labs.id
                JOIN activities ON receipts_devices.return_id = activities.id
                WHERE users.deleted_at IS NULL
                    AND receipts_devices.return_id IS NOT NULL
                GROUP BY 
                    receipts.id,
                    users.name,
                    users.image,
                    labs.name,
                    activities.created_at,
                    receipts_devices.expected_returned_at,
                    receipts_devices.note
            ) subquery
        `;

    const result = await db.queryRaw<{ count: number }>({
      sql,
      params: [],
    });

    return result[0]?.count ?? 0;
  },

  async createReceipt(params: CreateReceiptParams) {
    try {
      if (!params.devices || params.devices.length === 0) {
        throw new Error("No devices specified for borrowing");
      }

      const deviceValues = params.devices
        .map((item) => {
          const prevQuality = item.prevQuality || DeviceQuality.HEALTHY;
          const expectedReturnedLabId =
            item.expectedReturnedLabId || params.borrowedLabId;
          return `'${item.id}', '${item.expectedReturnedAt.toISOString()}'::timestamptz, '${expectedReturnedLabId}'::uuid, '${prevQuality}'::device_quality`;
        })
        .join("),(");

      await db.queryRaw<{ id: string }>({
        sql: `
          WITH receipt AS (
            INSERT INTO receipts (id, borrower_id, borrow_checker_id, borrowed_lab_id)
            VALUES ('${params.id}', '${params.borrowerId}', '${params.borrowCheckerId}', '${params.borrowedLabId}'::uuid)
            RETURNING id
          ),
          activity AS (
            INSERT INTO activities (type)
            VALUES ('borrow'::activity_type)
            RETURNING id
          ),
          receipts_devices_insert AS (
            INSERT INTO receipts_devices (
              receipt_id,
              device_id,
              borrow_id,
              expected_returned_at,
              expected_returned_lab_id,
              prev_quality
            )
            SELECT 
              '${params.id}',
              v.device_id,
              (SELECT id FROM activity),
              v.expected_returned_at,
              v.expected_returned_lab_id,
              v.prev_quality
            FROM (
              VALUES (${deviceValues})
            ) AS v(device_id, expected_returned_at, expected_returned_lab_id, prev_quality)
            RETURNING device_id
          )
          UPDATE devices 
          SET status = 'borrowing'::device_status 
          WHERE id IN (SELECT device_id FROM receipts_devices_insert)
        `,
      });

      return { success: true, id: params.id };
    } catch (error) {
      throw error;
    }
  },

  async returnReceipt() {},
};
