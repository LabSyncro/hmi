import { DeviceStatus, ShipmentStatus } from "@/types/db/generated";
import { db } from "./client";

type ShipmentResult = { id: string };

function generateUniqueId(): string {
  const now = new Date();
  const datePrefix = now.toISOString().split("T")[0].replace(/-/g, "");
  const randomSuffix = Math.floor(Math.random() * 1000000)
    .toString()
    .padStart(6, "0");
  return `${datePrefix}/${randomSuffix}`;
}

export const shipmentService = {
  async confirmInboundShipment({
    technicianId,
    sourceLabId,
    destinationLabId,
    notes,
    devices,
  }: {
    technicianId: string;
    sourceLabId: string;
    destinationLabId: string;
    notes?: string;
    devices: {
      id: string;
      status: DeviceStatus;
      inboundCondition: DeviceStatus;
    }[];
  }) {
    try {
      const shipmentId = generateUniqueId();

      const deviceRows =
        devices.length > 0
          ? devices
              .map((device) => {
                return `('${shipmentId}', '${device.id}', '${device.status}'::device_status, '${device.inboundCondition}'::device_status)`;
              })
              .join(", ")
          : "";

      const deviceIds =
        devices.length > 0
          ? devices.map((device) => `'${device.id}'`).join(", ")
          : "";

      const notesValue = notes ? `'${notes.replace(/'/g, "''")}'` : "NULL";

      const sql = `
        WITH activity_insert AS (
          INSERT INTO activities (
            type, 
            note
          )
          VALUES ('shipment', ${notesValue})
          RETURNING id
        ),
        shipment_insert AS (
          INSERT INTO shipments (
            id,
            sender_id, 
            start_lab_id, 
            arrive_lab_id, 
            status,
            from_at
          )
          VALUES (
            '${shipmentId}',
            '${technicianId}',
            '${sourceLabId}'::uuid,
            '${destinationLabId}'::uuid,
            '${ShipmentStatus.SHIPPING}'::shipment_status,
            (SELECT id FROM activity_insert)
          )
          RETURNING id
        )
        ${
          devices.length > 0
            ? `, device_rows(shipment_id, device_id, prev_status, after_status) AS (
            VALUES ${deviceRows}
          ),
          device_insert AS (
            INSERT INTO shipments_devices (
              shipment_id, 
              device_id, 
              prev_status, 
              after_status
            )
            SELECT
              dr.shipment_id,
              dr.device_id,
              dr.prev_status,
              dr.after_status
            FROM device_rows dr
            RETURNING device_id
          ),
          device_update AS (
            UPDATE devices
            SET status = '${DeviceStatus.SHIPPING}'::device_status
            WHERE id IN (${deviceIds})
            RETURNING id
          )
          SELECT id FROM device_update`
            : `SELECT id FROM shipment_insert`
        }
      `;

      await db.queryRaw<ShipmentResult>({ sql });

      return { id: shipmentId };
    } catch (error) {
      throw error;
    }
  },

  async confirmOutboundShipment({
    technicianId,
    notes,
    devices,
  }: {
    technicianId: string;
    notes?: string;
    devices: {
      id: string;
      status: DeviceStatus;
      outboundCondition: DeviceStatus;
    }[];
  }) {
    try {
      if (devices.length === 0) {
        throw new Error("No devices provided");
      }

      const deviceId = devices[0].id;

      const deviceUpdateCases = devices
        .map(
          (device) =>
            `WHEN device_id = '${device.id}' THEN '${device.outboundCondition}'::device_status`
        )
        .join(" ");

      const deviceStatusUpdates = devices
        .map(
          (device) =>
            `WHEN id = '${device.id}' THEN '${device.outboundCondition}'::device_status`
        )
        .join(" ");

      const deviceIds = devices.map((device) => `'${device.id}'`).join(", ");

      const notesValue = notes ? `'${notes.replace(/'/g, "''")}'` : "NULL";

      const sql = `
        WITH activity_insert AS (
          INSERT INTO activities (
            type, 
            note
          )
          VALUES ('shipment', ${notesValue})
          RETURNING id
        ),
        shipment_update AS (
          UPDATE shipments
          SET 
            receiver_id = '${technicianId}',
            status = '${ShipmentStatus.COMPLETED}'::shipment_status,
            to_at = (SELECT id FROM activity_insert)
          WHERE id IN (
            SELECT shipment_id 
            FROM shipments_devices 
            WHERE device_id = '${deviceId}'
          )
          RETURNING id
        ),
        shipment_device_update AS (
          UPDATE shipments_devices
          SET after_status = CASE ${deviceUpdateCases} END
          WHERE shipment_id = (SELECT id FROM shipment_update)
            AND device_id IN (${deviceIds})
          RETURNING device_id
        ),
        device_update AS (
          UPDATE devices
          SET status = CASE ${deviceStatusUpdates} END
          WHERE id IN (${deviceIds})
          RETURNING id
        )
        SELECT id FROM shipment_update
      `;

      const result = await db.queryRaw<ShipmentResult>({ sql });

      if (!result || result.length === 0) {
        throw new Error("No shipment found for the provided devices");
      }

      return { id: result[0].id };
    } catch (error) {
      throw error;
    }
  },
};
