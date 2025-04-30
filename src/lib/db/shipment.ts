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
    checkAtDestination,
  }: {
    technicianId: string;
    sourceLabId: string;
    destinationLabId: string;
    notes?: string;
    devices: {
      id: string;
      inboundCondition: DeviceStatus;
    }[];
    checkAtDestination?: boolean;
  }) {
    try {
      const shipmentId = generateUniqueId();

      const deviceRows =
        devices.length > 0
          ? devices
              .map((device) => {
                return `('${shipmentId}', '${device.id}', '${device.inboundCondition}'::device_status)`;
              })
              .join(", ")
          : "";

      const deviceIds =
        devices.length > 0
          ? devices.map((device) => `'${device.id}'`).join(", ")
          : "";

      const notesValue = notes ? `'${notes.replace(/'/g, "''")}'` : "NULL";
      const checkAtDestinationValue = checkAtDestination ? "true" : "false";

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
            from_at,
            check_at_destination
          )
          VALUES (
            '${shipmentId}',
            '${technicianId}',
            '${sourceLabId}'::uuid,
            '${destinationLabId}'::uuid,
            '${ShipmentStatus.SHIPPING}'::shipment_status,
            (SELECT id FROM activity_insert),
            ${checkAtDestinationValue}
          )
          RETURNING id
        )
        ${
          devices.length > 0
            ? `, device_rows(shipment_id, device_id, prev_status) AS (
            VALUES ${deviceRows}
          ),
          device_insert AS (
            INSERT INTO shipments_devices (
              shipment_id, 
              device_id, 
              prev_status
            )
            SELECT
              dr.shipment_id,
              dr.device_id,
              dr.prev_status
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
    shipmentId,
    notes,
    devices,
  }: {
    technicianId: string;
    shipmentId: string;
    notes?: string;
    devices: {
      id: string;
      outboundCondition: DeviceStatus;
    }[];
  }) {
    try {
      if (devices.length === 0) {
        throw new Error("No devices provided");
      }

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
          WHERE id = '${shipmentId}'
          RETURNING id
        ),
        shipment_device_update AS (
          UPDATE shipments_devices
          SET after_status = CASE ${deviceUpdateCases} END
          WHERE shipment_id = '${shipmentId}'
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
        throw new Error("No shipment found for the provided ID");
      }

      return { id: result[0].id };
    } catch (error) {
      throw error;
    }
  },

  async getShipmentById(id: string) {
    try {
      const sql = `
        SELECT 
          s.id,
          s.sender_id,
          s.receiver_id,
          s.start_lab_id,
          s.arrive_lab_id,
          s.status,
          s.check_at_destination,
          sl.room as source_room,
          sl.branch as source_branch,
          dl.room as destination_room,
          dl.branch as destination_branch,
          sender.name as sender_name,
          receiver.name as receiver_name,
          a_from.created_at as from_date,
          a_to.created_at as to_date,
          a_from.note as from_note,
          a_to.note as to_note
        FROM 
          shipments s
        LEFT JOIN 
          labs sl ON s.start_lab_id = sl.id
        LEFT JOIN 
          labs dl ON s.arrive_lab_id = dl.id
        LEFT JOIN 
          users sender ON s.sender_id = sender.id
        LEFT JOIN 
          users receiver ON s.receiver_id = receiver.id
        LEFT JOIN 
          activities a_from ON s.from_at = a_from.id
        LEFT JOIN 
          activities a_to ON s.to_at = a_to.id
        WHERE 
          s.id = '${id}'
      `;

      const result = await db.queryRaw<{
        id: string;
        sender_id: string;
        receiver_id: string | null;
        start_lab_id: string;
        arrive_lab_id: string;
        status: ShipmentStatus;
        check_at_destination: boolean;
        source_room: string;
        source_branch: string;
        destination_room: string;
        destination_branch: string;
        sender_name: string;
        receiver_name: string | null;
        from_date: string;
        to_date: string | null;
        from_note: string | null;
        to_note: string | null;
      }>({ sql });

      if (!result || result.length === 0) {
        return null;
      }

      return result[0];
    } catch (error) {
      throw error;
    }
  },

  async getShipmentDevices(shipmentId: string) {
    try {
      const sql = `
        SELECT 
          sd.shipment_id,
          sd.device_id,
          sd.prev_status,
          sd.after_status,
          d.id as device_id,
          d.status,
          d.kind_id,
          dk.name as device_name,
          dk.unit,
          dk.is_borrowable_lab_only as is_borrowable_lab_only,
          di.main_image
        FROM 
          shipments_devices sd
        JOIN 
          devices d ON sd.device_id = d.id
        JOIN 
          device_kinds dk ON d.kind_id = dk.id
        LEFT JOIN 
          device_images di ON d.kind_id = di.device_kind_id
        WHERE 
          sd.shipment_id = '${shipmentId}'
      `;

      const result = await db.queryRaw<
        {
          shipment_id: string;
          device_id: string;
          prev_status: DeviceStatus | null;
          after_status: DeviceStatus | null;
          status: DeviceStatus;
          kind_id: string;
          device_name: string;
          unit: string;
          is_borrowable_lab_only: boolean;
          main_image: string | null;
        }[]
      >({ sql });

      return result || [];
    } catch (error) {
      throw error;
    }
  },
};
