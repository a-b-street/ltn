import { defineConfig } from "vite";
import { resolve } from "path";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import wasmPack from "vite-plugin-wasm-pack";

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
    // disable hot module reloading during test runs
    svelte({ hot: !process.env.VITEST }),
    wasmPack(["../backend"], ["route-snapper"])
  ],
  // @ts-ignore - The works, but type checking fails. Not sure why.
  test: {
    environment: 'jsdom',
    setupFiles: ['./vitest.setup.ts'],
  }
})
