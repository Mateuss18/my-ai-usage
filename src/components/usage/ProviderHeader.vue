<script setup lang="ts">
withDefaults(defineProps<{
  name: string
  eyebrow: string
  logoSrc: string
  authenticating?: boolean
  authenticated?: boolean
  loginFailed?: boolean
  logoutFailed?: boolean
  logoutting?: boolean
}>(), { authenticating: false, authenticated: false, loginFailed: false, logoutFailed: false, logoutting: false })
const emit = defineEmits<{ refresh: []; switchAccount: []; cancelLogin: []; logout: [] }>()
</script>

<template>
  <header class="provider-header">
    <div class="provider-header__brand">
      <img class="provider-header__mark" :src="logoSrc" alt="" aria-hidden="true" />
      <div class="provider-header__copy">
        <p class="provider-header__eyebrow">{{ eyebrow }}</p>
        <h1 class="provider-header__name">{{ name }}</h1>
      </div>
    </div>
    <p v-if="authenticating" class="provider-header__auth-status" role="status">Waiting for login in your browser…</p>
    <p v-else-if="logoutting" class="provider-header__auth-status" role="status">Signing out…</p>
    <p v-else-if="loginFailed" class="provider-header__auth-status provider-header__auth-status--error" role="alert">Login failed. Try again.</p>
    <p v-else-if="logoutFailed" class="provider-header__auth-status provider-header__auth-status--error" role="alert">Sign out failed. Try again.</p>
    <details class="provider-header__menu">
      <summary :aria-label="`Options for ${name}`">•••</summary>
      <div class="provider-header__menu-popover">
        <button
          v-if="authenticating"
          type="button"
          aria-label="Cancel Codex sign-in"
          @click="emit('cancelLogin')"
        >Cancel sign-in</button>
        <template v-else>
          <button
            type="button"
            :aria-label="authenticated ? 'Switch Codex account' : 'Sign in to Codex'"
            :disabled="logoutting"
            @click="emit('switchAccount')"
          >{{ authenticated ? 'Switch account' : 'Sign in' }}</button>
          <button v-if="authenticated" type="button" aria-label="Sign out of Codex" :disabled="logoutting" @click="emit('logout')">Sign out</button>
        </template>
        <button type="button" :disabled="authenticating || logoutting" @click="emit('refresh')">Refresh usage</button>
      </div>
    </details>
  </header>
</template>

<style scoped>
.provider-header {
  position: relative; display: flex; align-items: center; justify-content: space-between;
  min-width: 0; padding: 9px 16px 8px; border-bottom: 1px solid #37373d;
}
.provider-header__brand { display: flex; min-width: 0; align-items: center; gap: 11px; }
.provider-header__mark {
  display: grid; width: 34px; height: 34px; flex: 0 0 auto; place-items: center;
  border: 0; border-radius: 9px; background: #151515; object-fit: cover;
}
.provider-header__copy { min-width: 0; }
.provider-header__eyebrow, .provider-header__name { margin: 0; overflow-wrap: anywhere; }
.provider-header__eyebrow { color: #9898a1; font-size: 0.68rem; font-weight: 600; letter-spacing: 0.075em; line-height: 1.2; text-transform: uppercase; }
.provider-header__name { margin-top: 2px; color: #f7f7f8; font-size: 1rem; font-weight: 600; line-height: 1.25; }
.provider-header__auth-status { margin: 0 8px 0 auto; color: #7dd3fc; font-size: 0.7rem; white-space: nowrap; }
.provider-header__auth-status--error { color: #fca5a5; }
.provider-header__menu { position: relative; flex: 0 0 auto; }
.provider-header__menu summary {
  display: grid; width: 34px; height: 34px; cursor: pointer; list-style: none; place-items: center;
  border-radius: 8px; color: #bdbdc4; letter-spacing: 0.08em; transition: background 120ms ease, color 120ms ease;
}
.provider-header__menu summary::-webkit-details-marker { display: none; }
.provider-header__menu summary:hover, .provider-header__menu[open] summary { background: #1c1c1c; color: #fff; }
.provider-header__menu summary:focus-visible, .provider-header__menu-popover button:focus-visible { outline: 2px solid #7dd3fc; outline-offset: 2px; }
.provider-header__menu-popover {
  position: absolute; z-index: 2; top: 39px; right: 0; width: max-content; padding: 4px;
  border: 1px solid #303030; border-radius: 8px; background: #121212; box-shadow: 0 10px 28px #0008;
}
.provider-header__menu-popover button { display: block; width: 100%; padding: 7px 10px; border: 0; border-radius: 5px; background: transparent; color: #f4f4f5; cursor: pointer; font-size: 0.78rem; text-align: left; }
.provider-header__menu-popover button:hover { background: #202020; }
.provider-header__menu-popover button:disabled { cursor: not-allowed; opacity: 0.5; }
@media (prefers-reduced-motion: reduce) { .provider-header__menu summary { transition: none; } }
</style>
