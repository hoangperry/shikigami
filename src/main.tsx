import { createRoot } from "react-dom/client";
import { App } from "./App";

// StrictMode intentionally disabled: double-mount races our async Pixi +
// Live2D init (mount → prebundle → plugin load → stage.addChild). The
// first effect's IIFE keeps going after React runs its cleanup (which
// disposes the Pixi app), so addChild lands on a torn-down renderer and
// nothing paints. Re-enable once the IIFE is fully abortable or we move
// to a Suspense-driven fetcher.
const container = document.getElementById("root");
if (!container) throw new Error("#root element missing");

createRoot(container).render(<App />);
