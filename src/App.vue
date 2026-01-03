<script setup lang="ts">
import { ref, watch, onMounted } from "vue";

type Theme = "light" | "dark" | "auto";

const autoStart = ref(false);
const theme = ref<Theme>("auto");

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
watch(autoStart, (newValue) => {
  localStorage.setItem("autoStart", String(newValue));
  // TODO: Invoke Tauri command to set auto-start
  // invoke("set_auto_start", { enabled: newValue });
});

// Load saved settings on mount
onMounted(() => {
  const savedTheme = localStorage.getItem("theme") as Theme | null;
  if (savedTheme) {
    theme.value = savedTheme;
  }
  applyTheme(theme.value);

  const savedAutoStart = localStorage.getItem("autoStart");
  if (savedAutoStart !== null) {
    autoStart.value = savedAutoStart === "true";
  }

  // Listen for system theme changes when in auto mode
  window.matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => {
    if (theme.value === "auto") {
      applyTheme("auto");
    }
  });
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