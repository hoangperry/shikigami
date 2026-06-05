/// <reference types="node" />
import { defineConfig, type Plugin } from "vite";
import react from "@vitejs/plugin-react";

const host = process.env.TAURI_DEV_HOST;

// Live2D binary assets (.moc3, etc.) need an explicit Content-Type, otherwise
// WKWebView's XHR inside the Live2D plugin treats the response as an opaque
// "Network error" even though curl shows HTTP 200. Attach a small middleware
// that stamps application/octet-stream on every extension the plugin streams.
const liveTwoDMimeTypes: Plugin = {
  name: "shikigami-live2d-mime",
  configureServer(server) {
    server.middlewares.use((req, res, next) => {
      const url = req.url ?? "";
      if (url.endsWith(".moc3")) {
        res.setHeader("Content-Type", "application/octet-stream");
      }
      next();
    });
  },
};

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), liveTwoDMimeTypes],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    ...(host ? { hmr: { protocol: "ws" as const, host, port: 1421 } } : {}),
    watch: {
      ignored: ["**/src-tauri/**", "**/characters/**"],
    },
  },

  envPrefix: ["VITE_", "TAURI_ENV_*"],

  build: {
    target:
      process.env.TAURI_ENV_PLATFORM === "windows"
        ? "chrome105"
        : "safari13",
    minify: !process.env.TAURI_ENV_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
    rollupOptions: {
      output: {
        // Split rarely-changing vendor code into its own chunks. This is a
        // Tauri app — assets load from local disk, not the network, so the
        // win is cleaner builds + cache stability across releases rather
        // than transfer size. The Live2D renderer is already lazy-loaded
        // (dynamic import in character-stage), so `pixi-live2d-display-*`
        // stays in its own async chunk; we only pin the eager vendors here.
        manualChunks: {
          react: ["react", "react-dom"],
          pixi: ["pixi.js"],
        },
      },
    },
    // The largest eager chunk is the Pixi runtime (~400 KB). For a
    // local-disk desktop app this is not a transfer-cost concern, so we
    // lift the default 500 KB advisory to a realistic ceiling instead of
    // chasing a web-only metric.
    chunkSizeWarningLimit: 800,
  },
});
