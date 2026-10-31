import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type ViewItem = { itemType: string; subItems: ViewItem[]; minSupportVersion?: string | null };
export type Battery = { device: number; level: number; charging: boolean };
export type Wear = { device: number; in_box: boolean; in_ear: boolean; cover_closed: boolean };
export type Key = { device: number; button: number; action: number; function: number };
export type EqPreset = { selected: boolean; min: number; max: number; id: number; name: string; points: [number, number][] };

export type DeviceState = {
  connected: boolean;
  mock: boolean;
  name: string;
  address: string;
  model: string | null;
  view_items: ViewItem[];
  product: Record<string, unknown>;
  supported: number[];
  battery: Battery[];
  wear: Wear[];
  noise: { action: number; ty: number; value: number } | null;
  eq_id: number | null;
  eq: EqPreset[];
  keymap: Key[];
  features: Record<string, number>;
  high_volume: boolean | null;
  color: number | null;
};

export type Paired = { name: string; address: string; connected: boolean };

export const api = {
  scan: () => invoke<Paired[]>("scan"),
  models: () => invoke<string[]>("models"),
  connect: (name: string, address: string) => invoke<DeviceState>("connect", { name, address }),
  connectMock: (model: string) => invoke<DeviceState>("connect_mock", { model }),
  disconnect: () => invoke<void>("disconnect"),
  setNoise: (value: number) => invoke<void>("set_noise", { value }),
  setFeature: (id: number, on: boolean) => invoke<void>("set_feature", { id, on }),
  setEq: (id: number) => invoke<void>("set_eq", { id }),
  setHighVolume: (on: boolean) => invoke<void>("set_high_volume", { on }),
  setKeymap: (keys: Key[]) => invoke<void>("set_keymap", { keys }),
  find: (on: boolean) => invoke<void>("find", { on }),
  onState: (cb: (s: DeviceState) => void) => listen<DeviceState>("state", (e) => cb(e.payload)),
};

/** Flatten the model's UI tree into the set of feature tags it declares. */
export function featureSet(items: ViewItem[]): Set<string> {
  const out = new Set<string>();
  const walk = (n: ViewItem[]) => n.forEach((i) => (out.add(i.itemType), walk(i.subItems ?? [])));
  walk(items);
  return out;
}

/** Wire values for noise control (see PROTOCOL.md). */
export const NOISE = {
  off: { wire: 1, tag: "ITEM_NOISE_CLOSE", label: "Off" },
  transparency: { wire: 2, tag: "ITEM_NOISE_TRANSPARENT", label: "Transparency" },
  anc: { wire: 8, tag: "ITEM_NOISE_REDUCTION", label: "Noise cancelling" },
} as const;

export const ANC_LEVELS = [
  { wire: 4, tag: "ITEM_LIGHT_REDUCTION", label: "Light" },
  { wire: 16, tag: "ITEM_MEDIUM_REDUCTION", label: "Medium" },
  { wire: 8, tag: "ITEM_DEEP_REDUCTION", label: "Deep" },
  { wire: 32, tag: "ITEM_SMART_REDUCTION", label: "Smart" },
] as const;

export const KEY_FUNCTIONS: Record<number, string> = {
  0: "Off",
  1: "Play / pause",
  6: "Next track",
  5: "Previous track",
  4: "Voice assistant",
  8: "Noise control",
  13: "Switch device",
  17: "Game mode",
  11: "Volume up",
  12: "Volume down",
  31: "AI assist",
};

export const KEY_ACTIONS: Record<number, string> = { 1: "Single tap", 2: "Double tap", 3: "Triple tap", 4: "Long press" };

/** feature-switch id -> label + the model tag that gates it */
export const FEATURES: { id: number; tag: string; label: string; hint: string }[] = [
  { id: 17, tag: "ITEM_DOUBLE_DEVICE_CONNECT", label: "Dual-device connection", hint: "Stay connected to two devices" },
  { id: 24, tag: "ITEM_HIGH_QUALITY_SOUND", label: "High-quality audio", hint: "Prefer the higher-bitrate codec" },
  { id: 27, tag: "ITEM_3D_SOUND_EFFECT", label: "3D sound", hint: "Spatial audio effect" },
  { id: 29, tag: "ITEM_DYNAMIC_LOW_SOUND_SWITCH", label: "Dynamic bass", hint: "Adaptive low-frequency boost" },
  { id: 4, tag: "ITEM_GAME_MODE", label: "Game mode", hint: "Lower latency for gaming" },
];
