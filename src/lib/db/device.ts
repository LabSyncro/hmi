import { db } from './client'
import type {
  Devices as DevicesType,
  DeviceKinds as DeviceKindsType,
  Labs as LabsType
} from '@/types/db/generated'

export interface DeviceDetail {
  device: DevicesType
  kind: DeviceKindsType
  lab: LabsType | null
}

const MAX_RETRIES = 3;
const RETRY_DELAY = 500;

async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export async function getDeviceById(id: string): Promise<DeviceDetail | null> {
  let retries = 0;

  while (retries < MAX_RETRIES) {
    try {
      const devices = await db.query<DevicesType>({
        table: 'public.devices',
        conditions: [['id' as keyof DevicesType, id]]
      });


      if (devices.length === 0) {
        return null;
      }

      const device = devices[0];

      const kinds = await db.query<DeviceKindsType>({
        table: 'public.device_kinds',
        conditions: [['id' as keyof DeviceKindsType, device.kind]]
      });

      if (kinds.length === 0) {
        throw new Error('Device kind not found');
      }

      const labs = device.labId ? await db.query<LabsType>({
        table: 'public.labs',
        conditions: [['id' as keyof LabsType, device.labId]]
      }) : [];

      return {
        device,
        kind: kinds[0],
        lab: labs[0] || null
      };
    } catch (error) {
      retries++;

      if (retries < MAX_RETRIES) {
        await sleep(RETRY_DELAY);
      } else {
        throw error;
      }
    }
  }

  return null;
} 