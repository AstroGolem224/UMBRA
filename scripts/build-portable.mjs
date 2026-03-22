import { cp, mkdir, readFile, rm, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import { execFile } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);

const root = process.cwd();
const packageJsonPath = path.join(root, "package.json");
const releaseExePath = path.join(root, "src-tauri", "target", "release", "umbra.exe");
const outRoot = path.join(root, "dist-portable");

const pkg = JSON.parse(await readFile(packageJsonPath, "utf8"));
const version = pkg.version;
const folderName = `UMBRA_${version}_x64_portable`;
const portableDir = path.join(outRoot, folderName);
const portableExePath = path.join(portableDir, "UMBRA.exe");
const readmePath = path.join(portableDir, "README.txt");
const zipPath = path.join(outRoot, `${folderName}.zip`);

await stat(releaseExePath);
await rm(portableDir, { recursive: true, force: true });
await rm(zipPath, { force: true });
await mkdir(portableDir, { recursive: true });

await cp(releaseExePath, portableExePath);

const readme = [
  "UMBRA portable",
  "",
  `version: ${version}`,
  "",
  "start:",
  "1. run UMBRA.exe",
  "2. open Settings on first launch",
  "3. set api url, dashboard url and repo root",
  "",
  "notes:",
  "- this is a portable no-installer build",
  "- on windows 11, webview2 is usually already present",
  "- github all-repos needs a github PAT in settings",
  "- updater checks only work when endpoint + public key are configured",
  "",
].join("\r\n");

await writeFile(readmePath, readme, "utf8");

const archiveCommand = `Compress-Archive -LiteralPath '${portableDir.replace(/'/g, "''")}' -DestinationPath '${zipPath.replace(/'/g, "''")}' -Force`;

await execFileAsync(
  "powershell.exe",
  [
    "-NoProfile",
    "-Command",
    archiveCommand,
  ],
  { cwd: root }
);

console.log(`portable dir: ${portableDir}`);
console.log(`portable zip: ${zipPath}`);
