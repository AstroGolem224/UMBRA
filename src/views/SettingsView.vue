<template>
  <div class="settings-layout">
    <!-- ── Left navigation ── -->
    <nav class="settings-nav">
      <div class="nav-brand">
        <span class="nav-kicker">configuration</span>
        <h1 class="nav-title">Settings</h1>
      </div>

      <div class="nav-items">
        <button
          v-for="sec in sections"
          :key="sec.id"
          class="nav-item"
          :class="{ active: activeSection === sec.id }"
          type="button"
          @click="activeSection = sec.id"
        >
          <span class="nav-icon">{{ sec.icon }}</span>
          <span class="nav-label">{{ sec.label }}</span>
        </button>
      </div>

      <div class="nav-footer">
        <NeonButton type="button" variant="primary" :loading="saving" @click="save">
          Save settings
        </NeonButton>
        <Transition name="fade">
          <span v-if="saved" class="saved-badge">✓ Saved</span>
        </Transition>
      </div>
    </nav>

    <!-- ── Content panel ── -->
    <main class="settings-content">

      <!-- ══ Appearance ══ -->
      <section v-show="activeSection === 'appearance'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Appearance</h2>
          <p class="section-desc">Choose how UMBRA looks and behaves when you minimize or close it.</p>
        </div>

        <GlassCard>
          <div class="group-title">Color theme</div>
          <p class="group-desc">Pick a color scheme for the entire app. Changes apply instantly — no need to save.</p>
          <div class="theme-swatches">
            <button
              v-for="t in themes"
              :key="t.value"
              class="theme-swatch"
              :class="{ active: draft.theme === t.value }"
              type="button"
              @click="applyTheme(t.value)"
            >
              <span class="swatch-dot" :style="{ background: t.color }" />
              {{ t.label }}
            </button>
          </div>
        </GlassCard>

        <GlassCard>
          <div class="group-title">Window behavior</div>

          <div class="setting-row">
            <div class="setting-copy">
              <span class="setting-label">Keep running when closed</span>
              <span class="setting-hint">Closing the window hides UMBRA to the system tray instead of quitting. Right-click the tray icon to show, hide, or quit.</span>
            </div>
            <label class="toggle">
              <input v-model="draft.closeToTray" type="checkbox" />
              <span class="toggle-track" />
            </label>
          </div>

          <div class="setting-divider" />

          <div class="setting-row">
            <div class="setting-copy">
              <span class="setting-label">Start automatically at login</span>
              <span class="setting-hint">Launches UMBRA in the background when you sign in to Windows. Works best with "Keep running when closed" turned on.</span>
            </div>
            <label class="toggle">
              <input v-model="autostart" type="checkbox" @change="toggleAutostart" />
              <span class="toggle-track" />
            </label>
          </div>
        </GlassCard>
      </section>

      <!-- ══ Notes & Vault ══ -->
      <section v-show="activeSection === 'notes'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Notes & Vault</h2>
          <p class="section-desc">Connect UMBRA to your Obsidian vault so notes load and sync correctly.</p>
        </div>

        <GlassCard>
          <div class="field">
            <label class="setting-label">Vault folder</label>
            <span class="setting-hint">The root folder of your Obsidian vault. UMBRA reads and writes notes inside this folder.</span>
            <div class="path-row">
              <input v-model="draft.vaultPath" class="field-input glass-input" type="text" placeholder="e.g. C:/Users/you/Documents/MyVault" />
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="pickVaultPath">Browse</NeonButton>
            </div>
          </div>

          <div class="field">
            <label class="setting-label">Notes subfolder</label>
            <span class="setting-hint">The folder inside your vault where UMBRA stores its notes. Keep the default unless you want to move them.</span>
            <input v-model="draft.notesSubdir" class="field-input glass-input" type="text" placeholder="UMBRA_Notes" />
          </div>
        </GlassCard>
      </section>

      <!-- ══ Project Management ══ -->
      <section v-show="activeSection === 'pm'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Project Management</h2>
          <p class="section-desc">Connect UMBRA to your PM tool to keep tasks and priorities in sync.</p>
        </div>

        <GlassCard>
          <div class="field">
            <label class="setting-label">API URL</label>
            <span class="setting-hint">The base URL of your PM tool's API endpoint. Leave blank to disable the PM integration.</span>
            <input v-model="draft.pmToolUrl" class="field-input glass-input" type="text" placeholder="http://100.115.61.30:8000/api" />
          </div>

          <div class="field">
            <label class="setting-label">Dashboard URL</label>
            <span class="setting-hint">Where to open the PM tool's web dashboard. Leave blank and UMBRA will derive it from the API URL.</span>
            <input v-model="draft.pmToolDashboardUrl" class="field-input glass-input" type="text" placeholder="http://100.115.61.30:4173" />
          </div>

          <div class="field">
            <label class="setting-label">Sync interval</label>
            <span class="setting-hint">How often UMBRA polls the PM tool for changes. Minimum 5 seconds, maximum 300.</span>
            <div class="number-row">
              <input v-model.number="draft.pmToolPollSeconds" class="field-input glass-input number-input" type="number" min="5" max="300" />
              <span class="unit-label">seconds</span>
            </div>
          </div>

          <div class="doc-links">
            <NeonButton size="sm" variant="secondary" ghost type="button" :disabled="!pmDocsUrl" @click="openExternal(pmDocsUrl)">Open API docs ↗</NeonButton>
            <NeonButton size="sm" variant="secondary" ghost type="button" :disabled="!pmDashboardUrl" @click="openExternal(pmDashboardUrl)">Open dashboard ↗</NeonButton>
          </div>
        </GlassCard>
      </section>

      <!-- ══ Agents & Workspaces ══ -->
      <section v-show="activeSection === 'agents'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Agents & Workspaces</h2>
          <p class="section-desc">Set up where agents work, which AI providers power them, and how they authenticate.</p>
        </div>

        <!-- Workspaces -->
        <GlassCard>
          <div class="group-title">Workspaces</div>
          <p class="group-desc">A workspace is a folder on your machine where an agent can read and write files. You need at least one workspace before dispatching work.</p>

          <div v-for="(workspace, index) in draft.workspacePresets" :key="workspace.id" class="workspace-card">
            <div class="workspace-row__head">
              <input v-model="workspace.name" class="glass-input workspace-name" placeholder="Workspace name (e.g. UMBRA)" />
              <label class="inline-check">
                <input v-model="workspace.writable" type="checkbox" />
                <span>Writable</span>
              </label>
              <label class="inline-check">
                <input
                  :checked="draft.defaultWorkspaceId === workspace.id"
                  type="radio"
                  name="default-workspace"
                  @change="draft.defaultWorkspaceId = workspace.id"
                />
                <span>Default</span>
              </label>
              <NeonButton variant="danger" size="sm" ghost type="button" @click="draft.workspacePresets.splice(index, 1)">Remove</NeonButton>
            </div>
            <div class="grant-row">
              <input v-model="workspace.rootPath" class="glass-input" placeholder="Absolute folder path (e.g. C:/Projects/UMBRA)" />
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="pickWorkspaceRoot(index)">Browse</NeonButton>
            </div>
            <div class="workspace-grid">
              <label class="field">
                <span class="field-label">Allowed providers</span>
                <span class="setting-hint">Which AI providers can use this workspace. Comma-separated (e.g. codex, claude).</span>
                <input
                  :value="workspaceProvidersValue(workspace)"
                  class="glass-input"
                  placeholder="codex, claude"
                  @input="setWorkspaceProviders(workspace, ($event.target as HTMLInputElement).value)"
                />
              </label>
              <label class="field">
                <span class="field-label">Allowed agents</span>
                <span class="setting-hint">Which agent IDs can use this workspace. Leave blank to allow all agents.</span>
                <input
                  :value="workspaceAgentsValue(workspace)"
                  class="glass-input"
                  placeholder="forge, prism (or leave blank for all)"
                  @input="setWorkspaceAgents(workspace, ($event.target as HTMLInputElement).value)"
                />
              </label>
            </div>
          </div>

          <div class="workspace-actions">
            <NeonButton size="sm" variant="secondary" type="button" @click="addWorkspacePreset">+ Add workspace</NeonButton>
            <NeonButton size="sm" variant="secondary" ghost type="button" @click="seedWorkspaceGrantRoots">Auto-fill allowed paths</NeonButton>
          </div>

          <div class="field" style="margin-top: 20px">
            <label class="setting-label">Allowed root paths</label>
            <span class="setting-hint">Safety allowlist — agents can only access folders inside these paths. Any dispatch outside this list is blocked, even if a workspace exists. Use "Auto-fill" to populate from your workspaces above.</span>
            <div v-for="(root, index) in draft.workspaceGrantRoots" :key="`${root}-${index}`" class="grant-row" style="margin-top: 8px">
              <input v-model="draft.workspaceGrantRoots[index]" class="glass-input" placeholder="e.g. C:/Projects" />
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="pickGrantRoot(index)">Browse</NeonButton>
              <NeonButton variant="danger" size="sm" ghost type="button" @click="draft.workspaceGrantRoots.splice(index, 1)">Remove</NeonButton>
            </div>
            <NeonButton size="sm" variant="secondary" ghost type="button" style="margin-top: 8px" @click="addWorkspaceGrantRoot">+ Add path</NeonButton>
          </div>
        </GlassCard>

        <!-- AI Providers -->
        <GlassCard>
          <div class="group-title">AI providers</div>
          <p class="group-desc">UMBRA launches a CLI tool for each provider. Set the exact command (or full path to the executable). Use "Check setup" to verify the provider is ready.</p>

          <div class="provider-bootstrap-grid">
            <label class="field">
              <span class="field-label">Target workspace</span>
              <select v-model="providerBootstrapWorkspaceId" class="glass-input">
                <option value="">Select workspace</option>
                <option v-for="ws in workspaceOptions" :key="ws.id" :value="ws.id">{{ ws.name }}</option>
              </select>
            </label>
            <label class="field">
              <span class="field-label">Target agent</span>
              <select v-model="providerBootstrapAgentId" class="glass-input">
                <option value="">Select agent</option>
                <option v-for="agent in bootstrapAgents" :key="agent.id" :value="agent.id">{{ agent.name }} · {{ agent.id }}</option>
              </select>
            </label>
          </div>

          <label class="checkbox-row compact" style="margin-bottom: 16px">
            <input v-model="providerBootstrapOverwrite" type="checkbox" />
            <span>Overwrite existing instruction files when writing bootstrap</span>
          </label>

          <div v-for="provider in draft.providerCommands" :key="provider.providerId" class="provider-block">
            <div class="provider-block__header">
              <div class="provider-copy">
                <span class="provider-name">{{ providerLabel(provider.providerId) }}</span>
                <span class="provider-hint">id: {{ provider.providerId }}</span>
                <span class="provider-template">template: {{ providerTemplate(provider.providerId) }}</span>
              </div>
              <input v-model="provider.command" class="glass-input provider-command" :placeholder="provider.providerId" />
            </div>

            <div class="provider-actions">
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="checkProviderAuth(provider.providerId)">Check auth</NeonButton>
              <NeonButton size="sm" variant="secondary" ghost type="button" :loading="Boolean(providerActionBusy[provider.providerId])" @click="refreshProviderChecklist(provider.providerId)">Check setup</NeonButton>
              <NeonButton size="sm" variant="secondary" ghost type="button" :loading="Boolean(providerActionBusy[provider.providerId])" @click="smokeTestProvider(provider.providerId)">Smoke test</NeonButton>
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="writeProviderBootstrap(provider.providerId)">Write instructions</NeonButton>
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="copyProviderEnv(provider.providerId)">Copy env</NeonButton>
              <NeonButton size="sm" variant="secondary" ghost type="button" :loading="Boolean(probingProviders[provider.providerId])" @click="probeProvider(provider.providerId)">Probe CLI</NeonButton>
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="openExternal(providerDocs(provider.providerId))">Docs ↗</NeonButton>
            </div>

            <span v-if="providerProbeResults[provider.providerId]" class="provider-result">
              {{ providerProbeResults[provider.providerId]?.summary }}
            </span>
            <span v-if="providerActionResults[provider.providerId]" class="provider-result">
              {{ providerActionResults[provider.providerId] }}
            </span>

            <div v-if="providerChecklists[provider.providerId]" class="provider-checklist">
              <div
                v-for="item in providerChecklists[provider.providerId]?.items ?? []"
                :key="`${provider.providerId}-${item.key}`"
                class="checklist-item"
                :class="{ ready: item.ready, missing: !item.ready }"
              >
                <span class="checklist-status">{{ item.ready ? "✓" : "○" }}</span>
                <strong>{{ item.label }}</strong>
                <span class="provider-result">{{ item.detail }}</span>
              </div>
            </div>
          </div>

          <div class="doc-links" style="margin-top: 12px">
            <NeonButton size="sm" variant="secondary" ghost type="button" @click="openExternal(agentSetupGuideUrl)">Agent setup guide ↗</NeonButton>
          </div>
        </GlassCard>

        <!-- Agent tokens -->
        <GlassCard>
          <div class="group-title">Agent auth tokens</div>
          <p class="group-desc">Each agent authenticates with its own secret token. Tokens are created automatically. Use "Rotate" only if a token is compromised — the agent must update its config afterwards.</p>

          <div class="token-table">
            <div class="token-row token-row--head">
              <span>Agent</span>
              <span>Token</span>
              <span>Actions</span>
            </div>
            <div v-for="(token, agentId) in agentTokenEntries" :key="agentId" class="token-row">
              <span class="token-agent">{{ agentId }}</span>
              <div class="token-value-cell">
                <input
                  :value="tokenVisible[String(agentId)] ? token : '••••••••••••••••'"
                  class="glass-input token-input"
                  type="text"
                  readonly
                />
                <button type="button" class="token-action" @click="toggleTokenVisibility(String(agentId))">
                  {{ tokenVisible[String(agentId)] ? "Hide" : "Show" }}
                </button>
                <button type="button" class="token-action" @click="copyToken(String(token))">Copy</button>
              </div>
              <div class="token-actions-cell">
                <button type="button" class="token-action token-action--danger" @click="regenerateToken(String(agentId))">Rotate</button>
              </div>
            </div>
          </div>
          <p v-if="tokenCopied" class="field-hint success-text">Token copied to clipboard.</p>
        </GlassCard>

        <!-- Personas -->
        <GlassCard>
          <div class="group-title">Personas</div>
          <p class="group-desc">Personas are prompt presets that change how an agent approaches a task. Select one in the Workbench composer before dispatching work.</p>

          <div v-for="(persona, index) in draft.personaPresets" :key="persona.id" class="persona-row">
            <div class="persona-row__head">
              <input v-model="persona.name" class="glass-input persona-name" placeholder="Persona name (e.g. Reviewer)" />
              <NeonButton variant="danger" size="sm" ghost type="button" @click="draft.personaPresets.splice(index, 1)">Remove</NeonButton>
            </div>
            <input v-model="persona.description" class="glass-input" placeholder="Short description shown in the dropdown" />
            <textarea
              v-model="persona.prompt"
              class="glass-input persona-prompt"
              rows="4"
              placeholder="System prompt fragment — prepended to the agent's instructions for this run"
            />
          </div>
          <NeonButton size="sm" variant="secondary" type="button" @click="addPersonaPreset">+ Add persona</NeonButton>
        </GlassCard>
      </section>

      <!-- ══ GitHub ══ -->
      <section v-show="activeSection === 'github'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">GitHub</h2>
          <p class="section-desc">Connect UMBRA to GitHub to open pull requests and browse repos from the Launcher.</p>
        </div>

        <GlassCard>
          <div class="field">
            <label class="setting-label">Local repos folder</label>
            <span class="setting-hint">The folder on this machine where your Git repositories live. Used by local Git actions and the Launcher.</span>
            <div class="path-row">
              <input v-model="draft.repoRootPath" class="field-input glass-input" type="text" placeholder="e.g. C:/Repos" />
              <NeonButton size="sm" variant="secondary" ghost type="button" @click="pickRepoRootPath">Browse</NeonButton>
            </div>
          </div>

          <div class="field">
            <label class="setting-label">Personal access token</label>
            <span class="setting-hint">A GitHub token with <code>public_repo</code> scope (or <code>repo</code> for private repos). Required to open PRs and list all repositories in the Launcher.</span>
            <input v-model="draft.githubPat" class="field-input glass-input" type="password" placeholder="ghp_..." autocomplete="off" />
          </div>
        </GlassCard>
      </section>

      <!-- ══ Launcher ══ -->
      <section v-show="activeSection === 'launcher'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Quick launch</h2>
          <p class="section-desc">Apps and tools that appear in the Launcher for one-click access.</p>
        </div>

        <GlassCard>
          <p v-if="!draft.launchTargets?.length" class="field-hint" style="margin-bottom: 12px">No apps added yet. Click "+ Add app" to create your first shortcut.</p>

          <div v-for="(target, i) in draft.launchTargets" :key="i" class="launch-row">
            <input v-model="target.name" class="glass-input launch-name" placeholder="Name (e.g. VS Code)" />
            <input v-model="target.path" class="glass-input launch-path" placeholder="Executable path (e.g. C:/Program Files/VS Code/Code.exe)" />
            <NeonButton variant="danger" size="sm" ghost type="button" @click="draft.launchTargets!.splice(i, 1)">Remove</NeonButton>
          </div>

          <NeonButton size="sm" variant="secondary" type="button" @click="addLaunchTarget">+ Add app</NeonButton>
        </GlassCard>
      </section>

      <!-- ══ Taskboard ══ -->
      <section v-show="activeSection === 'taskboard'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Taskboard</h2>
          <p class="section-desc">Choose how each column behaves when you open the Tasks view.</p>
        </div>

        <GlassCard>
          <div class="lane-pref-list">
            <div v-for="lane in laneOptions" :key="lane.kind" class="lane-pref-row">
              <div class="lane-copy">
                <span class="lane-name">{{ lane.label }}</span>
                <span class="lane-hint">{{ lane.hint }}</span>
              </div>
              <div class="lane-pref-actions">
                <button
                  v-for="mode in laneModes"
                  :key="mode.value"
                  type="button"
                  class="lane-mode"
                  :class="{ active: lanePrefValue(lane.kind) === mode.value }"
                  @click="setLanePref(lane.kind, mode.value)"
                >
                  {{ mode.label }}
                </button>
              </div>
            </div>
          </div>
          <p class="field-hint" style="margin-top: 14px">
            <strong>Smart</strong> — UMBRA collapses or expands based on task count.
            <strong>Expanded</strong> / <strong>Collapsed</strong> — always open or always closed.
          </p>
        </GlassCard>
      </section>

      <!-- ══ Advanced ══ -->
      <section v-show="activeSection === 'advanced'" class="settings-section">
        <div class="section-header">
          <h2 class="section-title">Advanced</h2>
          <p class="section-desc">Core infrastructure settings. Only change these if you know what you're doing — incorrect values can break agent connectivity.</p>
        </div>

        <GlassCard>
          <div class="group-title">Agent communication (UAP)</div>
          <p class="group-desc">UMBRA exposes a local HTTP server that agents connect to. These settings control where that server listens.</p>

          <div class="field">
            <label class="setting-label">Listen address</label>
            <span class="setting-hint">The IP address agents connect to. Use 127.0.0.1 for local-only access. Use a LAN IP if agents run on other machines on your network.</span>
            <input v-model="draft.uapAdvertiseHost" class="field-input glass-input" type="text" placeholder="127.0.0.1" />
          </div>

          <div class="field-row">
            <div class="field">
              <label class="setting-label">Port</label>
              <span class="setting-hint">Default: 8765</span>
              <input v-model.number="draft.uapPort" class="field-input glass-input" type="number" min="1" max="65535" />
            </div>
            <div class="field">
              <label class="setting-label">Legacy shared token</label>
              <span class="setting-hint">Kept for backwards compatibility only. All new connections use per-agent tokens instead.</span>
              <input v-model="draft.uapToken" class="field-input glass-input" type="text" autocomplete="off" />
            </div>
          </div>
        </GlassCard>

        <GlassCard>
          <div class="group-title">App updates</div>
          <p class="group-desc">Set up a release feed to receive update notifications. Leave blank to disable.</p>

          <div class="field">
            <label class="setting-label">Update feed URL</label>
            <span class="setting-hint">A URL pointing to a JSON file that describes the latest release. Leave blank to disable automatic update checks.</span>
            <textarea
              v-model="draft.updaterEndpoint"
              class="field-input glass-input updater-textarea"
              rows="2"
              placeholder="https://releases.example.com/latest.json"
            />
          </div>

          <div class="field">
            <label class="setting-label">Release signing key</label>
            <span class="setting-hint">The public key used to verify update packages are authentic. Paste the full contents of publickey.pem.</span>
            <textarea
              v-model="draft.updaterPublicKey"
              class="field-input glass-input updater-textarea"
              rows="3"
              placeholder="CONTENT FROM PUBLICKEY.PEM"
            />
          </div>

          <div class="setting-row" style="margin-top: 12px">
            <div class="setting-copy">
              <span class="setting-label">Check for updates on start</span>
              <span class="setting-hint">Automatically checks the release feed each time UMBRA launches.</span>
            </div>
            <label class="toggle">
              <input v-model="draft.autoCheckForUpdates" type="checkbox" />
              <span class="toggle-track" />
            </label>
          </div>

          <div class="doc-links" style="margin-top: 14px">
            <NeonButton size="sm" variant="secondary" ghost type="button" :loading="checkingUpdates" @click="checkForUpdates">Check now</NeonButton>
            <NeonButton size="sm" variant="secondary" ghost type="button" :disabled="!canInstallUpdate" :loading="installingUpdate" @click="installUpdate">Install update</NeonButton>
          </div>
          <p class="field-hint" style="margin-top: 8px">{{ updateMessage }}</p>
        </GlassCard>
      </section>

    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { open as shellOpen } from "@tauri-apps/plugin-shell";
import type {
  AppConfig,
  ProviderAuthState,
  ProviderProbeResult,
  ProviderSetupChecklistResult,
  UpdateCheckResult,
  WorkspaceBootstrapResult,
  WorkspacePreset,
} from "@/interfaces";
import { useAgentStore } from "@/stores/useAgentStore";
import { useConfigStore } from "@/stores/useConfigStore";
import GlassCard from "@/components/ui/GlassCard.vue";
import NeonButton from "@/components/ui/NeonButton.vue";

// ── Navigation ──────────────────────────────────────────────────
const activeSection = ref("appearance");

const sections = [
  { id: "appearance", label: "Appearance",         icon: "◑" },
  { id: "notes",      label: "Notes & Vault",      icon: "◧" },
  { id: "pm",         label: "Project Management", icon: "◈" },
  { id: "agents",     label: "Agents",             icon: "◎" },
  { id: "github",     label: "GitHub",             icon: "◻" },
  { id: "launcher",   label: "Quick Launch",       icon: "◁" },
  { id: "taskboard",  label: "Taskboard",          icon: "◫" },
  { id: "advanced",   label: "Advanced",           icon: "◌" },
] as const;

// ── Stores & state ──────────────────────────────────────────────
const agentStore = useAgentStore();
const configStore = useConfigStore();
const saving = ref(false);
const saved = ref(false);
const checkingUpdates = ref(false);
const installingUpdate = ref(false);
const lastUpdateCheck = ref<UpdateCheckResult | null>(null);
const updateError = ref("");
const providerProbeResults = ref<Record<string, ProviderProbeResult | null>>({});
const probingProviders = ref<Record<string, boolean>>({});
const providerActionResults = ref<Record<string, string>>({});
const autostart = ref(false);
const providerActionBusy = ref<Record<string, boolean>>({});
const providerChecklists = ref<Record<string, ProviderSetupChecklistResult | null>>({});
const tokenVisible = ref<Record<string, boolean>>({});
const tokenCopied = ref(false);

// ── Tokens ──────────────────────────────────────────────────────
const agentTokenEntries = computed(() => {
  const tokens = draft.agentAuthTokens ?? {};
  const sorted = Object.entries(tokens).sort(([a], [b]) => a.localeCompare(b));
  return Object.fromEntries(sorted);
});

function toggleTokenVisibility(agentId: string) {
  tokenVisible.value = { ...tokenVisible.value, [agentId]: !tokenVisible.value[agentId] };
}

async function copyToken(token: string) {
  try {
    await navigator.clipboard.writeText(token);
    tokenCopied.value = true;
    setTimeout(() => { tokenCopied.value = false; }, 2000);
  } catch { /* clipboard not available */ }
}

function regenerateToken(agentId: string) {
  if (!draft.agentAuthTokens) draft.agentAuthTokens = {};
  draft.agentAuthTokens[agentId] = crypto.randomUUID();
}

// ── Provider bootstrap ───────────────────────────────────────────
const providerBootstrapWorkspaceId = ref("");
const providerBootstrapAgentId = ref("");
const providerBootstrapOverwrite = ref(false);

// ── Static data ──────────────────────────────────────────────────
const themes = [
  { value: "ember", label: "Ember",  color: "#d4520a" },
  { value: "neon",  label: "Neon",   color: "#00f5ff" },
  { value: "light", label: "Light",  color: "#00658d" },
];

const laneOptions = [
  { kind: "backlog",     label: "Backlog",      hint: "Smart = collapse only on dense boards" },
  { kind: "in_progress", label: "In progress",  hint: "Usually best left open" },
  { kind: "review",      label: "Review",       hint: "Smart = starts collapsed when populated" },
  { kind: "done",        label: "Done",         hint: "Smart = starts collapsed when populated" },
] as const;

const laneModes = [
  { value: "smart",     label: "Smart" },
  { value: "expanded",  label: "Expanded" },
  { value: "collapsed", label: "Collapsed" },
] as const;

const defaultProviderCommands = [
  { providerId: "codex",  command: "codex" },
  { providerId: "claude", command: "claude" },
  { providerId: "gemini", command: "gemini" },
  { providerId: "kimi",   command: "kimi" },
];

const defaultPersonaPresets = [
  {
    id: "implementer",
    name: "implementer",
    description: "prefer shipping code, tests, and a concise result summary.",
    prompt: "you are the implementation persona. prefer concrete code changes over discussion. finish with changed files, checks, and blockers.",
  },
  {
    id: "reviewer",
    name: "reviewer",
    description: "prioritize bugs, regressions, missing tests, and trust boundaries.",
    prompt: "you are the review persona. prioritize findings, regressions, missing tests, trust boundaries, and operational risk over summaries.",
  },
  {
    id: "architect",
    name: "architect",
    description: "plan with clear tradeoffs, rollout phases, and failure modes.",
    prompt: "you are the architecture persona. respond with a concrete plan, tradeoffs, rollout order, and critical failure modes before implementation details.",
  },
];

// ── Computed URLs ────────────────────────────────────────────────
const pmApiUrl = computed(() => draft.pmToolUrl.trim().replace(/\/+$/, ""));
const pmDocsUrl = computed(() => {
  if (!pmApiUrl.value) return "";
  return `${pmApiUrl.value}/docs`;
});
const pmDashboardUrl = computed(() => {
  const explicit = draft.pmToolDashboardUrl.trim().replace(/\/+$/, "");
  if (explicit) return explicit;
  if (!pmApiUrl.value) return "";
  try {
    const url = new URL(pmApiUrl.value);
    if (url.port === "8000") url.port = "4173";
    url.pathname = "";
    url.search = "";
    url.hash = "";
    return url.toString().replace(/\/+$/, "");
  } catch {
    return "";
  }
});
const canInstallUpdate = computed(() => Boolean(lastUpdateCheck.value?.updateAvailable) && !installingUpdate.value);
const updateMessage = computed(() => {
  if (updateError.value) return updateError.value;
  if (!draft.updaterEndpoint.trim() || !draft.updaterPublicKey.trim()) {
    return "Set the update feed URL and signing key to enable runtime update checks.";
  }
  if (checkingUpdates.value) return "Checking release feed…";
  if (installingUpdate.value) return "Installing update — UMBRA will restart if an update is ready.";
  if (!lastUpdateCheck.value) return "No check run yet.";
  if (!lastUpdateCheck.value.configured) return "Updater configuration incomplete.";
  if (lastUpdateCheck.value.updateAvailable) {
    return `Update ${lastUpdateCheck.value.version} is ready to install.`;
  }
  return `Up to date. Current version: ${lastUpdateCheck.value.currentVersion}.`;
});

// ── Provider helpers ─────────────────────────────────────────────
function applyTheme(t: string) {
  draft.theme = t;
  configStore.setTheme(t);
}

function mergeProviderCommands(commands: AppConfig["providerCommands"] | undefined) {
  const byProvider = new Map(
    (commands ?? [])
      .filter((entry) => entry.providerId?.trim() && entry.command?.trim())
      .map((entry) => [entry.providerId.trim().toLowerCase(), entry.command.trim()]),
  );
  const merged = defaultProviderCommands.map((entry) => ({
    providerId: entry.providerId,
    command: byProvider.get(entry.providerId) ?? entry.command,
  }));
  for (const [providerId, command] of byProvider.entries()) {
    if (!merged.some((entry) => entry.providerId === providerId)) {
      merged.push({ providerId, command });
    }
  }
  return merged;
}

function mergePersonaPresets(personas: AppConfig["personaPresets"] | undefined) {
  const normalized = (personas ?? [])
    .filter((persona) => persona.name?.trim() && persona.prompt?.trim())
    .map((persona) => ({
      id: persona.id?.trim() || crypto.randomUUID(),
      name: persona.name.trim(),
      description: persona.description?.trim() ?? "",
      prompt: persona.prompt.trim(),
    }));
  return normalized.length > 0 ? normalized : defaultPersonaPresets.map((p) => ({ ...p }));
}

function providerLabel(providerId: string) {
  switch (providerId) {
    case "codex":  return "OpenAI Codex";
    case "claude": return "Claude Code";
    case "gemini": return "Gemini CLI";
    case "kimi":   return "Kimi";
    default:       return providerId;
  }
}

function providerDocs(providerId: string) {
  switch (providerId) {
    case "codex":  return "https://openai.com/index/unlocking-the-codex-harness/";
    case "claude": return "https://docs.anthropic.com/en/docs/claude-code/cli-reference";
    case "gemini": return "https://github.com/google-gemini/gemini-cli";
    case "kimi":   return "https://platform.moonshot.ai/docs/guide/agent-support";
    default:       return "https://github.com";
  }
}

function providerTemplate(providerId: string) {
  switch (providerId) {
    case "codex":  return "templates/AGENTS.codex.md";
    case "claude": return "templates/CLAUDE.md";
    case "gemini": return "templates/GEMINI.md";
    default:       return "templates/worker.env.example";
  }
}

const agentSetupGuideUrl =
  "https://github.com/AstroGolem224/UMBRA/blob/main/docs/agent-setup-guide-2026-03-25.md";

const workspaceOptions = computed(() =>
  (draft.workspacePresets ?? []).filter((ws) => ws.name.trim() && ws.rootPath.trim()),
);
const bootstrapAgents = computed(() =>
  agentStore.agents.filter((agent) => ["online", "idle", "working"].includes(agent.status)),
);

// ── Draft state ──────────────────────────────────────────────────
const draft = reactive<AppConfig>({
  ...configStore.config,
  workspaceGrantRoots: [...(configStore.config.workspaceGrantRoots ?? [])],
  personaPresets: mergePersonaPresets(configStore.config.personaPresets),
  providerCommands: mergeProviderCommands(configStore.config.providerCommands),
});

watch(
  () => configStore.config,
  (c) =>
    Object.assign(draft, {
      ...c,
      workspaceGrantRoots: [...(c.workspaceGrantRoots ?? [])],
      personaPresets: mergePersonaPresets(c.personaPresets),
      providerCommands: mergeProviderCommands(c.providerCommands),
    }),
  { deep: true },
);

// ── Lifecycle ────────────────────────────────────────────────────
onMounted(async () => {
  if (!agentStore.agents.length) await agentStore.loadAgents();
  ensureBootstrapSelections();
  await refreshAllProviderChecklists();
  try {
    autostart.value = await invoke<boolean>("plugin:autostart|is_enabled");
  } catch { /* not available in browser mode */ }
});

// ── Actions ──────────────────────────────────────────────────────
async function toggleAutostart() {
  try {
    if (autostart.value) {
      await invoke("plugin:autostart|enable");
    } else {
      await invoke("plugin:autostart|disable");
    }
  } catch { /* not available in browser mode */ }
}

function addLaunchTarget() {
  if (!draft.launchTargets) draft.launchTargets = [];
  draft.launchTargets.push({ id: crypto.randomUUID(), name: "", path: "", icon: "LN" });
}

function addWorkspacePreset() {
  draft.workspacePresets.push({
    id: crypto.randomUUID(),
    name: "",
    rootPath: "",
    writable: true,
    allowedProviders: [],
    allowedAgents: [],
  });
}

function addPersonaPreset() {
  draft.personaPresets.push({ id: crypto.randomUUID(), name: "", description: "", prompt: "" });
}

function addWorkspaceGrantRoot() {
  draft.workspaceGrantRoots = [...(draft.workspaceGrantRoots ?? []), ""];
}

function seedWorkspaceGrantRoots() {
  draft.workspaceGrantRoots = mergeWorkspaceGrantRoots(draft.workspaceGrantRoots, draft.workspacePresets);
}

function workspaceProvidersValue(workspace: WorkspacePreset) {
  return (workspace.allowedProviders ?? []).join(", ");
}

function setWorkspaceProviders(workspace: WorkspacePreset, value: string) {
  workspace.allowedProviders = parseCsvList(value);
}

function workspaceAgentsValue(workspace: WorkspacePreset) {
  return (workspace.allowedAgents ?? []).join(", ");
}

function setWorkspaceAgents(workspace: WorkspacePreset, value: string) {
  workspace.allowedAgents = parseCsvList(value);
}

function parseCsvList(value: string) {
  return value.split(",").map((e) => e.trim()).filter(Boolean);
}

function mergeWorkspacePresets(presets: AppConfig["workspacePresets"] | undefined) {
  return (presets ?? [])
    .filter((p) => p.name?.trim() || p.rootPath?.trim())
    .map((p) => ({
      id: p.id?.trim() || crypto.randomUUID(),
      name: p.name.trim(),
      rootPath: p.rootPath.trim(),
      writable: p.writable !== false,
      allowedProviders: [...new Set((p.allowedProviders ?? []).map((e) => e.trim()).filter(Boolean))],
      allowedAgents: [...new Set((p.allowedAgents ?? []).map((e) => e.trim()).filter(Boolean))],
    }))
    .filter((p) => p.name && p.rootPath);
}

function mergeWorkspaceGrantRoots(roots: string[] | undefined, presets: AppConfig["workspacePresets"] | undefined) {
  const next = new Set<string>();
  for (const root of roots ?? []) { if (root.trim()) next.add(root.trim()); }
  for (const preset of presets ?? []) { if (preset.rootPath?.trim()) next.add(preset.rootPath.trim()); }
  return [...next].sort((a, b) => a.localeCompare(b));
}

function ensureBootstrapSelections() {
  if (!providerBootstrapWorkspaceId.value && workspaceOptions.value.length > 0) {
    providerBootstrapWorkspaceId.value = draft.defaultWorkspaceId || workspaceOptions.value[0].id;
  }
  if (!providerBootstrapAgentId.value && bootstrapAgents.value.length > 0) {
    providerBootstrapAgentId.value = bootstrapAgents.value[0].id;
  }
}

async function pickWorkspaceRoot(index: number) {
  const selection = await open({ directory: true, multiple: false, title: "Select workspace root" });
  if (typeof selection === "string") draft.workspacePresets[index].rootPath = selection;
}

async function pickGrantRoot(index: number) {
  const selection = await open({ directory: true, multiple: false, title: "Select allowed root path" });
  if (typeof selection === "string") draft.workspaceGrantRoots[index] = selection;
}

async function pickVaultPath() {
  const selection = await open({ directory: true, multiple: false, title: "Select Obsidian vault folder" });
  if (typeof selection === "string") draft.vaultPath = selection;
}

async function pickRepoRootPath() {
  const selection = await open({ directory: true, multiple: false, title: "Select local repos folder" });
  if (typeof selection === "string") draft.repoRootPath = selection;
}

function lanePrefValue(kind: (typeof laneOptions)[number]["kind"]) {
  const pref = draft.taskLanePrefs?.[kind];
  if (pref === true) return "collapsed";
  if (pref === false) return "expanded";
  return "smart";
}

function setLanePref(kind: (typeof laneOptions)[number]["kind"], value: (typeof laneModes)[number]["value"]) {
  const next = { ...(draft.taskLanePrefs ?? {}) };
  if (value === "smart") { delete next[kind]; } else { next[kind] = value === "collapsed"; }
  draft.taskLanePrefs = next;
}

async function openExternal(url: string) {
  if (!url) return;
  try { await shellOpen(url); } catch { window.open(url, "_blank"); }
}

async function probeProvider(providerId: string) {
  probingProviders.value = { ...probingProviders.value, [providerId]: true };
  try {
    providerProbeResults.value = {
      ...providerProbeResults.value,
      [providerId]: await invoke<ProviderProbeResult>("probe_provider_command", { providerId }),
    };
  } catch (error) {
    providerProbeResults.value = {
      ...providerProbeResults.value,
      [providerId]: { providerId, command: "", launchable: false, summary: `probe failed: ${String(error)}` },
    };
  } finally {
    probingProviders.value = { ...probingProviders.value, [providerId]: false };
  }
}

function requireBootstrapTarget(providerId: string) {
  ensureBootstrapSelections();
  if (!providerBootstrapWorkspaceId.value) {
    providerActionResults.value = { ...providerActionResults.value, [providerId]: "select a target workspace first" };
    return null;
  }
  return { workspaceId: providerBootstrapWorkspaceId.value, agentId: providerBootstrapAgentId.value || undefined };
}

async function checkProviderAuth(providerId: string) {
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<ProviderAuthState>("get_provider_auth_state", { providerId });
    providerActionResults.value = { ...providerActionResults.value, [providerId]: result.summary };
  } catch (error) {
    providerActionResults.value = { ...providerActionResults.value, [providerId]: `auth check failed: ${String(error)}` };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function refreshProviderChecklist(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<ProviderSetupChecklistResult>("check_provider_setup", {
      providerId,
      workspaceId: target.workspaceId,
    });
    providerChecklists.value = { ...providerChecklists.value, [providerId]: result };
    providerActionResults.value = { ...providerActionResults.value, [providerId]: result.summary };
  } catch (error) {
    providerActionResults.value = { ...providerActionResults.value, [providerId]: `checklist failed: ${String(error)}` };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function refreshAllProviderChecklists() {
  if (!providerBootstrapWorkspaceId.value) return;
  for (const provider of draft.providerCommands) {
    await refreshProviderChecklist(provider.providerId);
  }
}

async function smokeTestProvider(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<ProviderProbeResult>("smoke_test_provider_command", {
      providerId,
      workspaceId: target.workspaceId,
    });
    providerActionResults.value = { ...providerActionResults.value, [providerId]: result.summary };
  } catch (error) {
    providerActionResults.value = { ...providerActionResults.value, [providerId]: `smoke test failed: ${String(error)}` };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function writeProviderBootstrap(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  providerActionBusy.value = { ...providerActionBusy.value, [providerId]: true };
  try {
    const result = await invoke<WorkspaceBootstrapResult>("bootstrap_provider_workspace", {
      providerId,
      workspaceId: target.workspaceId,
      agentId: target.agentId,
      overwrite: providerBootstrapOverwrite.value,
    });
    providerActionResults.value = { ...providerActionResults.value, [providerId]: `${result.summary}: ${result.files.join(", ")}` };
    await refreshProviderChecklist(providerId);
  } catch (error) {
    providerActionResults.value = { ...providerActionResults.value, [providerId]: `bootstrap failed: ${String(error)}` };
  } finally {
    providerActionBusy.value = { ...providerActionBusy.value, [providerId]: false };
  }
}

async function copyProviderEnv(providerId: string) {
  const target = requireBootstrapTarget(providerId);
  if (!target) return;
  try {
    const content = await invoke<string>("get_provider_env_template", { providerId, agentId: target.agentId });
    await navigator.clipboard.writeText(content);
    providerActionResults.value = { ...providerActionResults.value, [providerId]: "worker env copied to clipboard" };
  } catch (error) {
    providerActionResults.value = { ...providerActionResults.value, [providerId]: `copy env failed: ${String(error)}` };
  }
}

async function checkForUpdates() {
  checkingUpdates.value = true;
  updateError.value = "";
  try {
    lastUpdateCheck.value = await invoke<UpdateCheckResult>("check_for_updates");
  } catch (error) {
    updateError.value = `Update check failed: ${String(error)}`;
  } finally {
    checkingUpdates.value = false;
  }
}

async function installUpdate() {
  if (!canInstallUpdate.value) return;
  installingUpdate.value = true;
  updateError.value = "";
  try {
    const installed = await invoke<boolean>("install_pending_update");
    if (!installed) { updateError.value = "No pending update available."; return; }
    lastUpdateCheck.value = null;
  } catch (error) {
    updateError.value = `Update install failed: ${String(error)}`;
  } finally {
    installingUpdate.value = false;
  }
}

async function save() {
  saving.value = true;
  saved.value = false;
  try {
    await configStore.saveConfig({
      ...draft,
      workspacePresets: mergeWorkspacePresets(draft.workspacePresets),
      workspaceGrantRoots: mergeWorkspaceGrantRoots(draft.workspaceGrantRoots, draft.workspacePresets),
      personaPresets: mergePersonaPresets(draft.personaPresets),
      providerCommands: mergeProviderCommands(draft.providerCommands),
    });
    saved.value = true;
    setTimeout(() => (saved.value = false), 2500);
  } finally {
    saving.value = false;
  }
}

watch([providerBootstrapWorkspaceId, providerBootstrapAgentId], () => {
  void refreshAllProviderChecklists();
});
</script>

<style scoped>
/* ── Layout ─────────────────────────────────────────────────────── */
.settings-layout {
  display: grid;
  grid-template-columns: 200px 1fr;
  height: 100%;
  overflow: hidden;
}

/* ── Left nav ───────────────────────────────────────────────────── */
.settings-nav {
  display: flex;
  flex-direction: column;
  gap: 0;
  padding: 20px 12px 16px;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--glass-border);
  overflow-y: auto;
}

.nav-brand {
  padding: 0 8px 20px;
  border-bottom: 1px solid var(--glass-border);
  margin-bottom: 12px;
}

.nav-kicker {
  display: block;
  font-family: var(--font-mono);
  font-size: 9px;
  letter-spacing: 0.16em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.nav-title {
  font-family: var(--font-display);
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 0.06em;
  color: var(--text-primary);
  margin: 0;
}

.nav-items {
  display: flex;
  flex-direction: column;
  gap: 2px;
  flex: 1;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: var(--radius-md);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  text-align: left;
  transition: background 0.15s, color 0.15s;
  width: 100%;
}

.nav-item:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-dim);
  color: var(--accent);
  font-weight: 600;
}

.nav-icon {
  font-size: 14px;
  width: 16px;
  text-align: center;
  flex-shrink: 0;
  opacity: 0.8;
}

.nav-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-footer {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 16px;
  border-top: 1px solid var(--glass-border);
  margin-top: 12px;
}

.saved-badge {
  font-size: 12px;
  color: var(--accent-success);
  text-align: center;
}

/* ── Content panel ──────────────────────────────────────────────── */
.settings-content {
  overflow-y: auto;
  padding: 24px var(--stage-edge-pad, 18px);
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 14px;
  max-width: 720px;
}

.section-header {
  margin-bottom: 4px;
}

.section-title {
  font-family: var(--font-display);
  font-size: 24px;
  font-weight: 700;
  letter-spacing: 0.04em;
  color: var(--text-primary);
  margin: 0 0 6px;
}

.section-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin: 0;
  line-height: 1.6;
}

/* ── Card internals ─────────────────────────────────────────────── */
.group-title {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-muted);
  margin-bottom: 6px;
}

.group-desc {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 14px;
  line-height: 1.6;
}

/* ── Setting rows ───────────────────────────────────────────────── */
.setting-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 8px 0;
}

.setting-copy {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
}

.setting-label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.setting-hint {
  display: block;
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.55;
}

.setting-divider {
  height: 1px;
  background: var(--glass-border);
  margin: 4px 0;
}

/* ── Inputs ─────────────────────────────────────────────────────── */
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-bottom: 14px;
}

.field:last-child {
  margin-bottom: 0;
}

.field-label,
.card-title {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.14em;
  text-transform: uppercase;
  color: var(--text-muted);
}

.field-input {
  font-size: 13px;
  padding: 10px 12px;
}

.field-row {
  display: grid;
  grid-template-columns: 140px 1fr;
  gap: 12px;
}

.field-hint {
  font-size: 12px;
  color: var(--text-muted);
  line-height: 1.55;
}

.success-text {
  color: var(--accent-success) !important;
}

/* ── Path row (input + browse) ──────────────────────────────────── */
.path-row,
.grant-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.path-row .glass-input,
.grant-row .glass-input {
  flex: 1;
  min-width: 0;
}

/* ── Number input ───────────────────────────────────────────────── */
.number-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.number-input {
  width: 100px;
}

.unit-label {
  font-size: 12px;
  color: var(--text-muted);
}

/* ── Toggle control ─────────────────────────────────────────────── */
.toggle {
  position: relative;
  flex-shrink: 0;
  cursor: pointer;
}

.toggle input {
  position: absolute;
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-track {
  display: block;
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: var(--bg-elevated);
  border: 1px solid var(--glass-border);
  position: relative;
  transition: background 0.18s, border-color 0.18s;
}

.toggle-track::after {
  content: '';
  position: absolute;
  top: 3px;
  left: 3px;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--text-muted);
  transition: transform 0.18s, background 0.18s;
}

.toggle input:checked + .toggle-track {
  background: var(--accent);
  border-color: var(--accent);
}

.toggle input:checked + .toggle-track::after {
  transform: translateX(18px);
  background: #fff;
}

/* ── Inline checkbox ─────────────────────────────────────────────── */
.inline-check {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
}

.checkbox-row.compact {
  margin-bottom: 0;
}

/* ── Theme swatches ─────────────────────────────────────────────── */
.theme-swatches {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
}

.theme-swatch {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  border-radius: var(--radius-md);
  border: 1px solid var(--glass-border);
  background: transparent;
  color: var(--text-secondary);
  font-size: 13px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s, background 0.15s;
}

.theme-swatch:hover {
  border-color: var(--accent);
  color: var(--text-primary);
}

.theme-swatch.active {
  border-color: var(--accent);
  background: var(--accent-dim);
  color: var(--accent);
  font-weight: 600;
}

.swatch-dot {
  width: 12px;
  height: 12px;
  border-radius: 50%;
  flex-shrink: 0;
}

/* ── Workspace card ─────────────────────────────────────────────── */
.workspace-card {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 12px;
  padding: 14px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--glass-border);
  background: color-mix(in srgb, var(--bg-secondary) 60%, transparent);
}

.workspace-row__head {
  display: flex;
  gap: 8px;
  align-items: center;
  flex-wrap: wrap;
}

.workspace-name {
  flex: 1;
  min-width: 120px;
}

.workspace-actions {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-top: 4px;
}

.workspace-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
}

/* ── Provider block ─────────────────────────────────────────────── */
.provider-bootstrap-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  margin-bottom: 12px;
}

.provider-block {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 14px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--glass-border);
  background: color-mix(in srgb, var(--bg-secondary) 50%, transparent);
  margin-bottom: 10px;
}

.provider-block__header {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  flex-wrap: wrap;
}

.provider-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 140px;
  flex-shrink: 0;
}

.provider-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
}

.provider-hint,
.provider-template {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
}

.provider-command {
  flex: 1;
  min-width: 0;
}

.provider-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.provider-result {
  font-size: 11px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  white-space: pre-wrap;
  word-break: break-word;
}

.provider-checklist {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.checklist-item {
  display: flex;
  gap: 8px;
  align-items: baseline;
  font-size: 12px;
  padding: 6px 10px;
  border-radius: var(--radius-sm);
}

.checklist-item.ready {
  background: color-mix(in srgb, var(--accent-success) 10%, transparent);
  color: var(--accent-success);
}

.checklist-item.missing {
  background: color-mix(in srgb, var(--accent-error) 8%, transparent);
  color: var(--accent-error);
}

.checklist-status {
  font-family: var(--font-mono);
  font-size: 11px;
  flex-shrink: 0;
}

/* ── Token table ─────────────────────────────────────────────────── */
.token-table {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin-top: 4px;
}

.token-row {
  display: grid;
  grid-template-columns: 100px 1fr auto;
  gap: 10px;
  align-items: center;
  padding: 8px 10px;
  border-radius: var(--radius-md);
}

.token-row--head {
  font-family: var(--font-mono);
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--text-muted);
  padding-bottom: 4px;
}

.token-row:not(.token-row--head) {
  background: color-mix(in srgb, var(--bg-secondary) 50%, transparent);
}

.token-agent {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
}

.token-value-cell {
  display: flex;
  gap: 6px;
  align-items: center;
}

.token-input {
  flex: 1;
  min-width: 0;
  font-family: var(--font-mono);
  font-size: 12px;
}

.token-actions-cell {
  display: flex;
  gap: 6px;
}

.token-action {
  padding: 4px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--glass-border);
  background: transparent;
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  white-space: nowrap;
}

.token-action:hover {
  background: var(--accent-dim);
  color: var(--accent);
}

.token-action--danger:hover {
  background: color-mix(in srgb, var(--accent-error) 12%, transparent);
  color: var(--accent-error);
  border-color: var(--accent-error);
}

/* ── Persona rows ────────────────────────────────────────────────── */
.persona-row {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  border-radius: var(--radius-lg);
  border: 1px solid var(--glass-border);
  background: color-mix(in srgb, var(--bg-secondary) 50%, transparent);
  margin-bottom: 10px;
}

.persona-row__head {
  display: flex;
  gap: 8px;
  align-items: center;
}

.persona-name {
  flex: 1;
}

.persona-prompt {
  min-height: 80px;
  resize: vertical;
}

/* ── Launch rows ─────────────────────────────────────────────────── */
.launch-row {
  display: flex;
  gap: 8px;
  align-items: center;
  margin-bottom: 8px;
  flex-wrap: wrap;
}

.launch-name {
  width: 160px;
}

.launch-path {
  flex: 1;
  min-width: 200px;
}

/* ── Lane prefs ──────────────────────────────────────────────────── */
.lane-pref-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.lane-pref-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 8px 0;
}

.lane-copy {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.lane-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  text-transform: capitalize;
}

.lane-hint {
  font-size: 11px;
  color: var(--text-muted);
}

.lane-pref-actions {
  display: flex;
  gap: 4px;
}

.lane-mode {
  padding: 5px 12px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--glass-border);
  background: transparent;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s, border-color 0.15s;
}

.lane-mode:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.lane-mode.active {
  background: var(--accent-dim);
  border-color: var(--accent);
  color: var(--accent);
  font-weight: 600;
}

/* ── Updater textarea ────────────────────────────────────────────── */
.updater-textarea {
  min-height: 60px;
  resize: vertical;
}

/* ── Doc links ───────────────────────────────────────────────────── */
.doc-links {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 8px;
}

/* ── Fade transition ─────────────────────────────────────────────── */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
