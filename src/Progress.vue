<template>
  <div class="progress-container">
    <template v-if="state === 'loading'">
      <div class="spinner"></div>
      <div>
        <div class="text">Optimizing image...</div>
        <div class="text-secondary">Please wait</div>
      </div>
    </template>
    <template v-else-if="state === 'finished'">
      <div class="check-icon">✓</div>
      <div class="content">
        <div class="text">{{ statistics }}</div>
      </div>
      <!-- <button class="revert-btn" @click="revert">Revert</button> -->
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

type ProgressState = 'loading' | 'finished';

const state = ref<ProgressState>('loading');
const originalSize = ref<number>(0);
const newSize = ref<number>(0);
let autoHideTimer: ReturnType<typeof setTimeout> | null = null;
let unlistenStart: UnlistenFn | null = null;
let unlistenComplete: UnlistenFn | null = null;

function formatSize(size: number): string {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  return `${(size / (1024 * 1024)).toFixed(2)} MB`;
}

const statistics = computed(() => {
  return `${formatSize(originalSize.value)} → ${formatSize(newSize.value)}`;
});

function startAutoHideTimer() {
  clearAutoHideTimer();
  autoHideTimer = setTimeout(() => {
    invoke('hide_progress');
  }, 5000);
}

function clearAutoHideTimer() {
  if (autoHideTimer) {
    clearTimeout(autoHideTimer);
    autoHideTimer = null;
  }
}

// async function revert() {
//   clearAutoHideTimer();
//   await invoke('revert_clipboard');
// }

onMounted(async () => {
  unlistenStart = await listen('optimization-start', () => {
    state.value = 'loading';
    clearAutoHideTimer();
  });

  unlistenComplete = await listen<{ original_size: number, new_size: number }>('optimization-complete', (event) => {
    console.log('Received optimization-complete event:', event);
    state.value = 'finished';
    originalSize.value = event.payload.original_size;
    newSize.value = event.payload.new_size;
    startAutoHideTimer();
  });
});

onUnmounted(() => {
  clearAutoHideTimer();
  unlistenStart?.();
  unlistenComplete?.();
});
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.progress-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px 20px;
  width: 100vw;
  height: 100vh;
  background: rgba(30, 30, 30, 0.95);
  border-radius: 0px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
}

.spinner {
  width: 28px;
  height: 28px;
  border: 3px solid rgba(255, 255, 255, 0.2);
  border-top-color: #4fc3f7;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.check-icon {
  width: 28px;
  height: 28px;
  background: #4caf50;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-size: 16px;
  font-weight: bold;
  flex-shrink: 0;
}

.content {
  flex: 1;
  min-width: 0;
}

.text {
  color: #ffffff;
  font-size: 14px;
  font-weight: 500;
}

.text-secondary {
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
  margin-top: 2px;
}

.revert-btn {
  padding: 6px 12px;
  background: rgba(255, 255, 255, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 4px;
  color: #ffffff;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.2s;
  flex-shrink: 0;
}

.revert-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}
</style>

<style>
  body {
    margin: 0;
  }
</style>