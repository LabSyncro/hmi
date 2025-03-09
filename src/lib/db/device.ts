import { db } from './client'
import type { 
  Devices as DevicesType,
  DeviceKinds as DeviceKindsType,
  Labs as LabsType
} from '../../../src/types/db/generated'

export interface DeviceDetail {
  device: DevicesType
  kind: DeviceKindsType
  lab: LabsType | null
}

const MAX_RETRIES = 3;
const RETRY_DELAY = 500; // ms

async function sleep(ms: number) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

export async function getDeviceById(id: string): Promise<DeviceDetail | null> {
  let retries = 0;
  
  while (retries < MAX_RETRIES) {
    try {
      console.log(`Attempt ${retries + 1} to fetch device with ID:`, id);
      
      // Get device
      const devices = await db.query<DevicesType>({
        table: 'public.devices',
        conditions: [['id' as keyof DevicesType, id]]
      });
      
      console.log('Found devices:', devices);
      
      if (devices.length === 0) {
        console.log('No device found with ID:', id);
        return null;
      }
      
      const device = devices[0];
      
      // Get device kind
      const kinds = await db.query<DeviceKindsType>({
        table: 'public.device_kinds',
        conditions: [['id' as keyof DeviceKindsType, device.kind]]
      });
      
      console.log('Found device kinds:', kinds);
      
      if (kinds.length === 0) {
        throw new Error('Device kind not found');
      }
      
      // Get lab
      const labs = device.labId ? await db.query<LabsType>({
        table: 'public.labs',
        conditions: [['id' as keyof LabsType, device.labId]]
      }) : [];
      
      console.log('Found labs:', labs);
      
      return {
        device,
        kind: kinds[0],
        lab: labs[0] || null
      };
    } catch (error) {
      console.error(`Error fetching device details (attempt ${retries + 1}):`, error);
      retries++;
      
      if (retries < MAX_RETRIES) {
        console.log(`Retrying in ${RETRY_DELAY}ms...`);
        await sleep(RETRY_DELAY);
      } else {
        console.error('Max retries reached, giving up.');
        throw error;
      }
    }
  }
  
  return null; // This should never be reached due to the throw in the loop
} 