<!-- PathMismatchWarning: Subtle hint when shortcut command path doesn't match current binary -->
<script setup lang="ts">
</script>

<template>
  <p class="path-hint">
    <span class="hint-marker">[!]</span>
    Your shortcut points to a different binary. Update it in Keyboard Settings.
  </p>
</template>

<style scoped>
.path-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0;
  font-size: 10px;
  color: var(--text-weak);
}

.hint-marker {
  color: var(--text-weak);
  opacity: 0.7;
}
</style>
