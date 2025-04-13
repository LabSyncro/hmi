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
};
