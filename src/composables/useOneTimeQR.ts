import { toast } from "@/components/ui/toast";
import { userService, type UserDetail } from "@/lib/db";

const TOTP_CONFIG = {
  digits: 6,
  timeStep: 60,
};

export function useOneTimeQR() {
  const stringToBytes = (str: string): Uint8Array => {
    return new TextEncoder().encode(str);
  };

  const generateHMAC = async (
    key: Uint8Array,
    message: Uint8Array
  ): Promise<ArrayBuffer> => {
    const cryptoKey = await crypto.subtle.importKey(
      "raw",
      key,
      { name: "HMAC", hash: "SHA-256" },
      false,
      ["sign"]
    );
    return crypto.subtle.sign("HMAC", cryptoKey, message);
  };

  const verifyToken = async (
    token: string,
    userId: string,
    timestamp: number,
    secret?: string
  ): Promise<boolean> => {
    try {
      const counter = Math.floor(timestamp / 1000 / TOTP_CONFIG.timeStep);

      const counterBytes = new Uint8Array(8);
      let tempCounter = counter;
      for (let i = counterBytes.length - 1; i >= 0; i--) {
        counterBytes[i] = tempCounter & 0xff;
        tempCounter >>= 8;
      }

      const userSecret =
        secret ||
        `LabSyncro-${userId}-${new Date(timestamp).toISOString().split("T")[0]}`;
      const hmac = await generateHMAC(stringToBytes(userSecret), counterBytes);
      const hmacArray = new Uint8Array(hmac);

      const offset = hmacArray[hmacArray.length - 1] & 0xf;
      let code =
        ((hmacArray[offset] & 0x7f) << 24) |
        (hmacArray[offset + 1] << 16) |
        (hmacArray[offset + 2] << 8) |
        hmacArray[offset + 3];

      code = code % Math.pow(10, TOTP_CONFIG.digits);
      const generatedToken = code.toString().padStart(TOTP_CONFIG.digits, "0");

      return token === generatedToken;
    } catch (error) {
      toast({
        title: `Lỗi khi xác thực mã QR: ${error}`,
        variant: "destructive",
      });
      return false;
    }
  };

  const verifyScannedQrCode = async (
    scannedQrData: string
  ): Promise<{ user: UserDetail } | null> => {
    try {
      const qrData = JSON.parse(scannedQrData);
      const { token, userId, timestamp, expiry } = qrData;

      if (Date.now() > expiry) {
        toast({ title: "Mã QR đã hết hạn", variant: "destructive" });
        return null;
      }

      const isValid = await verifyToken(token, userId, timestamp);
      if (!isValid) {
        toast({ title: "Mã QR không hợp lệ", variant: "destructive" });
        return null;
      }

      try {
        const result = await userService.checkOneTimeQrCode({
          token,
          userId,
          timestamp,
        });
        return result;
      } catch (error: any) {
        toast({
          title: `Lỗi khi xác thực mã QR: ${error}`,
          variant: "destructive",
        });
        return null;
      }
    } catch (error) {
      toast({
        title: `Lỗi khi xác thực mã QR: ${error}`,
        variant: "destructive",
      });
      return null;
    }
  };

  return {
    verifyScannedQrCode,
  };
}
