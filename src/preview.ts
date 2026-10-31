// Dev-only: lets `pnpm dev` render in a plain browser by faking the Tauri backend.
import { emit } from "@tauri-apps/api/event";
import { mockIPC } from "@tauri-apps/api/mocks";

const view = (tags: string[]) => tags.map((t) => ({ itemType: t, subItems: [] }));
let st: any = null;
const push = () => void emit("state", structuredClone(st));

export function installPreview() {
  mockIPC(async (cmd, a: any) => {
    switch (cmd) {
      case "models":
        return ["realme Buds Air 5 Pro", "realme Buds T310", "realme Buds Air6", "realme Buds T110"];
      case "scan":
        return [{ name: "realme Buds Air 5 Pro", address: "AA:BB:CC:DD:EE:FF", connected: true }];
      case "connect":
      case "connect_mock":
        st = {
          connected: true, mock: true, name: a.model ?? a.name, address: "00:00:00:00:00:00", model: a.model ?? a.name,
          view_items: view(["GROUP_NOISE_CONTROL", "ITEM_NOISE_REDUCTION", "ITEM_NOISE_CLOSE", "ITEM_NOISE_TRANSPARENT", "ITEM_LIGHT_REDUCTION", "ITEM_MEDIUM_REDUCTION", "ITEM_DEEP_REDUCTION", "ITEM_DOUBLE_DEVICE_CONNECT", "ITEM_HIGH_QUALITY_SOUND", "ITEM_3D_SOUND_EFFECT", "ITEM_GAME_MODE"]),
          product: {}, supported: [0x404, 0x401],
          battery: [{ device: 1, level: 82, charging: false }, { device: 2, level: 74, charging: true }, { device: 3, level: 100, charging: false }],
          wear: [{ device: 1, in_box: false, in_ear: true, cover_closed: false }, { device: 2, in_box: true, in_ear: false, cover_closed: false }, { device: 3, in_box: true, in_ear: false, cover_closed: false }],
          noise: { action: 1, ty: 1, value: 8 }, eq_id: 1,
          eq: [
            { selected: true, min: -10, max: 10, id: 1, name: "Balanced", points: [[100, 0], [170, 0], [400, 0], [4300, 0], [9500, 0], [12000, 0]] },
            { selected: false, min: -10, max: 10, id: 2, name: "Bass Boost", points: [[100, 6], [170, 4], [400, 1], [4300, 0], [9500, 0], [12000, 0]] },
            { selected: false, min: -10, max: 10, id: 3, name: "Clear Vocals", points: [[100, -2], [170, 0], [400, 3], [4300, 4], [9500, 2], [12000, 0]] },
          ],
          keymap: [{ device: 1, button: 1, action: 2, function: 1 }, { device: 1, button: 1, action: 3, function: 6 }, { device: 1, button: 1, action: 4, function: 8 }, { device: 2, button: 1, action: 2, function: 1 }, { device: 2, button: 1, action: 3, function: 6 }, { device: 2, button: 1, action: 4, function: 8 }],
          features: { "17": 1, "24": 0, "27": 1, "4": 0 }, high_volume: false, color: 2,
        };
        return st;
      case "set_noise": st.noise.value = a.value; break;
      case "set_eq": st.eq_id = a.id; break;
      case "set_feature": st.features[String(a.id)] = a.on ? 1 : 0; break;
      case "set_high_volume": st.high_volume = a.on; break;
      case "set_keymap": st.keymap = a.keys; break;
      case "disconnect": st = null; return;
    }
    if (st) push();
  }, { shouldMockEvents: true });
}
