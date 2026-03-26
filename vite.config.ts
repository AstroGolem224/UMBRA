import { defineConfig, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import path from "path";
import fs from "fs";

const host = process.env.TAURI_DEV_HOST;
const isTauriDev = Boolean(process.env.TAURI_ENV_TARGET_TRIPLE);
const isVitest = process.env.VITEST === "true";

// When running Vite standalone (browser preview), shim Tauri IPC modules
const tauriMock = path.resolve(__dirname, "./src/lib/tauri-mock.ts");
const tauriAliases = isTauriDev || isVitest
  ? {}
  : {
      "@tauri-apps/api/core": tauriMock,
      "@tauri-apps/api/event": tauriMock,
      "@tauri-apps/api/window": tauriMock,
    };

// Browser-mode notes API — mirrors Rust notes.rs for dev preview
function notesApiPlugin(): Plugin {
  const VAULT = "D:/Obsidian/2nd-brain/2nd-brain";
  const SUBDIR = "UMBRA_Notes";
  const base = path.join(VAULT, SUBDIR);

  function parseFrontmatter(raw: string) {
    const match = raw.match(/^---\r?\n([\s\S]*?)\r?\n---/);
    if (!match) return { title: "", category: "misc", tags: [], body: raw };
    const fm: Record<string, unknown> = {};
    for (const line of match[1].split(/\r?\n/)) {
      const idx = line.indexOf(":");
      if (idx > 0) {
        const key = line.slice(0, idx).trim();
        let val = line.slice(idx + 1).trim();
        // Strip surrounding quotes
        if ((val.startsWith("'") && val.endsWith("'")) || (val.startsWith('"') && val.endsWith('"')))
          val = val.slice(1, -1);
        fm[key] = val;
      }
    }
    // Parse YAML-style tags: [] or tags: [a, b]
    if (typeof fm.tags === "string") {
      const tagStr = (fm.tags as string).trim();
      if (tagStr === "[]" || tagStr === "") {
        fm.tags = [];
      } else if (tagStr.startsWith("[") && tagStr.endsWith("]")) {
        fm.tags = tagStr.slice(1, -1).split(",").map((t: string) => t.trim().replace(/^['"]|['"]$/g, "")).filter(Boolean);
      } else {
        fm.tags = tagStr.split(",").map((t: string) => t.trim()).filter(Boolean);
      }
    } else {
      fm.tags = [];
    }
    return { ...fm, body: raw.slice(match[0].length).trim() };
  }

  return {
    name: "umbra-notes-api",
    configureServer(server) {
      server.middlewares.use("/notes-api/list", (_req, res) => {
        if (!fs.existsSync(base)) { res.writeHead(200); res.end("[]"); return; }
        const notes: unknown[] = [];
        for (const cat of fs.readdirSync(base)) {
          const catPath = path.join(base, cat);
          try { if (!fs.statSync(catPath).isDirectory()) continue; } catch { continue; }
          for (const file of fs.readdirSync(catPath)) {
            if (!file.endsWith(".md")) continue;
            const filePath = path.join(catPath, file);
            const raw = fs.readFileSync(filePath, "utf-8");
            const fm = parseFrontmatter(raw);
            const stat = fs.statSync(filePath);
            notes.push({
              id: file.replace(/\.md$/, ""),
              title: (fm.title as string) || file.replace(/\.md$/, ""),
              content: (fm.body as string) || "",
              category: (fm.category as string) || cat,
              tags: Array.isArray(fm.tags) ? fm.tags : [],
              filePath: filePath.replace(/\\/g, "/"),
              createdAt: (fm.created_at as string) || (fm.created as string) || stat.birthtime.toISOString(),
              updatedAt: (fm.updated_at as string) || (fm.updated as string) || stat.mtime.toISOString(),
            });
          }
        }
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify(notes));
      });
    },
  };
}

// Browser-mode skills API — mirrors Rust plugins.rs list_skills for dev preview
function skillsApiPlugin(): Plugin {
  const home = process.env.USERPROFILE || process.env.HOME || ".";
  const skillsDir = path.join(home, ".claude", "skills");

  function parseSkillMd(raw: string): { name: string; description: string } {
    const nameMatch = raw.match(/^#\s+(.+)/m);
    const descMatch = raw.match(/^(?!#)(.{10,})/m);
    return {
      name: nameMatch?.[1]?.trim() || "",
      description: descMatch?.[1]?.trim() || "",
    };
  }

  function inferCategory(skillPath: string): string {
    const rel = path.relative(skillsDir, skillPath);
    const parts = rel.split(path.sep);
    return parts.length > 1 ? parts[0] : "general";
  }

  function inferAgents(name: string, desc: string, content: string): string[] {
    const haystack = `${name} ${desc} ${content}`.toLowerCase();
    const agents: string[] = [];
    const forgeTerms = ["rust", "tauri", "web", "typescript", "code", "frontend", "vue", "api", "full-stack"];
    const prismTerms = ["godot", "game", "gdscript", "scene", "animation", "physics"];
    if (forgeTerms.some(t => haystack.includes(t))) agents.push("forge");
    if (prismTerms.some(t => haystack.includes(t))) agents.push("prism");
    if (agents.length === 0) agents.push("forge");
    return agents;
  }

  return {
    name: "umbra-skills-api",
    configureServer(server) {
      server.middlewares.use("/skills-api/list", (_req, res) => {
        if (!fs.existsSync(skillsDir)) { res.writeHead(200); res.end("[]"); return; }
        const skills: unknown[] = [];
        for (const entry of fs.readdirSync(skillsDir)) {
          const entryPath = path.join(skillsDir, entry);
          if (!fs.statSync(entryPath).isDirectory()) continue;
          const skillMdPath = path.join(entryPath, "SKILL.md");
          const versionPath = path.join(entryPath, "VERSION");
          const content = fs.existsSync(skillMdPath) ? fs.readFileSync(skillMdPath, "utf-8") : "";
          const version = fs.existsSync(versionPath) ? fs.readFileSync(versionPath, "utf-8").trim() : "";
          const parsed = parseSkillMd(content);
          const name = parsed.name || entry;
          skills.push({
            id: entry,
            name,
            version,
            description: parsed.description,
            category: inferCategory(entryPath),
            agents: inferAgents(name, parsed.description, content),
            content,
            folder: entry,
          });
        }
        skills.sort((a: any, b: any) => a.name.localeCompare(b.name));
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(JSON.stringify(skills));
      });
    },
  };
}

export default defineConfig({
  plugins: [vue(), ...(process.env.TAURI_ENV_TARGET_TRIPLE ? [] : [notesApiPlugin(), skillsApiPlugin()])],
  clearScreen: false,
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
      ...tauriAliases,
    },
  },
  test: {
    environment: "jsdom",
    globals: true,
    exclude: ["tests/e2e/**", "node_modules/**"],
  },
  server: {
    port: 1430,
    strictPort: true,
    host: host || false,
    allowedHosts: ["host.docker.internal"],
    hmr: host
      ? { protocol: "ws", host, port: 1431 }
      : undefined,
    watch: { ignored: ["**/src-tauri/**"] },
    proxy: isTauriDev ? undefined : {
      "/pm-api": {
        target: "http://100.115.61.30:8000",
        changeOrigin: true,
        rewrite: (p: string) => p.replace(/^\/pm-api/, "/api"),
      },
    },
  },
  preview: {
    host: host || "0.0.0.0",
    allowedHosts: ["host.docker.internal"],
  },
});
