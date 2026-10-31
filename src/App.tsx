import { useEffect, useState } from "react";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { toast } from "sonner";
import { DeviceState, api } from "./api";
import { Connect } from "./Connect";
import { Dashboard } from "./Dashboard";

export default function App() {
  const [dev, setDev] = useState<DeviceState | null>(null);
  const [wasConnected, setWasConnected] = useState(false);

  useEffect(() => {
    const un = api.onState((s) => {
      setDev(s);
      if (s.connected) setWasConnected(true);
    });
    if (import.meta.env.DEV && location.hash === "#demo") api.connectMock("realme Buds Air 5 Pro").then(setDev);
    return () => void un.then((f) => f());
  }, []);

  useEffect(() => {
    if (wasConnected && dev && !dev.connected) toast.error("Lost connection to your earbuds", { description: "Put them back in your ears or take them out of the case, then connect again." });
  }, [wasConnected, dev]);

  return (
    <TooltipProvider>
      {dev?.connected ? (
        <Dashboard dev={dev} onLeave={() => (api.disconnect(), setDev(null), setWasConnected(false))} />
      ) : (
        <Connect onConnected={setDev} />
      )}
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}
