import { defineConfig } from "tsup";

export default defineConfig({
  target: "es2022",
  entry: ["src/index.ts"],
  format: ["cjs", "esm"],
  dts: true,
  outDir: "dist",
  clean: true,
  sourcemap: true,
  minify: process.env.NODE_ENV !== "development",
  external: [],
  esbuildOptions(options) {
    // ensure esbuild is configured for tree-shaking (esbuild is used internally by tsup)
    options.treeShaking = true;
  },
});
