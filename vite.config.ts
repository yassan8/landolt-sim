import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  root: "web",
  plugins: [react()],
  server: {
    host: "localhost",
    port: 4173,
  },
  preview: {
    host: "localhost",
    port: 4174,
  },
  build: {
    outDir: "../dist",
    emptyOutDir: true,
  },
});
