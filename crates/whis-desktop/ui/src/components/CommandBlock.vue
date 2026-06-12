<!-- CommandBlock: Copyable command display. Props: command, segments (highlighted parts), copyable -->
<script setup lang="ts">
import { ref } from 'vue'

export interface CommandSegment {
  text: string
  highlight?: boolean
}

const props = withDefaults(defineProps<{
  command: string
  segments?: CommandSegment[]
  copyable?: boolean
}>(), {
  copyable: true,
})

const emit = defineEmits<{
  copied: []
}>()

const copied = ref(false)

async function copyCommand() {
  if (props.copyable === false)
    return

  try {
    await navigator.clipboard.writeText(props.command)
    copied.value = true
    emit('copied')
    setTimeout(() => {
      copied.value = false
    }, 2000)
  }
  catch {
    // Fallback for environments where clipboard API fails
    const el = document.createElement('textarea')
    el.value = props.command
    el.style.position = 'fixed'
    el.style.opacity = '0'
    document.body.appendChild(el)
    el.select()
    document.execCommand('copy')
    document.body.removeChild(el)
    copied.value = true
    emit('copied')
    setTimeout(() => {
      copied.value = false
    }, 2000)
  }
}
</script>

<template>
  <div
    class="command"
    :class="{ copied, clickable: copyable !== false }"
    @click="copyCommand"
  >
    <code>
      <template v-if="segments && segments.length > 0">
        <span
          v-for="(seg, i) in segments"
          :key="i"
          :class="seg.highlight ? 'cmd-highlight' : 'cmd-dim'"
        >{{ seg.text }}</span>
      </template>
      <template v-else>
        {{ command }}
      </template>
    </code>
    <svg
      v-if="!copied"
      class="copy-icon"
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
    >
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2" />
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" />
    </svg>
    <span v-if="copied" class="copied-indicator">copied</span>
  </div>
</template>

<style scoped>
.command {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  width: 100%;
  padding: 12px;
  background: var(--bg-weak);
  border: 1px solid var(--border);
  border-radius: 4px;
  transition: border-color 0.15s ease;
  pointer-events: auto;
}

.command.clickable {
  cursor: pointer;
}

.command.clickable:hover {
  border-color: var(--text-weak);
}

.command.copied {
  border-color: var(--accent);
}

.command code {
  flex: 1;
  min-width: 0;
  font-family: var(--font);
  font-size: 11px;
  color: var(--text);
  word-break: break-all;
  line-height: 1.5;
}

.copied-indicator {
  font-size: 10px;
  font-weight: 500;
  color: var(--accent);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.copy-icon {
  flex-shrink: 0;
  color: var(--text-weak);
  opacity: 0.5;
  transition: opacity 0.15s ease, color 0.15s ease;
  pointer-events: none;
}

.command.clickable:hover .copy-icon {
  opacity: 1;
  color: var(--accent);
}

.cmd-highlight {
  color: var(--text-strong);
  font-weight: 500;
  transition: color 0.15s ease;
}

.cmd-dim {
  color: var(--text-weak);
  transition: color 0.15s ease;
}

.command.clickable:hover .cmd-highlight,
.command.clickable:hover .cmd-dim {
  color: var(--accent);
}

.command.copied .cmd-highlight,
.command.copied .cmd-dim {
  color: var(--accent);
}
</style>
