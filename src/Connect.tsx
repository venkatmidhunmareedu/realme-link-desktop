import { useCallback, useEffect, useState } from "react";
import { motion } from "framer-motion";
import { RefreshCw } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { DeviceState, Paired, api } from "./api";

export function Connect({ onConnected }: { onConnected: (d: DeviceState) => void }) {
  const [paired, setPaired] = useState<Paired[] | null>(null);
  const [models, setModels] = useState<string[]>([]);
  const [busy, setBusy] = useState<string | null>(null);

  const scan = useCallback(() => {
    setPaired(null);
    api.scan().then(setPaired).catch((e) => (setPaired([]), toast.error("Couldn't read paired devices", { description: String(e) })));
  }, []);

  useEffect(() => {
    api.models().then(setModels);
    scan();
  }, [scan]);

  const go = async (key: string, name: string, fn: () => Promise<DeviceState>) => {
    setBusy(key);
    try {
      onConnected(await fn());
    } catch (e) {
      const msg = String(e).replace(/^Error: /, "");
      toast.error(`Couldn't connect to ${name}`, {
        description: /timed out/i.test(msg) ? `${msg} Take the earbuds out of the case and check they're connected in your system's Bluetooth settings.` : msg,
      });
    } finally {
      setBusy(null);
    }
  };

  const mine = (paired ?? []).filter((p) => models.some((m) => p.name.toLowerCase().startsWith(m.toLowerCase())));

  return (
    <main className="mx-auto flex min-h-full max-w-xl flex-col justify-center gap-10 px-6 py-16">
      <motion.header initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ duration: 0.5, ease: [0.22, 1, 0.36, 1] }}>
        <div className="mb-10 flex items-center gap-2.5 text-sm font-semibold">
          <span className="size-2.5 rounded-full bg-signal" /> Realme Link
        </div>
        <h1 className="font-display text-5xl leading-[1.02] font-semibold tracking-tight text-balance">Pick your earbuds</h1>
        <p className="mt-4 max-w-[46ch] text-muted-foreground">Pair them in your system's Bluetooth settings first. Supported models show up here.</p>
      </motion.header>

      <section aria-live="polite" className="flex flex-col gap-2">
        {paired === null && <p className="text-sm text-muted-foreground">Looking for paired earbuds…</p>}
        {paired !== null && mine.length === 0 && (
          <div className="flex items-center justify-between gap-4 rounded-xl border border-dashed p-5">
            <p className="text-sm text-muted-foreground">No supported earbuds are paired with this computer yet.</p>
            <Button variant="outline" size="sm" onClick={scan}>
              <RefreshCw data-icon="inline-start" /> Look again
            </Button>
          </div>
        )}
        {mine.map((p, i) => (
          <motion.div key={p.address} initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.1 + i * 0.05 }}>
            <button
              disabled={!!busy}
              onClick={() => go(p.address, p.name, () => api.connect(p.name, p.address))}
              className="group flex w-full items-center justify-between gap-4 rounded-xl border bg-card p-4 text-left transition-colors hover:border-signal/60 focus-visible:ring-3 focus-visible:ring-ring/50 disabled:opacity-60"
            >
              <span className="grid">
                <span className="font-semibold">{p.name}</span>
                <span className="text-sm text-muted-foreground">{p.connected ? "Connected to this computer" : "Paired, not connected"}</span>
              </span>
              <span className="rounded-full bg-primary px-3.5 py-1.5 text-sm font-semibold text-primary-foreground">{busy === p.address ? "Connecting…" : "Connect"}</span>
            </button>
          </motion.div>
        ))}
      </section>

      <details className="group text-sm text-muted-foreground">
        <summary className="w-fit cursor-pointer rounded-md py-1 hover:text-foreground focus-visible:ring-3 focus-visible:ring-ring/50">Try a demo device</summary>
        <p className="mt-2 mb-3">A simulated pair of earbuds, so you can explore the app without hardware.</p>
        <div className="flex max-h-56 flex-wrap gap-2 overflow-auto">
          {models.map((m) => (
            <Button key={m} variant="outline" size="sm" disabled={!!busy} onClick={() => go(m, m, () => api.connectMock(m))}>
              {m}
            </Button>
          ))}
        </div>
      </details>
    </main>
  );
}
