import { DeviceStatus, MaintenanceStatus } from "@/types/db/generated";
import { db } from "./client";

type CreateMaintenanceParams = {
  technicianId: string;
  location: string;
  notes?: string;
  status?: MaintenanceStatus;
  devices: {
    id: string;
    maintenanceOutcome: DeviceStatus;
    prevStatus?: DeviceStatus;
  }[];
};

export const maintenanceService = {
  async createMaintenance(
    params: CreateMaintenanceParams
  ): Promise<{ id: string; success: boolean }> {
    try {
      const status = params.status || MaintenanceStatus.MAINTAINING;
      const finishedAt =
        status === MaintenanceStatus.COMPLETED ? "CURRENT_TIMESTAMP" : "NULL";

      const deviceRows = params.devices
        .map((device) => {
          const prev = device.prevStatus
            ? `'${device.prevStatus}'::device_status`
            : "NULL";
          const after = `'${device.maintenanceOutcome}'::device_status`;
          return `(${prev}, ${after}, '${device.id}')`;
        })
        .join(", ");
      const updateCases = params.devices
        .map(
          (device) =>
            `WHEN id = '${device.id}' THEN 'maintaining'::device_status`
        )
        .join(" ");
      const updateIds = params.devices
        .map((device) => `'${device.id}'`)
        .join(", ");

      const sql = `
        WITH activity_insert AS (
          INSERT INTO activities (type${params.notes ? ", note" : ""})
          VALUES (
            'maintenance'::activity_type${params.notes ? `, '${params.notes.replace(/'/g, "''")}'` : ""}
          )
          RETURNING id
        ),
        maintenance_insert AS (
          INSERT INTO maintenances (
            id,
            lab_id,
            maintainer_id,
            status${status === MaintenanceStatus.COMPLETED ? ", finished_at" : ""}
          )
          SELECT
            id,
            '${params.location}'::uuid,
            '${params.technicianId}',
            '${status}'::maintenance_status${status === MaintenanceStatus.COMPLETED ? `, ${finishedAt}` : ""}
          FROM activity_insert
          RETURNING id
        ),
        device_rows(prev_status, after_status, device_id) AS (
          VALUES
            ${deviceRows}
        ),
        device_insert AS (
          INSERT INTO maintenances_devices (
            prev_status,
            after_status,
            maintaining_id,
            device_id
          )
          SELECT
            dr.prev_status,
            dr.after_status,
            mi.id,
            dr.device_id
          FROM device_rows dr, maintenance_insert mi
          RETURNING maintaining_id
        )${
          updateCases
            ? `,
        device_update AS (
          UPDATE devices
          SET status = CASE
            ${updateCases}
            ELSE status
          END
          WHERE id IN (${updateIds})
        )`
            : ""
        }
        SELECT id FROM maintenance_insert;
      `;

      const result = await db.queryRaw<{ id: string }>({ sql });
      const maintenanceId = result[0]?.id;

      if (!maintenanceId) {
        throw new Error("Failed to create maintenance record");
      }

      return { id: maintenanceId, success: true };
    } catch (error) {
      throw error;
    }
  },

  async getIncompleteMaintenance(labId: string) {
    try {
      const sql = `
        SELECT 
          m.id, 
          m.status, 
          m.maintainer_id, 
          u.name as maintainer_name,
          m.lab_id, 
          l.room as lab_room,
          l.branch as lab_branch,
          a.created_at,
          a.note,
          (SELECT array_agg(md.device_id)
            FROM maintenances_devices md
            WHERE md.maintaining_id = m.id
          ) as device_ids,
          (SELECT COUNT(*) 
            FROM maintenances_devices md 
            WHERE md.maintaining_id = m.id
          ) as device_count
        FROM 
          maintenances m
        JOIN
          activities a ON m.id = a.id
        LEFT JOIN 
          users u ON m.maintainer_id = u.id
        LEFT JOIN 
          labs l ON m.lab_id = l.id
        WHERE 
          m.status = '${MaintenanceStatus.MAINTAINING}'
          AND m.lab_id = '${labId}'
        ORDER BY 
          a.created_at DESC
      `;

      const result = await db.queryRaw<Record<string, any>>({ sql });

      return result.map((row) => ({
        id: row.id,
        status: row.status,
        maintainerId: row.maintainerId,
        maintainerName: row.maintainerName,
        labId: row.labId,
        labRoom: row.labRoom,
        labBranch: row.labBranch,
        notes: row.note,
        createdAt: row.createdAt ? new Date(row.createdAt) : new Date(),
        finishedAt: null,
        deviceCount: parseInt(row.deviceCount, 10),
        deviceIds: row.deviceIds || [],
      }));
    } catch (error) {
      throw error;
    }
  },

  async addDeviceToMaintenance(
    maintenanceId: string,
    deviceId: string,
    prevStatus: DeviceStatus,
    afterStatus: DeviceStatus
  ): Promise<void> {
    try {
      const currentMaintenanceSql = `
        SELECT status
        FROM maintenances
        WHERE id = '${maintenanceId}'
      `;
      const currentMaintenance = await db.queryRaw<{
        status: MaintenanceStatus;
      }>({
        sql: currentMaintenanceSql,
      });
      if (currentMaintenance.length === 0) {
        throw new Error(`Maintenance with ID ${maintenanceId} not found`);
      }
      if (currentMaintenance[0].status !== MaintenanceStatus.MAINTAINING) {
        return;
      }

      const sql = `
        WITH ins AS (
          INSERT INTO maintenances_devices (
            prev_status,
            after_status,
            maintaining_id,
            device_id
          )
          VALUES (
            '${prevStatus}'::device_status,
            '${afterStatus}'::device_status,
            '${maintenanceId}',
            '${deviceId}'
          )
          RETURNING id
        ),
        upd AS (
          UPDATE devices
          SET status = 'maintaining'::device_status
          WHERE id = '${deviceId}'
          RETURNING id
        )
        SELECT * FROM ins;
      `;
      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async updateListDeviceConditions(
    maintenanceId: string,
    devices: {
      id: string;
      condition: DeviceStatus;
    }[]
  ): Promise<void> {
    try {
      if (!devices.length) return;

      const values = devices
        .map((d) => `('${d.id}', '${d.condition}'::device_status)`)
        .join(", ");

      const sql = `
        WITH device_updates(device_id, new_condition) AS (
          VALUES ${values}
        )
        UPDATE maintenances_devices
        SET after_status = du.new_condition
        FROM device_updates du
        WHERE maintenances_devices.maintaining_id = '${maintenanceId}'
          AND maintenances_devices.device_id = du.device_id
      `;

      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async updateDeviceCondition(
    maintenanceId: string,
    deviceId: string,
    condition: DeviceStatus
  ): Promise<void> {
    try {
      const sql = `
        WITH update_device AS (
          UPDATE maintenances_devices
          SET after_status = '${condition}'::device_status
          WHERE maintaining_id = '${maintenanceId}'
            AND device_id = '${deviceId}'
          RETURNING device_id
        )
        SELECT device_id FROM update_device
      `;
      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async removeDeviceFromMaintenance(
    maintenanceId: string,
    deviceId: string
  ): Promise<void> {
    try {
      const sql = `
        WITH device_info AS (
          SELECT prev_status 
          FROM maintenances_devices
          WHERE maintaining_id = '${maintenanceId}'
            AND device_id = '${deviceId}'
        ),
        delete_device AS (
          DELETE FROM maintenances_devices
          WHERE maintaining_id = '${maintenanceId}'
            AND device_id = '${deviceId}'
          RETURNING device_id
        ),
        update_device AS (
          UPDATE devices
          SET status = COALESCE((SELECT prev_status FROM device_info), '${DeviceStatus.HEALTHY}'::device_status)
          WHERE id = '${deviceId}'
          RETURNING id
        )
        SELECT id FROM update_device
      `;

      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async completeMaintenance(
    maintenanceId: string,
    notes?: string
  ): Promise<void> {
    try {
      const notesValue = notes ? `'${notes.replace(/'/g, "''")}'` : "NULL";

      const sql = `
        WITH update_maintenance AS (
          UPDATE maintenances
          SET 
            status = '${MaintenanceStatus.COMPLETED}'::maintenance_status,
            finished_at = CURRENT_TIMESTAMP
          WHERE id = '${maintenanceId}'
          RETURNING id
        ),
        device_rows AS (
          SELECT 
            device_id,
            COALESCE(after_status, prev_status, '${DeviceStatus.HEALTHY}'::device_status) AS status_to_set
          FROM maintenances_devices
          WHERE maintaining_id = '${maintenanceId}'
        ),
        update_devices AS (
          UPDATE devices
          SET status = dr.status_to_set
          FROM device_rows dr
          WHERE devices.id = dr.device_id
          RETURNING id
        ),
        update_activities AS (
          UPDATE activities
          SET note = ${notesValue}
          WHERE id = '${maintenanceId}'
          RETURNING id
        )
        SELECT id FROM update_maintenance
      `;
      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async cancelMaintenance(maintenanceId: string): Promise<void> {
    try {
      const sql = `
        WITH upd_maintenance AS (
          UPDATE maintenances
          SET
            status = '${MaintenanceStatus.CANCELLED}'::maintenance_status,
            finished_at = CURRENT_TIMESTAMP
          WHERE id = '${maintenanceId}'
          RETURNING id
        ),
        device_rows AS (
          SELECT
            device_id,
            COALESCE(prev_status, '${DeviceStatus.HEALTHY}'::device_status) AS status_to_set
          FROM maintenances_devices
          WHERE maintaining_id = '${maintenanceId}'
        ),
        upd_devices AS (
          UPDATE devices
          SET status = dr.status_to_set::device_status
          FROM device_rows dr
          WHERE devices.id = dr.device_id
          RETURNING devices.id
        )
        SELECT id FROM upd_maintenance;
      `;
      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },
};
