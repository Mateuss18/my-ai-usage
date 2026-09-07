<script setup lang="ts">
import { ref } from 'vue'

import { describeBridgeError, invokeGreeting } from './bridge'

const response = ref('')
const error = ref('')
const isLoading = ref(false)

async function runGreeting(): Promise<void> {
  isLoading.value = true
  response.value = ''
  error.value = ''

  try {
    response.value = await invokeGreeting()
  } catch (cause: unknown) {
    error.value = `Falha ao chamar Rust: ${describeBridgeError(cause)}`
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <main class="app-shell" aria-labelledby="app-title">
    <p class="eyebrow">Rewrite isolado</p>
    <h1 id="app-title">My AI Usage</h1>
    <p class="intro">Tauri 2 + Vue 3 + TypeScript</p>

    <section class="command-panel" aria-labelledby="command-title">
      <h2 id="command-title">Ponte frontend ↔ Rust</h2>
      <p>Execute o comando mínimo registrado no host Tauri.</p>
      <button type="button" :disabled="isLoading" @click="runGreeting">
        {{ isLoading ? 'Chamando Rust…' : 'Chamar comando Rust' }}
      </button>

      <p v-if="response" class="response" role="status">Resposta: {{ response }}</p>
      <p v-if="error" class="error" role="alert">{{ error }}</p>
    </section>
  </main>
</template>

<style scoped>
:global(*) {
  box-sizing: border-box;
}

:global(body) {
  margin: 0;
  min-width: 320px;
  background: #f5f7fb;
  color: #172033;
  font-family: Inter, ui-sans-serif, system-ui, sans-serif;
}

.app-shell {
  width: min(100% - 2rem, 42rem);
  margin: 0 auto;
  padding: 4rem 0;
}

.eyebrow {
  margin: 0 0 0.5rem;
  color: #526581;
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

h1,
h2,
p {
  margin-top: 0;
}

h1 {
  margin-bottom: 0.5rem;
  font-size: clamp(2rem, 8vw, 3.5rem);
  letter-spacing: -0.04em;
}

.intro {
  color: #526581;
}

.command-panel {
  margin-top: 2rem;
  padding: 1.5rem;
  border: 1px solid #dbe3f0;
  border-radius: 1rem;
  background: #fff;
  box-shadow: 0 1rem 2.5rem #17203312;
}

h2 {
  margin-bottom: 0.5rem;
  font-size: 1.15rem;
}

button {
  min-height: 2.75rem;
  padding: 0.6rem 1rem;
  border: 0;
  border-radius: 0.6rem;
  background: #315efb;
  color: #fff;
  cursor: pointer;
  font: inherit;
  font-weight: 700;
}

button:disabled {
  cursor: wait;
  opacity: 0.65;
}

button:focus-visible {
  outline: 3px solid #8ca7ff;
  outline-offset: 3px;
}

.response,
.error {
  margin: 1rem 0 0;
  padding: 0.75rem;
  border-radius: 0.5rem;
}

.response {
  background: #e9f8ef;
  color: #166534;
}

.error {
  background: #fff0f0;
  color: #b42318;
}
</style>
