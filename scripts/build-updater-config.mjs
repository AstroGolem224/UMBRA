import { readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const basePath = path.join(root, "src-tauri", "tauri.conf.json");
const outPath = path.join(root, "src-tauri", "tauri.updater.generated.json");

const endpointEnv = process.env.UMBRA_UPDATER_ENDPOINT ?? "";
const publicKey = (process.env.UMBRA_UPDATER_PUBLIC_KEY ?? "").trim();

const endpoints = endpointEnv
  .split(/[\n,;]+/)
  .map((entry) => entry.trim())
  .filter(Boolean);

if (endpoints.length === 0) {
  throw new Error("UMBRA_UPDATER_ENDPOINT is required for updater release config generation.");
}

if (!publicKey) {
  throw new Error("UMBRA_UPDATER_PUBLIC_KEY is required for updater release config generation.");
}

const baseConfig = JSON.parse(await readFile(basePath, "utf8"));
const nextConfig = {
  ...baseConfig,
  bundle: {
    ...(baseConfig.bundle ?? {}),
    createUpdaterArtifacts: true,
  },
  plugins: {
    ...(baseConfig.plugins ?? {}),
    updater: {
      pubkey: publicKey,
      endpoints,
    },
  },
};

await writeFile(outPath, `${JSON.stringify(nextConfig, null, 2)}\n`, "utf8");
console.log(`generated ${path.relative(root, outPath)}`);
