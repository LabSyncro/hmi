import { DeviceStatus } from "@/types/db/generated";
import { db } from "./client";

type DeviceReceiptDetail = {
  fullId: string;
  status: DeviceStatus | null;
  prevQuality?: DeviceStatus | null;
  image: any;
  unit: string;
  deviceName: string;
  allowedBorrowRoles: string[];
  allowedViewRoles: string[];
  brand: string | null;
  manufacturer: string | null;
  description: string | null;
  isBorrowableLabOnly: boolean;
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
  expectedReturnedAt?: string | null;
  borrowedLab?: string | null;
  expectedReturnLab?: string | null;
  receiptId?: string | null;
};

type DeviceAuditDetail = {
  id: string;
  fullId: string;
  status: DeviceStatus;
  currentStatus: DeviceStatus;
  auditCondition: DeviceStatus;
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

type DeviceMaintenanceDetail = {
  id: string;
  maintenanceId: string;
  technician: {
    id: string;
    name: string;
  };
  status: DeviceStatus;
  currentStatus: DeviceStatus;
  outcome: DeviceStatus;
  kind: string;
  deviceName: string;
  image: any;
  unit: string;
  isBorrowableLabOnly: boolean;
  location: string;
  notes?: string;
  createdAt: Date;
};

type DeviceShipmentDetail = {
  status: DeviceStatus | null;
  prevCondition?: DeviceStatus | null;
  afterCondition?: DeviceStatus | null;
  shipmentId?: string | null;
  sourceLocation?: string | null;
  destinationLocation?: string | null;
  senderName?: string | null;
  receiverName?: string | null;
  image: any;
  unit: string;
  deviceName: string;
  isBorrowableLabOnly: boolean;
};

type DeviceInventory = {
  availableQuantity: number;
  unscannedDeviceIds: string[];
};

type InventorySummary = {
  location: string;
  healthy: number;
  broken: number;
  discarded: number;
  lost: number;
};

type BorrowHistory = {
  id: string;
  fullId: string;
  status: string;
  borrower: {
    id: string;
    name: string;
    email: string;
    avatar?: string;
  };
  borrowDate: string;
  expectedReturnedAt: string;
  hasBeenReturned: boolean;
  returnedAt?: string;
  returnedNote?: string;
  borrowedLab: string;
  expectedReturnLab: string;
};

type AuditHistory = {
  id: string;
  fullId: string;
  auditor: {
    id: string;
    name: string;
    email: string;
    avatar?: string;
  };
  auditDate: string;
  auditResult: string;
  notes?: string;
  prevStatus?: DeviceStatus;
  afterStatus?: DeviceStatus;
};

type MaintenanceHistory = {
  id: string;
  fullId: string;
  maintenanceReason: string;
  status: string;
  technician: {
    id: string;
    name: string;
    email: string;
    avatar?: string;
  };
  maintenanceStartDate: string;
  expectedCompletionDate?: string;
  finishedAt?: string;
  notes?: string;
};

type TransportHistory = {
  id: string;
  fullId: string;
  sourceLocation: string;
  destinationLocation: string;
  transportDate: string;
  status: string;
  sender?: {
    id: string;
    name: string;
    email?: string;
    avatar?: string;
  };
  receiver?: {
    id: string;
    name: string;
    email?: string;
    avatar?: string;
  };
};

export type DeviceDetail = {
  id: string;
  fullId: string;
  status: DeviceStatus;
  image: { mainImage: string | null };
  unit: string;
  deviceName: string;
  allowedBorrowRoles: string[];
  allowedViewRoles: string[];
  brand: string | null;
  manufacturer: string | null;
  description: string | null;
  isBorrowableLabOnly: boolean;
  categoryName: string;
  labId: string | null;
  labRoom: string | null;
  labBranch: string | null;
  kind: string;
};

export const deviceService = {
  async getDeviceReceiptById(
    id: string,
    labId: string
  ): Promise<DeviceReceiptDetail> {
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

      const deviceReceiptDetail: DeviceReceiptDetail = {
        fullId: row.fullId as string,
        status: row.status as DeviceStatus | null,
        prevQuality: row.prevQuality as DeviceStatus | null,
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
        receiptId: row.receiptId as string | null,
        borrower: row.borrowerId
          ? {
              id: row.borrowerId as string,
              name: row.borrowerName as string,
              image: row.borrowerImage as string | null,
            }
          : null,
        borrowedAt: row.borrowedAt ? new Date(row.borrowedAt as string) : null,
        expectedReturnedAt: row.expectedReturnedAt as string | null,
        borrowedLab: row.borrowedLab as string | null,
        expectedReturnLab: row.expectedReturnLab as string | null,
      };

      return deviceReceiptDetail;
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
        WITH active_assessment AS (
          SELECT 
            ia.id
          FROM inventory_assessments ia
          WHERE ia.status = 'assessing'
            AND ia.finished_at IS NULL
            AND ia.lab_id = $2
          LIMIT 1
        )
        SELECT 
          d.id,
          d.full_id,
          CASE 
            WHEN d.status = 'assessing' THEN COALESCE(iad.prev_status, d.status)
            ELSE d.status
          END as status,
          iad.after_status as audit_condition,
          d.status as current_status,
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
          LEFT JOIN active_assessment aa ON true
          LEFT JOIN inventory_assessments_devices iad ON iad.device_id = d.id AND iad.assessing_id = aa.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [id, labId],
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
        currentStatus: row.currentStatus as DeviceStatus,
        auditCondition: row.auditCondition as DeviceStatus,
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

  async getDeviceMaintenanceById(
    deviceId: string,
    labId?: string
  ): Promise<DeviceMaintenanceDetail> {
    if (!deviceId) {
      throw new Error("Missing device ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          CASE 
            WHEN d.status = 'maintaining' THEN COALESCE(md.prev_status, d.status)
            ELSE d.status
          END as status,
          d.status as current_status,
          md.after_status as outcome,
          d.kind,
          d.lab_id,
          dk.image,
          dk.unit,
          dk.name AS device_name,
          dk.is_borrowable_lab_only,
          l.room,
          l.branch,
          m.id as maintenance_id,
          m.maintainer_id as technician_id,
          u.name as technician_name,
          a.note as notes,
          a.created_at
        FROM 
          devices d
          LEFT JOIN device_kinds dk ON d.kind = dk.id
          LEFT JOIN labs l ON d.lab_id = l.id
          LEFT JOIN maintenances_devices md ON d.id = md.device_id
          LEFT JOIN maintenances m ON md.maintaining_id = m.id
          LEFT JOIN users u ON m.maintainer_id = u.id
          LEFT JOIN activities a ON m.id = a.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
        ORDER BY 
          a.created_at DESC
        LIMIT 1
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [deviceId],
      });

      if (results.length === 0) {
        throw new Error("Device not found");
      }

      const row = results[0];

      if (labId && row.labId !== labId) {
        throw new Error("Device does not belong to this lab");
      }

      return {
        id: row.id as string,
        maintenanceId: row.maintenanceId as string,
        technician: {
          id: row.technicianId as string,
          name: row.technicianName as string,
        },
        status: row.status as DeviceStatus,
        currentStatus: row.currentStatus as DeviceStatus,
        outcome: row.outcome as DeviceStatus,
        kind: row.kind as string,
        deviceName: row.deviceName as string,
        image: row.image as any,
        unit: row.unit as string,
        isBorrowableLabOnly: row.isBorrowableLabOnly as boolean,
        location: row.room && row.branch ? `${row.room}, ${row.branch}` : "",
        notes: (row.notes as string) || undefined,
        createdAt: row.createdAt
          ? new Date(row.createdAt as string)
          : new Date(),
      };
    } catch (error) {
      throw error;
    }
  },

  async getDeviceShipmentById(
    deviceId: string,
    labId?: string
  ): Promise<DeviceShipmentDetail> {
    if (!deviceId) {
      throw new Error("Missing device ID");
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
          sd.prev_status as prev_condition,
          sd.after_status as after_condition,
          s.id as shipment_id,
          s.status as shipment_status,
          s_start.room || ', ' || s_start.branch AS source_location,
          s_arrive.room || ', ' || s_arrive.branch AS destination_location,
          sender.name AS sender_name,
          receiver.name AS receiver_name
        FROM 
          devices d
          LEFT JOIN device_kinds dk ON d.kind = dk.id
          LEFT JOIN labs l ON d.lab_id = l.id
          LEFT JOIN categories c ON dk.category_id = c.id
          LEFT JOIN shipments_devices sd ON d.id = sd.device_id
          LEFT JOIN shipments s ON sd.shipment_id = s.id
          LEFT JOIN labs s_start ON s.start_lab_id = s_start.id
          LEFT JOIN labs s_arrive ON s.arrive_lab_id = s_arrive.id
          LEFT JOIN users sender ON s.sender_id = sender.id
          LEFT JOIN users receiver ON s.receiver_id = receiver.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
        ORDER BY
          s.from_at DESC
        LIMIT 1
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [deviceId],
      });

      if (results.length === 0) {
        throw new Error("Device not found");
      }

      const row = results[0];

      if (labId && row.labId !== labId) {
        throw new Error("Device does not belong to this lab");
      }

      const deviceShipmentDetail: DeviceShipmentDetail = {
        status: row.status as DeviceStatus | null,
        prevCondition: row.prevCondition as DeviceStatus | null,
        afterCondition: row.afterCondition as DeviceStatus | null,
        shipmentId: row.shipmentId as string | null,
        sourceLocation: row.sourceLocation as string | null,
        destinationLocation: row.destinationLocation as string | null,
        senderName: row.senderName as string | null,
        receiverName: row.receiverName as string | null,
        image: row.image as any,
        unit: row.unit as string,
        deviceName: row.deviceName as string,
        isBorrowableLabOnly: row.isBorrowableLabOnly as boolean,
      };

      return deviceShipmentDetail;
    } catch (error) {
      throw error;
    }
  },

  async getDeviceById(id: string, labId?: string): Promise<DeviceDetail> {
    if (!id) {
      throw new Error("Missing device ID");
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
          l.branch
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

      if (labId && row.labId !== labId) {
        throw new Error("Device does not belong to this lab");
      }

      const deviceDetail: DeviceDetail = {
        id: row.id as string,
        fullId: row.fullId as string,
        status: row.status as DeviceStatus,
        image: {
          mainImage: row.image ? (row.image as any).mainImage : "",
        },
        unit: row.unit as string,
        deviceName: row.deviceName as string,
        allowedBorrowRoles: row.allowedBorrowRoles as string[],
        allowedViewRoles: row.allowedViewRoles as string[],
        brand: row.brand as string | null,
        manufacturer: row.manufacturer as string | null,
        description: row.description as string | null,
        isBorrowableLabOnly: row.isBorrowableLabOnly as boolean,
        categoryName: row.categoryName as string,
        labId: row.labId as string | null,
        labRoom: row.room as string | null,
        labBranch: row.branch as string | null,
        kind: row.kind as string,
      };

      return deviceDetail;
    } catch (error) {
      throw error;
    }
  },

  async getDeviceInventoryInAudit(
    kindId: string,
    labId: string
  ): Promise<DeviceInventory> {
    if (!kindId || !labId) {
      throw new Error("Missing kind ID or lab ID");
    }

    try {
      const sql = `
        WITH lab_devices AS (
          SELECT 
            d.id,
            d.status
          FROM devices d
          WHERE d.kind = $1 
          AND d.lab_id = $2 
          AND d.deleted_at IS NULL
        ),
        lab_inventory AS (
          SELECT 
            COUNT(*) FILTER (WHERE status IN ('healthy', 'broken', 'discarded', 'lost', 'assessing')) as available_quantity
          FROM lab_devices
        ),
        unscanned_devices AS (
          SELECT 
            id
          FROM lab_devices
          WHERE status IN ('healthy', 'broken', 'discarded', 'lost', 'assessing')
        )
        SELECT 
          li.*,
          ARRAY_AGG(ud.id) as unscanned_device_ids
        FROM lab_inventory li
        LEFT JOIN unscanned_devices ud ON TRUE
        GROUP BY 
          li.available_quantity
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [kindId, labId],
      });

      if (results.length === 0) {
        throw new Error("Device inventory not found");
      }

      const row = results[0];
      return {
        availableQuantity: (row.availableQuantity as number) || 0,
        unscannedDeviceIds: (row.unscannedDeviceIds as string[]) || [],
      };
    } catch (error) {
      throw error;
    }
  },

  async getDeviceInventoryByKind(kindId: string): Promise<InventorySummary[]> {
    if (!kindId) {
      throw new Error("Missing kind ID");
    }

    try {
      const sql = `
        SELECT 
          l.room || ', ' || l.branch AS location,
          COUNT(d.id) FILTER (WHERE d.status = 'healthy') AS healthy,
          COUNT(d.id) FILTER (WHERE d.status = 'broken') AS broken,
          COUNT(d.id) FILTER (WHERE d.status = 'discarded') AS discarded,
          COUNT(d.id) FILTER (WHERE d.status = 'lost') AS lost
        FROM 
          devices d
          JOIN labs l ON d.lab_id = l.id
        WHERE 
          d.kind = $1
          AND d.deleted_at IS NULL
        GROUP BY 
          l.room, l.branch
        ORDER BY 
          l.branch, l.room
      `;

      const results = await db.queryRaw<InventorySummary>({
        sql,
        params: [kindId],
      });

      if (results.length === 0) {
        return [];
      }

      return results;
    } catch (error) {
      throw error;
    }
  },

  async getDeviceBorrowHistory(deviceId: string): Promise<BorrowHistory[]> {
    if (!deviceId) {
      throw new Error("Missing device ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          d.full_id AS "fullId",
          CASE
            WHEN rd.returned_receipt_id IS NULL THEN 
              CASE
                WHEN rd.expected_returned_at < NOW() THEN 'OVERDUE'
                WHEN rd.expected_returned_at < NOW() + INTERVAL '3 days' THEN 'NEAR_DUE'
                ELSE 'ON_TIME'
              END
            ELSE 'RETURNED'
          END AS status,
          actor.id AS "borrower_id",
          actor.name AS "borrower_name",
          actor.email AS "borrower_email",
          actor.image AS "borrower_avatar",
          a_borrow.created_at AS "borrow_date",
          rd.expected_returned_at,
          (rd.return_id IS NOT NULL) AS has_been_returned,
          a_return.created_at AS "returned_at",
          a_return.note AS "returned_note",
          bl.room || ', ' || bl.branch AS "borrowed_lab",
          COALESCE(rl.room || ', ' || rl.branch, 'N/A') AS "expected_return_lab"
        FROM 
          devices d
          JOIN receipts_devices rd ON d.id = rd.device_id
          JOIN receipts r_borrow ON rd.borrowed_receipt_id = r_borrow.id
          JOIN users actor ON r_borrow.actor_id = actor.id
          JOIN labs bl ON r_borrow.lab_id = bl.id
          LEFT JOIN labs rl ON rd.expected_returned_lab_id = rl.id
          LEFT JOIN activities a_borrow ON rd.borrow_id = a_borrow.id
          LEFT JOIN receipts r_return ON rd.returned_receipt_id = r_return.id
          LEFT JOIN activities a_return ON rd.return_id = a_return.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
        ORDER BY 
          a_borrow.created_at DESC
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [deviceId],
      });

      if (results.length === 0) {
        return [];
      }

      return results.map((row) => ({
        id: row.id as string,
        fullId: row.fullId as string,
        status: row.status as string,
        borrower: {
          id: row.borrowerId as string,
          name: row.borrowerName as string,
          email: row.borrowerEmail as string,
          avatar: row.borrowerAvatar as string,
        },
        borrowDate: row.borrowDate as string,
        expectedReturnedAt: row.expectedReturnedAt as string,
        returnedAt: row.returnedAt as string | undefined,
        hasBeenReturned: row.hasBeenReturned as boolean,
        borrowedLab: row.borrowedLab as string,
        expectedReturnLab: row.expectedReturnLab as string,
        returnedNote: row.returnedNote as string | undefined,
      }));
    } catch (error) {
      throw error;
    }
  },

  async getDeviceMaintenanceHistory(
    deviceId: string
  ): Promise<MaintenanceHistory[]> {
    if (!deviceId) {
      throw new Error("Missing device ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          d.full_id AS "fullId",
          a.note AS "maintenanceReason",
          m.status,
          u.id AS "technician_id",
          u.name AS "technician_name",
          u.email AS "technician_email",
          u.image AS "technician_avatar",
          a.created_at AS "maintenanceStartDate",
          m.finished_at AS "finishedAt"
        FROM 
          devices d
          JOIN maintenances_devices md ON d.id = md.device_id
          JOIN maintenances m ON md.maintaining_id = m.id
          JOIN activities a ON m.id = a.id
          LEFT JOIN users u ON m.maintainer_id = u.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
        ORDER BY 
          a.created_at DESC
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [deviceId],
      });

      if (results.length === 0) {
        return [];
      }

      return results.map((row) => ({
        id: row.id as string,
        fullId: row.fullId as string,
        maintenanceReason:
          (row.maintenanceReason as string) || "Bảo trì định kỳ",
        status: row.status as string,
        technician: {
          id: row.technicianId as string,
          name: row.technicianName as string,
          email: row.technicianEmail as string,
          avatar: row.technicianAvatar as string,
        },
        maintenanceStartDate: row.maintenanceStartDate as string,
        finishedAt: row.finishedAt as string | undefined,
        expectedCompletionDate: row.finishedAt
          ? (row.finishedAt as string)
          : new Date(
              new Date(row.maintenanceStartDate as string).getTime() +
                14 * 24 * 60 * 60 * 1000
            ).toISOString(),
        notes: row.note as string | undefined,
      }));
    } catch (error) {
      throw error;
    }
  },

  async getDeviceTransportHistory(
    deviceId: string
  ): Promise<TransportHistory[]> {
    if (!deviceId) {
      throw new Error("Missing device ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          d.full_id AS "fullId",
          s.id AS "shipmentId",
          start_lab.room || ', ' || start_lab.branch AS "sourceLocation",
          arrive_lab.room || ', ' || arrive_lab.branch AS "destinationLocation",
          a_from.created_at AS "transportDate",
          s.status,
          sender.id AS "sender_id",
          sender.name AS "sender_name",
          sender.email AS "sender_email",
          sender.image AS "sender_avatar",
          receiver.id AS "receiver_id",
          receiver.name AS "receiver_name",
          receiver.email AS "receiver_email",
          receiver.image AS "receiver_avatar"
        FROM 
          devices d
          JOIN shipments_devices sd ON d.id = sd.device_id
          JOIN shipments s ON sd.shipment_id = s.id
          JOIN labs start_lab ON s.start_lab_id = start_lab.id
          JOIN labs arrive_lab ON s.arrive_lab_id = arrive_lab.id
          LEFT JOIN activities a_from ON s.from_at = a_from.id
          LEFT JOIN users sender ON s.sender_id = sender.id
          LEFT JOIN users receiver ON s.receiver_id = receiver.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
        ORDER BY 
          a_from.created_at DESC
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [deviceId],
      });

      if (results.length === 0) {
        return [];
      }

      return results.map((row) => ({
        id: row.id as string,
        fullId: row.fullId as string,
        sourceLocation: row.sourceLocation as string,
        destinationLocation: row.destinationLocation as string,
        transportDate: row.transportDate as string,
        status: row.status as string,
        sender: row.senderId
          ? {
              id: row.senderId as string,
              name: row.senderName as string,
              email: row.senderEmail as string,
              avatar: row.senderAvatar as string,
            }
          : undefined,
        receiver: row.receiverId
          ? {
              id: row.receiverId as string,
              name: row.receiverName as string,
              email: row.receiverEmail as string,
              avatar: row.receiverAvatar as string,
            }
          : undefined,
      }));
    } catch (error) {
      throw error;
    }
  },

  async getDeviceAuditHistory(deviceId: string): Promise<AuditHistory[]> {
    if (!deviceId) {
      throw new Error("Missing device ID");
    }

    try {
      const sql = `
        SELECT 
          d.id,
          d.full_id AS "fullId",
          ia.id AS "assessmentId",
          u.id AS "auditor_id",
          u.name AS "auditor_name",
          u.email AS "auditor_email",
          u.image AS "auditor_avatar",
          a.created_at AS "auditDate",
          ia.status AS "auditResult",
          a.note AS "notes",
          iad.prev_status AS "prevStatus",
          iad.after_status AS "afterStatus"
        FROM 
          devices d
          JOIN inventory_assessments_devices iad ON d.id = iad.device_id
          JOIN inventory_assessments ia ON iad.assessing_id = ia.id
          JOIN activities a ON ia.id = a.id
          LEFT JOIN users u ON ia.accountant_id = u.id
        WHERE 
          d.id = $1
          AND d.deleted_at IS NULL
        ORDER BY 
          a.created_at DESC
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [deviceId],
      });

      if (results.length === 0) {
        return [];
      }

      return results.map((row) => ({
        id: row.id as string,
        fullId: row.fullId as string,
        auditor: {
          id: row.auditorId as string,
          name: row.auditorName as string,
          email: row.auditorEmail as string,
          avatar: row.auditorAvatar as string,
        },
        auditDate: row.auditDate as string,
        auditResult: row.auditResult as string,
        notes: row.notes as string | undefined,
        prevStatus: row.prevStatus as DeviceStatus | undefined,
        afterStatus: row.afterStatus as DeviceStatus | undefined,
      }));
    } catch (error) {
      throw error;
    }
  },
};
