import { db } from "./client";
import { DeviceStatus } from "@/types/db/generated";

export const searchService = {
  async getAccessoriesForDeviceKind(
    kindId: string,
    labId?: string | string[]
  ): Promise<Accessory[]> {
    if (!kindId) {
      throw new Error("Device kind ID is required");
    }

    try {
      const labCondition = labId
        ? Array.isArray(labId)
          ? `AND d.lab_id IN (${labId.map((_, i) => `$${i + 2}`).join(",")})`
          : `AND d.lab_id = $2`
        : "";

      const params = [kindId];
      if (labId) {
        if (Array.isArray(labId)) {
          params.push(...labId);
        } else {
          params.push(labId);
        }
      }

      const sql = `
        SELECT 
          d.id,
          d.full_id,
          d.status,
          dk.image,
          dk.name,
          dk.brand,
          dk.unit,
          COUNT(d.id) as quantity
        FROM 
          devices d
          JOIN device_kinds dk ON d.kind = dk.id
        WHERE 
          d.accessory_for_kind_id = $1
          ${labCondition}
          AND d.deleted_at IS NULL
        GROUP BY
          d.id, d.full_id, d.status, dk.image, dk.name, dk.brand, dk.unit
        ORDER BY
          dk.name
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params,
      });

      return results.map((row) => ({
        id: row.id as string,
        fullId: row.fullId as string,
        status: row.status as DeviceStatus,
        image: row.image,
        name: row.name as string,
        brand: row.brand as string | null,
        unit: row.unit as string | null,
        quantity: parseInt(row.quantity as string, 10),
      }));
    } catch (error) {
      throw error;
    }
  },

  async getDeviceDetailsByIdOrKind(
    deviceId?: string,
    kindId?: string,
    labId?: string
  ) {
    if (!deviceId && !kindId) {
      throw new Error("Either device ID or kind ID is required");
    }

    try {
      let whereClause = "";
      const params = [];

      if (deviceId) {
        whereClause = "d.id = $1";
        params.push(deviceId);
      } else if (kindId) {
        whereClause = "d.kind = $1";
        params.push(kindId);
      }

      if (labId) {
        whereClause += " AND d.lab_id = $2";
        params.push(labId);
      }

      const sql = `
        SELECT 
          d.id,
          d.full_id,
          d.status,
          d.kind,
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
          l.branch
        FROM 
          devices d
          LEFT JOIN device_kinds dk ON d.kind = dk.id
          LEFT JOIN labs l ON d.lab_id = l.id
          LEFT JOIN categories c ON dk.category_id = c.id
        WHERE 
          ${whereClause}
          AND d.deleted_at IS NULL
        LIMIT 1
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params,
      });

      if (results.length === 0) {
        return null;
      }

      const row = results[0];

      return {
        id: row.id as string,
        fullId: row.fullId as string,
        status: row.status as DeviceStatus,
        image: row.image,
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
      };
    } catch (error) {
      throw error;
    }
  },
};
