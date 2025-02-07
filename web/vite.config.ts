import { defineConfig } from "vite";
import { resolve } from "path";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import wasmPack from "vite-plugin-wasm-pack";

export default defineConfig({
  base: "/ltn/",
  build: {
    rollupOptions: {
      input: {
        main: resolve(__dirname, "index.html"),
        cnt: resolve(__dirname, "cnt.html"),
      },
    },
  },
  plugins: [svelte(), wasmPack(["../backend"], ["route-snapper"])]
})
