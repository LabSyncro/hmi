import { DeviceStatus } from "@/types/db/generated";
import { db } from "./client";

export type AuditRecord = {
  deviceId: string;
  auditorId: string;
  auditCondition: DeviceStatus;
  location: string;
  notes?: string;
};

interface CreateAuditParams {
  id: string;
  auditorId: string;
  location: string;
  devices: {
    id: string;
    condition: DeviceStatus;
  }[];
  notes?: string;
}

export const auditService = {
  async recordAudit(records: AuditRecord[]): Promise<void> {
    try {
      // Create audit activity
      const activitySql = `
        INSERT INTO activities (type, created_at)
        VALUES ('audit', CURRENT_TIMESTAMP)
        RETURNING id
      `;
      const activity = await db.queryRaw<{ id: string }>({ sql: activitySql });
      const activityId = activity[0].id;

      // Record each device audit
      for (const record of records) {
        const auditSql = `
          INSERT INTO devices_audit (
            device_id, auditor_id, audit_id, condition, location, notes, created_at
          )
          VALUES (
            $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP
          )
        `;
        await db.queryRaw({
          sql: auditSql,
          params: [
            record.deviceId,
            record.auditorId,
            activityId,
            record.auditCondition,
            record.location,
            record.notes,
          ],
        });

        // Update device status if needed
        if (record.auditCondition !== "healthy") {
          const updateSql = `
            UPDATE devices
            SET status = $1
            WHERE id = $2
          `;
          await db.queryRaw({
            sql: updateSql,
            params: [record.auditCondition, record.deviceId],
          });
        }
      }
    } catch (error) {
      throw error;
    }
  },

  async createAudit(params: CreateAuditParams) {
    try {
      if (!params.devices || params.devices.length === 0) {
        throw new Error("No devices specified for audit");
      }

      // Create audit activity
      const activitySql = `
        INSERT INTO activities (type, created_at)
        VALUES ('audit', CURRENT_TIMESTAMP)
        RETURNING id
      `;
      const activity = await db.queryRaw<{ id: string }>({ sql: activitySql });
      const activityId = activity[0].id;

      // Record each device audit
      for (const device of params.devices) {
        const auditSql = `
          INSERT INTO devices_audit (
            device_id, auditor_id, audit_id, condition, location, notes, created_at
          )
          VALUES (
            $1, $2, $3, $4, $5, $6, CURRENT_TIMESTAMP
          )
        `;
        await db.queryRaw({
          sql: auditSql,
          params: [
            device.id,
            params.auditorId,
            activityId,
            device.condition,
            params.location,
            params.notes,
          ],
        });

        // Update device status if needed
        if (device.condition !== "healthy") {
          const updateSql = `
            UPDATE devices
            SET status = $1
            WHERE id = $2
          `;
          await db.queryRaw({
            sql: updateSql,
            params: [device.condition, device.id],
          });
        }
      }

      return { success: true, id: params.id };
    } catch (error) {
      throw error;
    }
  },

  async getDeviceAuditHistory(deviceId: string): Promise<
    {
      auditorName: string;
      auditorImage: string | null;
      condition: DeviceStatus;
      location: string;
      notes?: string;
      createdAt: Date;
    }[]
  > {
    try {
      const sql = `
        SELECT 
          u.name as auditor_name,
          u.image as auditor_image,
          da.condition,
          da.location,
          da.notes,
          da.created_at
        FROM 
          devices_audit da
          JOIN users u ON da.auditor_id = u.id
        WHERE 
          da.device_id = $1
        ORDER BY 
          da.created_at DESC
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
