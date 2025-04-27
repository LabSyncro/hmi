import { AssessmentStatus, DeviceStatus } from "@/types/db/generated";
import { db } from "./client";

interface CreateAuditParams {
  auditorId: string;
  location: string;
  devices: {
    id: string;
    condition: DeviceStatus;
    prevStatus?: DeviceStatus;
  }[];
  status?: AssessmentStatus;
  notes?: string;
}

export const auditService = {
  async createAudit(params: CreateAuditParams) {
    try {
      const status = params.status || AssessmentStatus.ASSESSING;
      const finishedAt =
        status === AssessmentStatus.COMPLETED ? "CURRENT_TIMESTAMP" : "NULL";

      const deviceRows = params.devices
        .map((device) => {
          const prev = device.prevStatus
            ? `'${device.prevStatus}'::device_status`
            : "NULL";
          const after = `'${device.condition}'::device_status`;
          return `(${prev}, ${after}, '${device.id}')`;
        })
        .join(", ");
      const updateCases = params.devices
        .map((d) => `WHEN id = '${d.id}' THEN 'assessing'::device_status`)
        .join(" ");
      const updateIds = params.devices.map((d) => `'${d.id}'`).join(", ");

      const sql = `
        WITH activity_insert AS (
          INSERT INTO activities (type${params.notes ? ", note" : ""})
          VALUES (
            'assessment'::activity_type${params.notes ? `, '${params.notes.replace(/'/g, "''")}'` : ""}
          )
          RETURNING id
        ),
        assessment_insert AS (
          INSERT INTO inventory_assessments (
            id,
            lab_id,
            accountant_id,
            status${status === AssessmentStatus.COMPLETED ? ", finished_at" : ""}
          )
          SELECT
            id,
            '${params.location}'::uuid,
            '${params.auditorId}',
            '${status}'::assessment_status${status === AssessmentStatus.COMPLETED ? `, ${finishedAt}` : ""}
          FROM activity_insert
          RETURNING id
        ),
        device_rows(prev_status, after_status, device_id) AS (
          VALUES
            ${deviceRows}
        ),
        device_insert AS (
          INSERT INTO inventory_assessments_devices (
            prev_status,
            after_status,
            assessing_id,
            device_id
          )
          SELECT
            dr.prev_status,
            dr.after_status,
            ai.id,
            dr.device_id
          FROM device_rows dr, assessment_insert ai
          RETURNING assessing_id
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
        SELECT id FROM assessment_insert;
      `;

      const result = await db.queryRaw<{ id: string }>({ sql });
      const assessmentId = result[0]?.id;
      return { success: true, id: assessmentId };
    } catch (error) {
      throw error;
    }
  },

  async getIncompleteAudits(labId: string) {
    try {
      const sql = `
        SELECT 
          ia.id, 
          ia.status, 
          ia.accountant_id, 
          u.name as accountant_name,
          ia.lab_id, 
          l.room as lab_room,
          l.branch as lab_branch,
          a.created_at,
          (SELECT array_agg(iad.device_id)
            FROM inventory_assessments_devices iad
            WHERE iad.assessing_id = ia.id
          ) as device_ids,
          (SELECT COUNT(*) 
            FROM inventory_assessments_devices iad 
            WHERE iad.assessing_id = ia.id
          ) as device_count
        FROM 
          inventory_assessments ia
        JOIN
          activities a ON ia.id = a.id
        LEFT JOIN 
          users u ON ia.accountant_id = u.id
        LEFT JOIN 
          labs l ON ia.lab_id = l.id
        WHERE 
          ia.status = '${AssessmentStatus.ASSESSING}'
          AND ia.lab_id = '${labId}'
        ORDER BY 
          a.created_at DESC
      `;

      const result = await db.queryRaw<IncompleteAudit>({ sql });

      return result;
    } catch (error) {
      throw error;
    }
  },

  async addDeviceToAudit(
    auditId: string,
    deviceId: string,
    prevStatus: DeviceStatus,
    afterStatus: DeviceStatus
  ): Promise<void> {
    try {
      const currentAuditSql = `
        SELECT status
        FROM inventory_assessments
        WHERE id = '${auditId}'
      `;
      const currentAudit = await db.queryRaw<{ status: AssessmentStatus }>({
        sql: currentAuditSql,
      });
      if (currentAudit.length === 0) {
        throw new Error(`Audit with ID ${auditId} not found`);
      }
      if (currentAudit[0].status !== AssessmentStatus.ASSESSING) {
        return;
      }

      const sql = `
        WITH ins AS (
          INSERT INTO inventory_assessments_devices (
            prev_status,
            after_status,
            assessing_id,
            device_id
          )
          VALUES (
            '${prevStatus}'::device_status,
            '${afterStatus}'::device_status,
            '${auditId}',
            '${deviceId}'
          )
          RETURNING id
        ),
        upd AS (
          UPDATE devices
          SET status = 'assessing'::device_status
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
    auditId: string,
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
        UPDATE inventory_assessments_devices
        SET after_status = du.new_condition
        FROM device_updates du
        WHERE inventory_assessments_devices.assessing_id = '${auditId}'
          AND inventory_assessments_devices.device_id = du.device_id
      `;

      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async updateDeviceCondition(
    auditId: string,
    deviceId: string,
    condition: DeviceStatus
  ): Promise<void> {
    try {
      const sql = `
        WITH update_device AS (
          UPDATE inventory_assessments_devices
          SET after_status = '${condition}'::device_status
          WHERE assessing_id = '${auditId}'
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
  async addUnscannedDevices(
    auditId: string,
    unscannedItems: {
      deviceId: string;
      condition: DeviceStatus;
    }[]
  ): Promise<void> {
    try {
      if (!unscannedItems.length) return;

      const deviceValues = unscannedItems
        .map(
          (item) => `('${item.deviceId}', '${item.condition}'::device_status)`
        )
        .join(", ");

      const sql = `
        WITH device_info(device_id, desired_condition) AS (
          VALUES ${deviceValues}
        ),
        available_devices AS (
          SELECT 
            di.device_id,
            di.desired_condition,
            d.status as current_status
          FROM device_info di
          JOIN devices d ON di.device_id = d.id
          LEFT JOIN inventory_assessments_devices iad ON 
            iad.assessing_id = '${auditId}' AND iad.device_id = di.device_id
          WHERE 
            iad.device_id IS NULL
            AND d.deleted_at IS NULL
            AND d.status != 'assessing'::device_status
        ),
        insert_records AS (
          INSERT INTO inventory_assessments_devices (assessing_id, device_id, prev_status, after_status)
          SELECT 
            '${auditId}',
            device_id,
            current_status::device_status,
            desired_condition::device_status
          FROM available_devices
          RETURNING device_id
        )
        UPDATE devices
        SET status = 'assessing'::device_status
        WHERE id IN (SELECT device_id FROM insert_records)
      `;

      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async removeDeviceFromAudit(
    auditId: string,
    deviceId: string
  ): Promise<void> {
    try {
      const sql = `
        WITH device_info AS (
          SELECT prev_status 
          FROM inventory_assessments_devices
          WHERE assessing_id = '${auditId}'
            AND device_id = '${deviceId}'
        ),
        delete_device AS (
          DELETE FROM inventory_assessments_devices
          WHERE assessing_id = '${auditId}'
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

  async completeAudit(auditId: string, notes?: string): Promise<void> {
    try {
      const notesValue = notes ? `'${notes.replace(/'/g, "''")}'` : "NULL";

      const sql = `
        WITH update_assessment AS (
          UPDATE inventory_assessments
          SET 
            status = '${AssessmentStatus.COMPLETED}'::assessment_status,
            finished_at = CURRENT_TIMESTAMP
          WHERE id = '${auditId}'
          RETURNING id
        ),
        device_rows AS (
          SELECT 
            device_id,
            COALESCE(after_status, prev_status, '${DeviceStatus.HEALTHY}'::device_status) AS status_to_set
          FROM inventory_assessments_devices
          WHERE assessing_id = '${auditId}'
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
          WHERE id = '${auditId}'
          RETURNING id
        )
        SELECT id FROM update_assessment
      `;
      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },

  async cancelAudit(auditId: string): Promise<void> {
    try {
      const sql = `
        WITH upd_assess AS (
          UPDATE inventory_assessments
          SET
            status = '${AssessmentStatus.CANCELLED}'::assessment_status,
            finished_at = CURRENT_TIMESTAMP
          WHERE id = '${auditId}'
          RETURNING id
        ),
        device_rows AS (
          SELECT
            device_id,
            COALESCE(prev_status, '${DeviceStatus.HEALTHY}'::device_status) AS status_to_set
          FROM inventory_assessments_devices
          WHERE assessing_id = '${auditId}'
        ),
        upd_devices AS (
          UPDATE devices
          SET status = dr.status_to_set::device_status
          FROM device_rows dr
          WHERE devices.id = dr.device_id
          RETURNING devices.id
        )
        SELECT id FROM upd_assess;
      `;
      await db.queryRaw({ sql });
    } catch (error) {
      throw error;
    }
  },
};
