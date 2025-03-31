import { db } from './client'

export type UserDetail = {
    id: string
    name: string
    email: string
    tel: string
    avatar: string
    lastActiveAt: Date | null
    roles: {
        name: string
        key: string
    }[]
}

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
            `

            const results = await db.queryRaw<UserDetail>({
                sql,
                params: [id]
            })

            if (results.length === 0) {
                return null
            }

            const row = results[0]
            console.log('row', row)

            const userDetail: UserDetail = {
                id: row.id as string,
                name: row.name as string,
                email: row.email as string,
                tel: row.tel as string,
                avatar: row.avatar as string,
                lastActiveAt: row.lastActiveAt as Date | null,
                roles: row.roles as { name: string, key: string }[],
            }

            return userDetail;
        } catch (error) {
            throw error
        }
    }
}