import { db } from "./client";

export interface Lab {
  id: string;
  name: string;
  room: string;
  branch: string;
}

export const labService = {
  async getAllLabs(): Promise<Lab[]> {
    try {
      const results = await db
        .table<Record<string, unknown>>("labs")
        .where("deleted_at", null)
        .orderBy("branch")
        .orderBy("room")
        .execute();

      return results.map((row) => ({
        id: row.id as string,
        name: (row.name as string) || "",
        room: (row.room as string) || "",
        branch: (row.branch as string) || "",
      }));
    } catch (error) {
      console.error("Error fetching labs:", error);
      throw error;
    }
  },

  async getLabById(id: string): Promise<Lab | null> {
    if (!id) {
      throw new Error("Lab ID is required");
    }

    try {
      const result = await db
        .table<Record<string, unknown>>("labs")
        .where("id", id)
        .where("deleted_at", null)
        .first();

      if (!result) {
        return null;
      }

      return {
        id: result.id as string,
        name: (result.name as string) || "",
        room: (result.room as string) || "",
        branch: (result.branch as string) || "",
      };
    } catch (error) {
      console.error("Error fetching lab by ID:", error);
      throw error;
    }
  },

  async getLabsByFaculty(faculty: string): Promise<Lab[]> {
    if (!faculty) {
      throw new Error("Faculty is required");
    }

    try {
      const results = await db
        .table<Record<string, unknown>>("labs")
        .where("faculty", faculty)
        .where("deleted_at", null)
        .orderBy("branch")
        .orderBy("room")
        .execute();

      return results.map((row) => ({
        id: row.id as string,
        name: (row.name as string) || "",
        room: (row.room as string) || "",
        branch: (row.branch as string) || "",
      }));
    } catch (error) {
      console.error("Error fetching labs by faculty:", error);
      throw error;
    }
  },
};
