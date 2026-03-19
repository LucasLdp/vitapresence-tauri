import React, { useEffect } from "react";
import ReactDOM from "react-dom/client";
import { getCurrentWindow } from "@tauri-apps/api/window";
import "./index.css";
import App from "./App";

function Root() {
  useEffect(() => {
    // Show window after content loads
    const showWindow = async () => {
      try {
        const w = getCurrentWindow();
        await w.show();
      } catch (e) {
        console.error("Failed to show window:", e);
      }
    };

    // Small delay to ensure DOM is ready
    const timer = setTimeout(showWindow, 100);
    return () => clearTimeout(timer);
  }, []);

  return <App />;
}

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <Root />
  </React.StrictMode>,
);
