import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./index.css";

// Ink is the default; follow the system when it asks for light.
document.documentElement.classList.toggle("dark", !matchMedia("(prefers-color-scheme: light)").matches);

if (import.meta.env.DEV && !("__TAURI_INTERNALS__" in window)) {
  (await import("./preview")).installPreview();
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
