import { db } from "./client";

export type HmiDetail = {
  hmiCode: string;
  expiresIn: number;
  expiresAt: string;
};

const HMI_CODE_TTL = 10 * 60;

const generateRandomCode = () => {
  return Math.floor(100000 + Math.random() * 900000).toString();
};

export const hmiService = {
  async generateHMICode(): Promise<HmiDetail | null> {
    try {
      let hmiCodeStr = generateRandomCode();
      let isUnique = false;
      const tableName = "hmi_codes";

      while (!isUnique) {
        const hmiCodeNum = parseInt(hmiCodeStr, 10);
        if (isNaN(hmiCodeNum)) {
          throw new Error("Invalid HMI code generated (NaN).");
        }

        const existingSql = `
          SELECT EXISTS (
            SELECT 1 FROM ${tableName} WHERE code = $1
          ) as exists
        `;

        const existingCode = await db.queryRaw<{ exists: boolean }>({
          sql: existingSql,
          params: [hmiCodeNum],
        });

        if (!existingCode[0].exists) {
          isUnique = true;
        } else {
          hmiCodeStr = generateRandomCode();
        }
      }

      const now = new Date();
      const expiresAtDate = new Date(now.getTime() + HMI_CODE_TTL * 1000);
      const finalHmiCodeNum = parseInt(hmiCodeStr, 10);

      await db.table(tableName).insert({
        code: finalHmiCodeNum,
        status: "pending",
        expires_at: expiresAtDate.getTime(),
      });

      const response: HmiDetail = {
        hmiCode: hmiCodeStr,
        expiresIn: HMI_CODE_TTL,
        expiresAt: expiresAtDate.toISOString(),
      };

      return response;
    } catch (error: any) {
      console.log(error);
      const userMessage = error.message?.includes("relation")
        ? "Lỗi cấu hình bảng mã HMI."
        : error.message?.includes("convert") ||
            error.message?.includes("operator does not exist")
          ? "Lỗi định dạng dữ liệu HMI."
          : "Lỗi tạo mã HMI không xác định.";
      throw new Error(userMessage);
    }
  },
};
