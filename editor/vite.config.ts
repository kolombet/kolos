import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(async () => ({
  plugins: [svelte(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
        protocol: "ws",
        host,
        port: 1421,
      }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    chunkSizeWarningLimit: 800,
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (!id.includes("node_modules")) return;
          if (/[\\/]node_modules[\\/](svelte|@sveltejs)[\\/]/.test(id)) return "svelte";
          if (/[\\/]node_modules[\\/]mermaid[\\/]/.test(id)) return "mermaid";
          if (/[\\/]node_modules[\\/](katex|rehype-katex|remark-math)[\\/]/.test(id)) return "katex";
          if (/[\\/]node_modules[\\/](rehype-highlight|lowlight|highlight\.js)[\\/]/.test(id)) return "highlight";
          if (/[\\/]node_modules[\\/](marked|unified|remark|rehype)[\\/]/.test(id)) return "markdown";
        },
      },
    },
  },
}));
