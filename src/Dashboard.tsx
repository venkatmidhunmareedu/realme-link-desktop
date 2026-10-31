import { ReactNode, useEffect } from "react";
import { AnimatePresence, motion, useMotionValue, useSpring, useTransform } from "framer-motion";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/lib/utils";
import { ANC_LEVELS, DeviceState, EqPreset, FEATURES, KEY_ACTIONS, KEY_FUNCTIONS, NOISE, api, featureSet } from "./api";
import { Buds3D } from "./Buds3D";

const slug = (s: string) => s.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
const NOISE_TONE: Record<number, { pill: string; text: string }> = {
  [NOISE.off.wire]: { pill: "bg-foreground/12", text: "text-foreground" },
  [NOISE.transparency.wire]: { pill: "bg-transparency", text: "text-background" },
  [NOISE.anc.wire]: { pill: "bg-primary", text: "text-primary-foreground" },
};
const EASE = [0.22, 1, 0.36, 1] as const;

export function Dashboard({ dev, onLeave }: { dev: DeviceState; onLeave: () => void }) {
  const has = featureSet(dev.view_items);
  const title = dev.model ?? dev.name;
  const canNoise = dev.supported.includes(0x404) && (has.has("GROUP_NOISE_CONTROL") || has.has("ITEM_NOISE_REDUCTION"));
  const toggles = FEATURES.filter((f) => has.has(f.tag) && String(f.id) in dev.features);
  const sections: ReactNode[] = [];
  if (canNoise) sections.push(<Noise key="n" dev={dev} has={has} />);
  if (dev.eq.length > 0) sections.push(<Sound key="s" eq={dev.eq} activeId={dev.eq_id} />);
  if (toggles.length || dev.high_volume !== null) sections.push(<Features key="f" dev={dev} rows={toggles} />);
  if (dev.keymap.length > 0 && dev.supported.includes(0x401)) sections.push(<Controls key="c" dev={dev} />);

  return (
    <div className="flex h-full flex-col overflow-y-auto lg:overflow-hidden">
      <header className="flex items-center justify-between px-6 py-4">
        <div className="flex items-center gap-2.5 text-sm font-semibold">
          <span className="size-2.5 rounded-full bg-signal" /> Realme Link
        </div>
        <Button variant="ghost" size="sm" onClick={onLeave}>
          Disconnect
        </Button>
      </header>

      <main className="grid min-h-0 flex-1 gap-6 px-6 pb-6 lg:grid-cols-[minmax(0,1.1fr)_minmax(0,1fr)]">
        <motion.section
          initial={{ opacity: 0, scale: 0.985 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.6, ease: EASE }}
          className="stage relative flex min-h-[38rem] flex-col sm:min-h-[30rem] overflow-hidden rounded-3xl border"
        >
          <div className="flex items-start justify-between gap-4 p-7">
            <div>
              <h1 className="font-display text-4xl leading-none font-semibold tracking-tight text-balance">{title}</h1>
              <p className="mt-2 flex items-center gap-2 text-sm text-muted-foreground">
                <span className="size-1.5 rounded-full bg-signal" />
                {dev.mock ? "Demo device" : "Connected"}
              </p>
            </div>
          </div>
          <div className="min-h-0 flex-1">
            <Buds3D slug={slug(title)} />
          </div>
          <Batteries dev={dev} />
        </motion.section>

        <ScrollArea className="lg:min-h-0 lg:pr-3">
          <div className="flex flex-col pb-4">
            {sections.map((s, i) => (
              <div key={i}>
                {i > 0 && <Separator className="my-7" />}
                {s}
              </div>
            ))}
            {sections.length === 0 && <p className="text-muted-foreground">This model doesn't expose any adjustable settings.</p>}
          </div>
        </ScrollArea>
      </main>
    </div>
  );
}

/* ---------------------------------------------------------------- battery */

type Cell = { n: number; label: string; pct: number | null; where: string };

function Batteries({ dev }: { dev: DeviceState }) {
  const items: Cell[] = [
    { n: 1, label: "Left bud" },
    { n: 2, label: "Right bud" },
    { n: 3, label: "Case" },
  ].flatMap((i): Cell[] => {
    const b = dev.battery.find((x) => x.device === i.n);
    const w = dev.wear.find((x) => x.device === i.n);
    if (!b) return [];
    if (i.n === 3) {
      // The case only reports while a bud is docked in it (its wear entry exists); otherwise it reads 0.
      if (!w) return [{ ...i, pct: null, where: "Put a bud in the case to see it" }];
      return [{ ...i, pct: b.level, where: w.cover_closed ? "Lid closed" : "Lid open" }];
    }
    return [{ ...i, pct: b.level, where: w?.in_ear ? "In ear" : w?.in_box ? "In the case" : "" }];
  });
  return (
    <div className="glass grid grid-flow-col auto-cols-fr divide-x border-t">
      {items.map(({ n, label, pct, where }) => (
        <Level key={n} label={label} where={where} pct={pct} />
      ))}
    </div>
  );
}

function Level({ label, where, pct }: { label: string; where: string; pct: number | null }) {
  const mv = useMotionValue(0);
  const spring = useSpring(mv, { stiffness: 90, damping: 20 });
  const text = useTransform(spring, (v) => Math.round(v));
  useEffect(() => mv.set(pct ?? 0), [pct, mv]);
  const low = pct !== null && pct <= 20;
  return (
    <div className="px-4 py-5 sm:px-6">
      <div className="flex items-baseline gap-1">
        {pct === null ? (
          <span className="font-display text-3xl font-medium tracking-tight text-muted-foreground sm:text-4xl" aria-label="Not available">&ndash;</span>
        ) : (
          <>
            <motion.span className={cn("font-display text-3xl font-medium tracking-tight sm:text-4xl", low && "text-low")}>{text}</motion.span>
            <span className="text-sm text-muted-foreground">%</span>
          </>
        )}
      </div>
      <div className="mt-2 h-1 overflow-hidden rounded-full bg-muted" role="presentation">
        <motion.div className={cn("h-full rounded-full", low ? "bg-low" : "bg-foreground")} animate={{ width: `${pct ?? 0}%` }} transition={{ duration: 0.8, ease: EASE }} />
      </div>
      <p className="mt-2 text-sm whitespace-nowrap">{label}</p>
      <p className="min-h-4 text-xs text-muted-foreground">{where}</p>
    </div>
  );
}

/* ---------------------------------------------------------------- sections */

function Section({ title, hint, children }: { title: string; hint?: string; children: ReactNode }) {
  return (
    <section>
      <h2 className="font-display text-xl font-semibold tracking-tight">{title}</h2>
      {hint && <p className="mt-1 text-sm text-muted-foreground">{hint}</p>}
      <div className="mt-4">{children}</div>
    </section>
  );
}

/** Shared segmented control: one pill slides between options (answers the click). */
function Segmented<T extends string | number>({
  id, label, value, options, onPick, size = "lg",
}: {
  id: string;
  label: string;
  value: T | undefined;
  options: { key: T; label: string; pill?: string; text?: string }[];
  onPick: (k: T) => void;
  size?: "lg" | "sm";
}) {
  return (
    <ToggleGroup
      aria-label={label}
      value={value === undefined ? [] : [String(value)]}
      onValueChange={(v) => {
        const k = options.find((o) => String(o.key) === v[0]);
        if (k) onPick(k.key);
      }}
      spacing={0}
      className="glass w-full gap-1 rounded-2xl border p-1"
    >
      {options.map((o) => {
        const active = o.key === value;
        return (
          <ToggleGroupItem
            key={String(o.key)}
            value={String(o.key)}
            className={cn("relative flex-1 rounded-xl bg-transparent! font-medium text-muted-foreground hover:bg-transparent hover:text-foreground", size === "lg" ? "h-11 px-1 text-xs whitespace-nowrap sm:px-2.5 sm:text-sm" : "h-9 text-[0.8rem]", active && "hover:text-inherit")}
          >
            {active && <motion.span layoutId={id} transition={{ type: "spring", stiffness: 300, damping: 32 }} className={cn("absolute inset-0 rounded-xl shadow-[0_8px_18px_-8px_color-mix(in_oklch,var(--foreground)_45%,transparent)]", o.pill ?? "bg-primary")} />}
            <span className={cn("relative", active && (o.text ?? "text-primary-foreground"))}>{o.label}</span>
          </ToggleGroupItem>
        );
      })}
    </ToggleGroup>
  );
}

function Noise({ dev, has }: { dev: DeviceState; has: Set<string> }) {
  const cur = dev.noise?.value;
  const modes = Object.values(NOISE).filter((m) => has.has(m.tag));
  const levels = ANC_LEVELS.filter((l) => has.has(l.tag));
  const ancOn = cur !== undefined && cur !== NOISE.off.wire && cur !== NOISE.transparency.wire;
  return (
    <Section title="Noise control" hint="Choose how much of the outside world you hear.">
      <Segmented
        id="noise-mode"
        label="Noise control mode"
        value={ancOn ? NOISE.anc.wire : cur}
        options={modes.map((m) => ({ key: m.wire, label: m.label, ...NOISE_TONE[m.wire] }))}
        onPick={(w) => api.setNoise(w === NOISE.anc.wire && levels.length && ancOn ? cur! : w)}
      />
      <AnimatePresence initial={false}>
        {ancOn && levels.length > 0 && (
          <motion.div key="lv" initial={{ height: 0, opacity: 0 }} animate={{ height: "auto", opacity: 1 }} exit={{ height: 0, opacity: 0 }} transition={{ duration: 0.45, ease: EASE }} className="overflow-hidden">
            <div className="pt-3">
              <Segmented id="noise-level" size="sm" label="Cancelling strength" value={cur} options={levels.map((l) => ({ key: l.wire, label: l.label }))} onPick={(w) => api.setNoise(w)} />
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </Section>
  );
}

function Sound({ eq, activeId }: { eq: EqPreset[]; activeId: number | null }) {
  const active = eq.find((e) => e.id === activeId) ?? eq.find((e) => e.selected) ?? eq[0];
  return (
    <Section title="Sound" hint="Pick an equaliser preset.">
      <Segmented id="eq" label="Equaliser preset" value={active.id} options={eq.map((e) => ({ key: e.id, label: e.name || `Preset ${e.id}` }))} onPick={(id) => api.setEq(id)} />
      <Curve preset={active} />
    </Section>
  );
}

function Curve({ preset }: { preset: EqPreset }) {
  const W = 360, H = 104, range = Math.max(1, Math.abs(preset.min), Math.abs(preset.max));
  const pts = preset.points.map(([f, db], i, a) => [(i / Math.max(1, a.length - 1)) * (W - 32) + 16, H / 2 - (db / range) * (H / 2 - 22), f, db] as const);
  const d = pts.map(([x, y], i) => `${i ? "L" : "M"}${x.toFixed(1)},${y.toFixed(1)}`).join(" ");
  const summary = pts.map(([, , f, db]) => `${f} hertz ${db > 0 ? "+" : ""}${db} decibels`).join(", ");
  return (
    <svg viewBox={`0 0 ${W} ${H}`} className="mt-5 w-full overflow-visible" role="img" aria-label={`Equaliser curve: ${summary}`}>
      <line x1="0" x2={W} y1={H / 2} y2={H / 2} className="stroke-border" strokeDasharray="3 5" />
      <motion.path initial={false} animate={{ d }} transition={{ duration: 0.45, ease: EASE }} fill="none" className="stroke-primary" strokeWidth="2.25" strokeLinejoin="round" strokeLinecap="round" />
      {pts.map(([x, y, f]) => (
        <g key={f}>
          <motion.circle initial={false} animate={{ cx: x, cy: y }} transition={{ duration: 0.45, ease: EASE }} r="4" className="fill-background stroke-primary" strokeWidth="2" />
          <text x={x} y={H + 4} textAnchor="middle" className="fill-muted-foreground text-[10.5px]">{f >= 1000 ? `${f / 1000}k` : f}</text>
        </g>
      ))}
    </svg>
  );
}

function Row({ label, hint, children }: { label: string; hint: string; children: ReactNode }) {
  return (
    <label className="flex items-center justify-between gap-6 py-3 first:pt-0 last:pb-0">
      <span className="grid">
        <span className="font-medium">{label}</span>
        <span className="text-sm text-muted-foreground">{hint}</span>
      </span>
      {children}
    </label>
  );
}

function Features({ dev, rows }: { dev: DeviceState; rows: typeof FEATURES }) {
  return (
    <Section title="Features">
      <div className="divide-y [&>label]:py-3.5 [&>label:first-child]:pt-0">
        {dev.high_volume !== null && (
          <Row label="Volume protection" hint="Limits loud output to protect your hearing.">
            <Switch checked={dev.high_volume} onCheckedChange={(v) => api.setHighVolume(v)} />
          </Row>
        )}
        {rows.map((f) => (
          <Row key={f.id} label={f.label} hint={f.hint}>
            <Switch checked={dev.features[String(f.id)] !== 0} onCheckedChange={(v) => api.setFeature(f.id, v)} />
          </Row>
        ))}
      </div>
    </Section>
  );
}

function Controls({ dev }: { dev: DeviceState }) {
  const update = (idx: number, fn: number) => api.setKeymap(dev.keymap.map((k, i) => (i === idx ? { ...k, function: fn } : k)));
  const items = Object.fromEntries(Object.entries(KEY_FUNCTIONS));
  return (
    <Section title="Touch controls" hint="Tap patterns are inferred from the app's code and can differ between models.">
      <div className="grid gap-6 sm:grid-cols-2">
        {([[1, "Left bud"], [2, "Right bud"]] as const).map(([d, label]) => {
          const rows = dev.keymap.map((k, i) => ({ k, i })).filter(({ k }) => k.device === d);
          if (!rows.length) return null;
          return (
            <div key={d} className="grid content-start gap-3">
              <h3 className="text-sm font-semibold">{label}</h3>
              {rows.map(({ k, i }) => (
                <div key={i} className="grid gap-1.5">
                  <span className="text-xs text-muted-foreground">{KEY_ACTIONS[k.action] ?? `Gesture ${k.action}`}</span>
                  <Select items={items} value={String(k.function)} onValueChange={(v) => v !== null && update(i, Number(v))}>
                    <SelectTrigger className="w-full" aria-label={`${label}, ${KEY_ACTIONS[k.action] ?? "gesture"}`}>
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      {!(k.function in KEY_FUNCTIONS) && <SelectItem value={String(k.function)}>Function {k.function}</SelectItem>}
                      {Object.entries(KEY_FUNCTIONS).map(([v, n]) => (
                        <SelectItem key={v} value={v}>{n}</SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              ))}
            </div>
          );
        })}
      </div>
    </Section>
  );
}
