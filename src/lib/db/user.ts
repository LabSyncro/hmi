import { db } from "./client";

export type UserDetail = {
  id: string;
  name: string;
  email: string;
  tel: string;
  avatar: string;
  lastActiveAt: Date | null;
  roles: {
    name: string;
    key: string;
  }[];
};

type OneTimeQrCodeParams = {
  token: string;
  userId: string;
  timestamp: number;
};

export const userService = {
  async getUserById(id: string): Promise<UserDetail | null> {
    try {
      const sql = `
                SELECT 
                    u.id,
                    u.name,
                    u.email,
                    u.tel,
                    u.image as avatar,
                    u.last_active_at,
                    COALESCE(
                        json_agg(
                        json_build_object(
                            'name', r.name,
                            'key', r.key
                        )
                        ) FILTER (WHERE r.id IS NOT NULL),
                        '[]'
                    ) as roles
                FROM users u
                LEFT JOIN user_roles ur ON u.id = ur.user_id
                LEFT JOIN roles r ON ur.role_id = r.id
                WHERE 
                    u.deleted_at IS NULL 
                    AND u.id = $1
                GROUP BY 
                    u.id, 
                    u.name, 
                    u.email, 
                    u.tel,
                    u.image, 
                    u.last_active_at
            `;

      const results = await db.queryRaw<UserDetail>({
        sql,
        params: [id],
      });

      if (results.length === 0) {
        return null;
      }

      const row = results[0];

      const userDetail: UserDetail = {
        id: row.id as string,
        name: row.name as string,
        email: row.email as string,
        tel: row.tel as string,
        avatar: row.avatar as string,
        lastActiveAt: row.lastActiveAt as Date | null,
        roles: row.roles as { name: string; key: string }[],
      };

      return userDetail;
    } catch (error) {
      throw error;
    }
  },

  async checkOneTimeQrCode(params: OneTimeQrCodeParams) {
    const { token, userId, timestamp } = params;

    const existingTokenSQL = `
      SELECT EXISTS (
        SELECT 1 
        FROM used_qr_tokens 
        WHERE token = $1 AND user_id = $2
      ) AS token_exists;
    `;

    const existingToken = await db.queryRaw<{ token_exists: boolean }>({
      sql: existingTokenSQL,
      params: [token, userId],
    });

    if (existingToken[0].token_exists) {
      throw "Mã QR đã được sử dụng";
    }

    const userSQL = `
        SELECT 
            u.id,
            u.name,
            u.email,
            u.tel,
            u.image as avatar,
            u.last_active_at,
            COALESCE(
                json_agg(
                json_build_object(
                    'name', r.name,
                    'key', r.key
                )
                ) FILTER (WHERE r.id IS NOT NULL),
                '[]'
            ) as roles
        FROM users u
        LEFT JOIN user_roles ur ON u.id = ur.user_id
        LEFT JOIN roles r ON ur.role_id = r.id
        WHERE 
            u.deleted_at IS NULL 
            AND u.id = $1
        GROUP BY 
            u.id, 
            u.name, 
            u.email, 
            u.tel,
            u.image, 
            u.last_active_at
    `;

    const user = await db.queryRaw<UserDetail>({
      sql: userSQL,
      params: [userId],
    });

    if (!user) {
      throw "Không tìm thấy người dùng";
    }

    try {
      await db.table("used_qr_tokens").insert({
        token: token,
        user_id: userId,
        created_at: timestamp,
      });
    } catch (_error) {
      throw "Mã QR đã được sử dụng";
    }

    return { user: user[0] };
  },

  async getBorrowedHistoryByUser(
    userId: string
  ): Promise<UserBorrowHistoryItem[]> {
    if (!userId) {
      throw new Error("Missing user ID");
    }

    try {
      const sql = `
        SELECT 
          r.id AS "receiptId",
          d.id AS "deviceId",
          d.kind AS "deviceKindId",
          dk.name AS "deviceName",
          dk.image AS "deviceImage",
          dk.is_borrowable_lab_only,
          bl.id AS "labId",
          bl.room AS "labRoom",
          bl.branch AS "labBranch",
          a_borrow.created_at AS "borrowDate",
          rd.expected_returned_at AS "expectedReturnedAt",
          CASE
            WHEN rd.expected_returned_at < NOW() THEN 'OVERDUE'
            WHEN rd.expected_returned_at < NOW() + INTERVAL '3 days' THEN 'NEAR_DUE'
            ELSE 'ON_TIME'
          END AS status
        FROM 
          receipts r
          JOIN receipts_devices rd ON r.id = rd.borrowed_receipt_id
          JOIN devices d ON rd.device_id = d.id
          JOIN device_kinds dk ON d.kind = dk.id
          JOIN labs bl ON r.lab_id = bl.id
          LEFT JOIN activities a_borrow ON rd.borrow_id = a_borrow.id
        WHERE 
          r.actor_id = $1
          AND rd.returned_receipt_id IS NULL
          AND d.deleted_at IS NULL
        ORDER BY 
          a_borrow.created_at DESC
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [userId],
      });

      if (results.length === 0) {
        return [];
      }

      return results.map((row) => ({
        receiptId: row.receiptId as string,
        deviceId: row.deviceId as string,
        deviceKindId: row.deviceKindId as string,
        deviceName: row.deviceName as string,
        deviceImage:
          typeof row.deviceImage === "string"
            ? JSON.parse(row.deviceImage)
            : row.deviceImage || { mainImage: null },
        deviceBorrowableLabOnly: row.isBorrowableLabOnly as boolean,
        labId: row.labId as string,
        labRoom: row.labRoom as string,
        labBranch: row.labBranch as string,
        borrowDate: row.borrowDate as string,
        expectedReturnedAt: row.expectedReturnedAt as string,
        status: row.status as "ON_TIME" | "NEAR_DUE" | "OVERDUE",
      }));
    } catch (error) {
      throw error;
    }
  },

  async getUserActivitiesHistory(userId: string): Promise<UserActivityItem[]> {
    if (!userId) {
      throw new Error("Missing user ID");
    }

    try {
      const sql = `
        WITH audit_activities AS (
          SELECT 
            ia.id::text,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            l.room || ', ' || l.branch AS location,
            a.created_at AS activity_date,
            ia.status::text,
            a.note,
            'AUDIT' AS activity_type,
            NULL as prev_quality,
            NULL as after_quality
          FROM 
            inventory_assessments ia
            JOIN inventory_assessments_devices iad ON ia.id = iad.assessing_id
            JOIN devices d ON iad.device_id = d.id
            JOIN device_kinds dk ON d.kind = dk.id
            JOIN labs l ON ia.lab_id = l.id
            JOIN activities a ON ia.id = a.id
          WHERE 
            ia.accountant_id = $1
            AND d.deleted_at IS NULL
        ),
        maintenance_activities AS (
          SELECT 
            m.id::text,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            l.room || ', ' || l.branch AS location,
            a.created_at AS activity_date,
            m.status::text,
            a.note,
            'MAINTENANCE' AS activity_type,
            NULL as prev_quality,
            NULL as after_quality
          FROM 
            maintenances m
            JOIN maintenances_devices md ON m.id = md.maintaining_id
            JOIN devices d ON md.device_id = d.id
            JOIN device_kinds dk ON d.kind = dk.id
            JOIN labs l ON d.lab_id = l.id
            JOIN activities a ON m.id = a.id
          WHERE 
            m.maintainer_id = $1
            AND d.deleted_at IS NULL
        ),
        transport_activities AS (
          SELECT 
            s.id::text,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            start_lab.room || ', ' || start_lab.branch || ' → ' || arrive_lab.room || ', ' || arrive_lab.branch AS location,
            a.created_at AS activity_date,
            s.status::text,
            a.note,
            'TRANSPORT' AS activity_type,
            NULL as prev_quality,
            NULL as after_quality
          FROM 
            shipments s
            JOIN shipments_devices sd ON s.id = sd.shipment_id
            JOIN devices d ON sd.device_id = d.id
            JOIN device_kinds dk ON d.kind = dk.id
            JOIN labs start_lab ON s.start_lab_id = start_lab.id
            JOIN labs arrive_lab ON s.arrive_lab_id = arrive_lab.id
            JOIN activities a ON s.from_at = a.id
          WHERE 
            (s.sender_id = $1 OR s.receiver_id = $1)
            AND d.deleted_at IS NULL
        ),
        returned_devices_activities AS (
          SELECT 
            rd.returned_receipt_id::text AS id,
            d.id AS device_id,
            d.kind AS device_kind_id,
            dk.name AS device_name,
            dk.image AS device_image,
            l.room || ', ' || l.branch AS location,
            a.created_at AS activity_date,
            'returned' AS status,
            a.note,
            'RETURNED' AS activity_type,
            rd.prev_quality::text as prev_quality,
            rd.after_quality::text as after_quality
          FROM 
            receipts_devices rd
            JOIN devices d ON rd.device_id = d.id
            JOIN device_kinds dk ON d.kind = dk.id
            JOIN receipts r ON rd.returned_receipt_id = r.id
            JOIN labs l ON r.lab_id = l.id
            JOIN activities a ON rd.return_id = a.id
          WHERE 
            rd.returned_receipt_id IS NOT NULL
            AND (r.actor_id = $1 OR r.checker_id = $1)
            AND d.deleted_at IS NULL
        )
        
        SELECT * FROM audit_activities
        UNION ALL
        SELECT * FROM maintenance_activities
        UNION ALL
        SELECT * FROM transport_activities
        UNION ALL
        SELECT * FROM returned_devices_activities
        ORDER BY activity_date DESC
      `;

      const results = await db.queryRaw<Record<string, unknown>>({
        sql,
        params: [userId],
      });

      if (results.length === 0) {
        return [];
      }

      return results.map((row) => ({
        id: row.id as string,
        type: row.activityType as
          | "AUDIT"
          | "MAINTENANCE"
          | "TRANSPORT"
          | "RETURNED",
        deviceId: row.deviceId as string,
        deviceKindId: row.deviceKindId as string,
        deviceName: row.deviceName as string,
        deviceImage:
          typeof row.deviceImage === "string"
            ? JSON.parse(row.deviceImage)
            : row.deviceImage || { mainImage: null },
        location: row.location as string,
        date: row.activityDate as string,
        status: row.status as string,
        note: row.note as string | undefined,
        prevQuality: row.prevQuality as string | null,
        afterQuality: row.afterQuality as string | null,
      }));
    } catch (error) {
      throw error;
    }
  },
};
