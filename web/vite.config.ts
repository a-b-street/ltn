import { resolve } from "path";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        cnt: resolve(__dirname, "cnt.html"),
        england: resolve(__dirname, "england.html"),
        user_guide: resolve(__dirname, "user_guide.html"),
        not_found: resolve(__dirname, "404.html"),
      },
    },
  },
  plugins: [
    svelte(),
    wasm(),
  ],
  server: {
    fs: {
      allow: ["./", "../backend/pkg"],
    },
  },
  // @ts-ignore - The works, but type checking fails. Not sure why.
  test: {
    environment: "jsdom",
    setupFiles: ["./vitest.setup.ts"],
  },
  optimizeDeps: {
    // Something breaks WASM import
    exclude: ["route-snapper"],
  },
});
