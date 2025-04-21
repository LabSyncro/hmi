import { DeviceQuality, DeviceStatus } from "@/types/db/generated";
import { db } from "./client";

export type DeviceDetail = {
  fullId: string;
  status: DeviceStatus | null;
  prevQuality?: DeviceQuality | null;
  image: any;
  unit: string;
  deviceName: string;
  allowedBorrowRoles: string[];
  allowedViewRoles: string[];
  isBorrowableLabOnly: boolean;
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
  expectedReturnAt?: string | null;
  borrowedLab?: string | null;
  expectedReturnLab?: string | null;
  receiptId?: string | null;
};

export type DeviceInventory = {
  branch: string;
  room: string;
  borrowingQuantity: number;
  availableQuantity: number;
};

export type MaintenanceRecord = {
  deviceId: string;
  technicianId: string;
  maintenanceOutcome: DeviceStatus;
  location: string;
  notes?: string;
};

export type DeviceAuditDetail = {
  id: string;
  fullId: string;
  status: DeviceStatus;
  image: {
    mainImage: string;
  };
  unit: string;
  deviceName: string;
  isBorrowableLabOnly: boolean;
  labRoom: string;
  labBranch: string;
  kind: string;
  categoryName: string;
};

export const deviceService = {
  async getDeviceReceiptById(id: string, labId: string): Promise<DeviceDetail> {
    if (!id || !labId) {
      throw new Error("Missing device ID or lab ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          d.full_id,
          d.status,
          d.kind,
          d.lab_id,
          dk.image,
          dk.unit,
          dk.name AS device_name,
          dk.allowed_borrow_roles,
          dk.allowed_view_roles,
          dk.brand,
          dk.manufacturer,
          dk.description,
          dk.is_borrowable_lab_only,
          c.name AS category_name,
          l.room,
          l.branch,
          r.id AS receipt_id,
          a.created_at AS borrowed_at,
          rd.expected_returned_at,
          rd.prev_quality,
          bl.room || ', ' || bl.branch AS borrowed_lab,
          rl.room || ', ' || rl.branch AS expected_return_lab,
          actor.id AS borrower_id,
          actor.name AS borrower_name,
          actor.image AS borrower_image
        FROM 
          devices d
          LEFT JOIN device_kinds dk ON d.kind = dk.id
          LEFT JOIN labs l ON d.lab_id = l.id
          LEFT JOIN categories c ON dk.category_id = c.id
          LEFT JOIN receipts_devices rd ON d.id = rd.device_id AND rd.returned_receipt_id IS NULL
          LEFT JOIN receipts r ON rd.borrowed_receipt_id = r.id
          LEFT JOIN users actor ON r.actor_id = actor.id
          LEFT JOIN labs bl ON r.lab_id = bl.id
          LEFT JOIN labs rl ON rd.expected_returned_lab_id = rl.id
          LEFT JOIN activities a ON rd.borrow_id = a.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [id],
      });

      if (results.length === 0) {
        throw new Error("Device not found");
      }

      const row = results[0];

      if (row.labId !== labId) {
        throw new Error("Device does not belong to this lab");
      }

      const deviceDetail: DeviceDetail = {
        fullId: row.fullId as string,
        status: row.status as DeviceStatus | null,
        prevQuality: row.prevQuality as DeviceQuality | null,
        image: row.image as string,
        unit: row.unit as string,
        deviceName: row.deviceName as string,
        allowedBorrowRoles: row.allowedBorrowRoles as string[],
        allowedViewRoles: row.allowedViewRoles as string[],
        brand: row.brand as string | null,
        manufacturer: row.manufacturer as string | null,
        description: row.description as string | null,
        isBorrowableLabOnly: row.isBorrowableLabOnly as boolean,
        categoryName: row.categoryName as string,
        labRoom: row.room as string | null,
        labBranch: row.branch as string | null,
        kind: row.kind as string,
        receiptId: row.receiptId as string | null,
        borrower: row.borrowerId
          ? {
              id: row.borrowerId as string,
              name: row.borrowerName as string,
              image: row.borrowerImage as string | null,
            }
          : null,
        borrowedAt: row.borrowedAt ? new Date(row.borrowedAt as string) : null,
        expectedReturnAt: row.expectedReturnedAt as string | null,
        borrowedLab: row.borrowedLab as string | null,
        expectedReturnLab: row.expectedReturnLab as string | null,
      };

      return deviceDetail;
    } catch (error) {
      throw error;
    }
  },

  async getDeviceAuditById(
    id: string,
    labId: string
  ): Promise<DeviceAuditDetail> {
    if (!id || !labId) {
      throw new Error("Missing device ID or lab ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          d.full_id,
          d.status,
          d.kind,
          d.lab_id,
          dk.image,
          dk.unit,
          dk.name AS device_name,
          dk.is_borrowable_lab_only,
          l.room,
          l.branch,
          c.name AS category_name
        FROM 
          devices d
          LEFT JOIN device_kinds dk ON d.kind = dk.id
          LEFT JOIN labs l ON d.lab_id = l.id
          LEFT JOIN categories c ON dk.category_id = c.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [id],
      });

      if (results.length === 0) {
        throw new Error("Device not found");
      }

      const row = results[0];

      if (row.labId !== labId) {
        throw new Error("Device does not belong to this lab");
      }

      const deviceAuditDetail: DeviceAuditDetail = {
        id: row.id as string,
        fullId: row.fullId as string,
        status: row.status as DeviceStatus,
        image: {
          mainImage: row.image ? (row.image as any).mainImage : "",
        },
        unit: row.unit as string,
        deviceName: row.deviceName as string,
        isBorrowableLabOnly: row.isBorrowableLabOnly as boolean,
        labRoom: row.room as string,
        labBranch: row.branch as string,
        kind: row.kind as string,
        categoryName: row.categoryName as string,
      };

      return deviceAuditDetail;
    } catch (error) {
      throw error;
    }
  },

  async getDeviceInventoryByKindId(
    kindId: string,
    labId: string
  ): Promise<DeviceInventory[]> {
    if (!kindId || !labId) {
      return [];
    }

    try {
      const sql = `
        SELECT 
          l.branch,
          l.room,
          SUM(CASE WHEN d.status = 'borrowing' THEN 1 ELSE 0 END)::int as borrowing_quantity,
          SUM(CASE WHEN d.status IN ('healthy', 'broken', 'discarded', 'lost') THEN 1 ELSE 0 END)::int as available_quantity
        FROM 
          labs l
          JOIN devices d ON l.id = d.lab_id
          JOIN device_kinds dk ON d.kind = dk.id
        WHERE 
          dk.id = $1
          AND d.lab_id = $2
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
        params: [kindId, labId],
      });

      return results.map((row) => ({
        branch: row.branch as string,
        room: row.room as string,
        borrowingQuantity: row.borrowingQuantity as number,
        availableQuantity: row.availableQuantity as number,
      }));
    } catch (error) {
      throw error;
    }
  },

  async getDeviceStatusById(id: string): Promise<{
    status: DeviceStatus;
    kind: string;
    deviceName: string;
    image: any;
    unit: string;
  } | null> {
    if (!id) {
      return null;
    }
    try {
      type DeviceStatusResult = {
        status: DeviceStatus;
        kind: string;
        deviceKindsName: string;
        deviceKindsImage: any;
        deviceKindsUnit: string;
      };

      const device = await db
        .table<DeviceStatusResult>("devices")
        .select(["status", "kind"])
        .include({
          table: "device_kinds",
          select: ["name", "image", "unit"],
          on: {
            from: "kind",
            to: "id",
          },
        })
        .whereMany({
          id,
          deleted_at: null,
        })
        .first();

      if (!device || !Object.values(DeviceStatus).includes(device.status)) {
        return null;
      }

      return {
        status: device.status,
        kind: device.kind,
        deviceName: device.deviceKindsName,
        image: device.deviceKindsImage,
        unit: device.deviceKindsUnit,
      };
    } catch (error) {
      return null;
    }
  },

  async recordMaintenance(records: MaintenanceRecord[]): Promise<void> {
    try {
      // Create maintenance activity
      const activitySql = `
        INSERT INTO activities (type, created_at)
        VALUES ('maintenance', CURRENT_TIMESTAMP)
        RETURNING id
      `;
      const activity = await db.queryRaw<{ id: string }>({ sql: activitySql });
      const activityId = activity[0].id;

      // Record each device maintenance
      for (const record of records) {
        const maintenanceSql = `
          INSERT INTO devices_maintenance (
            device_id, technician_id, maintenance_id, outcome, location, notes, created_at
          )
          VALUES (
            $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP
          )
        `;
        await db.queryRaw({
          sql: maintenanceSql,
          params: [
            record.deviceId,
            record.technicianId,
            activityId,
            record.maintenanceOutcome,
            record.location,
            record.notes,
          ],
        });

        // Update device status
        const updateSql = `
          UPDATE devices
          SET status = $1
          WHERE id = $2
        `;
        await db.queryRaw({
          sql: updateSql,
          params: [record.maintenanceOutcome, record.deviceId],
        });
      }
    } catch (error) {
      throw error;
    }
  },

  async getDeviceMaintenanceHistory(deviceId: string): Promise<
    {
      technicianName: string;
      technicianImage: string | null;
      outcome: DeviceStatus;
      location: string;
      notes?: string;
      createdAt: Date;
    }[]
  > {
    try {
      const sql = `
        SELECT 
          u.name as technician_name,
          u.image as technician_image,
          dm.outcome,
          dm.location,
          dm.notes,
          dm.created_at
        FROM 
          devices_maintenance dm
          JOIN users u ON dm.technician_id = u.id
        WHERE 
          dm.device_id = $1
        ORDER BY 
          dm.created_at DESC
      `;

      return await db.queryRaw({
        sql,
        params: [deviceId],
      });
    } catch (error) {
      throw error;
    }
  },
};
