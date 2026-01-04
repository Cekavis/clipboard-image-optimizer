<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { check, Update } from "@tauri-apps/plugin-updater";

type Theme = "light" | "dark" | "auto";
type UpdateStatus = "idle" | "checking" | "up-to-date" | "available" | "downloading" | "error";

const autoStart = ref(false);
const theme = ref<Theme>("auto");

// Update-related state
const currentVersion = ref("");
const updateStatus = ref<UpdateStatus>("idle");
const availableUpdate = ref<Update | null>(null);
const updateError = ref("");
const downloadProgress = ref(0);
let updateCheckInterval: ReturnType<typeof setInterval> | null = null;

// Check for updates
async function checkForUpdates() {
  updateStatus.value = "checking";
  updateError.value = "";
  
  try {
    const update = await check();
    if (update) {
      availableUpdate.value = update;
      updateStatus.value = "available";
    } else {
      availableUpdate.value = null;
      updateStatus.value = "up-to-date";
    }
  } catch (error) {
    console.error("Failed to check for updates:", error);
    updateError.value = String(error);
    updateStatus.value = "error";
  }
}

// Download and install update
async function downloadAndInstall() {
  if (!availableUpdate.value) return;
  
  updateStatus.value = "downloading";
  downloadProgress.value = 0;
  
  try {
    await availableUpdate.value.downloadAndInstall((event) => {
      if (event.event === "Started" && event.data.contentLength) {
        downloadProgress.value = 0;
      } else if (event.event === "Progress") {
        downloadProgress.value += event.data.chunkLength;
      } else if (event.event === "Finished") {
        downloadProgress.value = 100;
      }
    });
  } catch (error) {
    console.error("Failed to download/install update:", error);
    updateError.value = String(error);
    updateStatus.value = "error";
  }
}

// Apply theme to document
function applyTheme(selectedTheme: Theme) {
  const root = document.documentElement;
  
  if (selectedTheme === "auto") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.setAttribute("data-theme", prefersDark ? "dark" : "light");
  } else {
    root.setAttribute("data-theme", selectedTheme);
  }
}

// Watch for theme changes
watch(theme, (newTheme) => {
  applyTheme(newTheme);
  localStorage.setItem("theme", newTheme);
});

// Watch for auto-start changes
watch(autoStart, async (newValue) => {
  try {
    await invoke("set_auto_start", { enabled: newValue });
    localStorage.setItem("autoStart", String(newValue));
  } catch (error) {
    console.error("Failed to set auto-start:", error);
  }
});

// Load saved settings on mount
onMounted(async () => {
  // Get current version
  try {
    currentVersion.value = await getVersion();
  } catch (error) {
    console.error("Failed to get version:", error);
  }

  const savedTheme = localStorage.getItem("theme") as Theme | null;
  if (savedTheme) {
    theme.value = savedTheme;
  }
  applyTheme(theme.value);

  // Load autostart state from the system
  try {
    autoStart.value = await invoke<boolean>("get_auto_start");
  } catch (error) {
    console.error("Failed to get auto-start state:", error);
  }

  // Listen for system theme changes when in auto mode
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
    if (theme.value === "auto") {
      applyTheme("auto");
    }
  });

  // Check for updates on startup
  checkForUpdates();
  
  // Periodically check for updates (every 12 hours)
  updateCheckInterval = setInterval(() => {
    checkForUpdates();
  }, 12 * 60 * 60 * 1000);
});

onUnmounted(() => {
  if (updateCheckInterval) {
    clearInterval(updateCheckInterval);
  }
});
</script>

<template>
  <main class="container">
    <h1>Settings</h1>
    
    <div class="settings-list">
      <!-- Auto-start setting -->
      <div class="setting-item">
        <div class="setting-info">
          <label for="auto-start">Start on login</label>
          <span class="setting-description">Automatically start the app when you log in</span>
        </div>
        <label class="toggle">
          <input 
            type="checkbox" 
            id="auto-start" 
            v-model="autoStart"
          />
          <span class="toggle-slider"></span>
        </label>
      </div>

      <!-- Theme setting -->
      <div class="setting-item">
        <div class="setting-info">
          <label>Theme</label>
          <span class="setting-description">Choose your preferred appearance</span>
        </div>
        <div class="theme-selector">
          <button 
            :class="['theme-btn', { active: theme === 'light' }]"
            @click="theme = 'light'"
            title="Light"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="5"/>
              <line x1="12" y1="1" x2="12" y2="3"/>
              <line x1="12" y1="21" x2="12" y2="23"/>
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/>
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/>
              <line x1="1" y1="12" x2="3" y2="12"/>
              <line x1="21" y1="12" x2="23" y2="12"/>
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/>
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/>
            </svg>
          </button>
          <button 
            :class="['theme-btn', { active: theme === 'dark' }]"
            @click="theme = 'dark'"
            title="Dark"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>
            </svg>
          </button>
          <button 
            :class="['theme-btn', { active: theme === 'auto' }]"
            @click="theme = 'auto'"
            title="Auto"
          >
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="12" cy="12" r="10"/>
              <path d="M12 2a10 10 0 0 1 0 20"/>
            </svg>
          </button>
        </div>
      </div>

      <!-- Version & Update setting -->
      <div class="setting-item update-section">
        <div class="setting-info">
          <label>Version</label>
          <span class="setting-description">
            v{{ currentVersion }}
            <span v-if="updateStatus === 'available' && availableUpdate" class="new-version">
              → v{{ availableUpdate.version }} available
            </span>
          </span>
        </div>
        <div class="update-controls">
          <!-- Check for updates button -->
          <button 
            v-if="updateStatus === 'idle' || updateStatus === 'up-to-date' || updateStatus === 'error'"
            class="update-btn"
            @click="checkForUpdates"
          >
            Check for updates
          </button>
          
          <!-- Checking status -->
          <span v-if="updateStatus === 'checking'" class="update-status checking">
            <span class="spinner"></span> Checking...
          </span>
          
          <!-- Up to date status -->
          <span v-if="updateStatus === 'up-to-date'" class="update-status up-to-date">
            ✓ Up to date
          </span>
          
          <!-- Download & Install button -->
          <button 
            v-if="updateStatus === 'available'"
            class="update-btn primary"
            @click="downloadAndInstall"
          >
            Download & Install
          </button>
          
          <!-- Downloading status -->
          <span v-if="updateStatus === 'downloading'" class="update-status downloading">
            <span class="spinner"></span> Downloading...
          </span>
          
          <!-- Error status -->
          <span v-if="updateStatus === 'error'" class="update-status error" :title="updateError">
            ✗ Error
          </span>
        </div>
      </div>
    </div>
  </main>
</template>

<style scoped>
.container {
  max-width: 500px;
  margin: 0 auto;
  padding: 2rem;
}

h1 {
  font-size: 1.5rem;
  font-weight: 600;
  margin-bottom: 1.5rem;
  color: var(--text-color);
}

.settings-list {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem;
  background: var(--card-bg);
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.setting-info {
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.setting-info label {
  font-weight: 500;
  color: var(--text-color);
}

.setting-description {
  font-size: 0.85rem;
  color: var(--text-secondary);
}

/* Toggle switch */
.toggle {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 28px;
}

.toggle input {
  opacity: 0;
  width: 0;
  height: 0;
}

.toggle-slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--toggle-bg);
  transition: 0.2s;
  border-radius: 28px;
}

.toggle-slider:before {
  position: absolute;
  content: "";
  height: 22px;
  width: 22px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: 0.2s;
  border-radius: 50%;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.toggle input:checked + .toggle-slider {
  background-color: var(--accent-color);
}

.toggle input:checked + .toggle-slider:before {
  transform: translateX(20px);
}

/* Theme selector */
.theme-selector {
  display: flex;
  gap: 0.5rem;
  background: var(--toggle-bg);
  padding: 4px;
  border-radius: 10px;
}

.theme-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 36px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.2s;
}

.theme-btn:hover {
  color: var(--text-color);
}

.theme-btn.active {
  background: var(--card-bg);
  color: var(--accent-color);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

/* Update section */
.update-section {
  flex-wrap: wrap;
  gap: 0.75rem;
}

.update-controls {
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.update-btn {
  padding: 0.5rem 1rem;
  border: 1px solid var(--border-color);
  background: var(--card-bg);
  color: var(--text-color);
  border-radius: 8px;
  cursor: pointer;
  font-size: 0.85rem;
  transition: all 0.2s;
}

.update-btn:hover {
  background: var(--toggle-bg);
}

.update-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.update-btn.primary {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.update-btn.primary:hover {
  opacity: 0.9;
}

.update-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-size: 0.85rem;
  padding: 0.5rem 0.75rem;
  border-radius: 8px;
}

.update-status.checking,
.update-status.downloading {
  color: var(--text-secondary);
}

.update-status.up-to-date {
  color: #34c759;
}

.update-status.error {
  color: #ff3b30;
}

.new-version {
  color: var(--accent-color);
  font-weight: 500;
}

.spinner {
  width: 14px;
  height: 14px;
  border: 2px solid var(--border-color);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>

<style>
:root {
  --text-color: #1a1a1a;
  --text-secondary: #666;
  --card-bg: #fff;
  --bg-color: #f5f5f5;
  --border-color: #e5e5e5;
  --toggle-bg: #e0e0e0;
  --accent-color: #007aff;
}

[data-theme="dark"] {
  --text-color: #f5f5f5;
  --text-secondary: #999;
  --card-bg: #2a2a2a;
  --bg-color: #1a1a1a;
  --border-color: #3a3a3a;
  --toggle-bg: #404040;
  --accent-color: #0a84ff;
}

html, body {
  background-color: var(--bg-color);
  color: var(--text-color);
  transition: background-color 0.2s, color 0.2s;
}
</style>