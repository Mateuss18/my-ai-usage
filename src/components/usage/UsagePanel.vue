<script setup lang="ts">
import ProviderHeader from './ProviderHeader.vue'
import UsageCard from './UsageCard.vue'
import appLogo from '../../assets/icone-my-ai-usage.svg'
import type { AccountUsage } from './usageTypes'
import type { UsageError } from '../../domain/usage'

withDefaults(defineProps<{
  accounts: AccountUsage[]
  loading: boolean
  error: UsageError | null
  authenticating?: boolean
  authenticated?: boolean
  loginFailed?: boolean
  logoutFailed?: boolean
  logoutting?: boolean
}>(), { authenticating: false, authenticated: false, loginFailed: false, logoutFailed: false, logoutting: false })
const emit = defineEmits<{ refresh: []; switchAccount: []; cancelLogin: []; logout: [] }>()
</script>

<template>
  <section class="usage-panel" :aria-busy="(loading && accounts.length === 0) || authenticating || logoutting">
    <ProviderHeader
      name="AI Usage"
      eyebrow="Codex"
      :logo-src="appLogo"
      :authenticating="authenticating"
      :authenticated="authenticated"
      :login-failed="loginFailed"
      :logout-failed="logoutFailed"
      :logoutting="logoutting"
      @refresh="emit('refresh')"
      @switch-account="emit('switchAccount')"
      @cancel-login="emit('cancelLogin')"
      @logout="emit('logout')"
    />

    <div v-if="loading && accounts.length === 0" class="usage-panel__loading" role="status">
      <span class="sr-only">Updating usage…</span>
      <div v-for="index in 2" :key="index" class="usage-panel__skeleton" aria-hidden="true">
        <span class="usage-panel__skeleton-ring"></span>
        <span class="usage-panel__skeleton-line"></span>
        <span class="usage-panel__skeleton-line usage-panel__skeleton-line--short"></span>
      </div>
    </div>

    <div v-else-if="accounts.length === 0" class="usage-panel__empty" :class="{ 'usage-panel__empty--error': error }" :role="error ? 'alert' : 'status'">
      <span class="usage-panel__empty-icon" aria-hidden="true">{{ error ? '!' : '—' }}</span>
      <p>{{ error?.message ?? 'No usage accounts found.' }}</p>
      <button v-if="error" type="button" @click="emit('refresh')">Try again</button>
    </div>

    <div v-else class="usage-panel__content">
      <p v-if="error" class="usage-panel__root-error" role="alert">{{ error.message }}</p>
      <article v-for="account in accounts" :key="account.account.key" class="usage-panel__account">
        <header class="usage-panel__account-header">
          <div class="usage-panel__account-copy">
            <h2 class="usage-panel__email">{{ account.account.email }}</h2>
            <p class="usage-panel__account-label">
              {{ account.account.accountType }}<span v-if="account.account.plan"> · {{ account.account.plan }}</span>
            </p>
          </div>
          <span class="usage-panel__account-status" role="status" :aria-label="`${account.accountStatus} account`">{{ account.accountStatus }}</span>
        </header>
        <p class="usage-panel__status" :class="`usage-panel__status--${account.usage.state}`" :role="account.usage.state === 'error' ? 'alert' : 'status'">{{ account.usage.statusLabel }}</p>
        <div class="usage-panel__grid">
          <UsageCard v-for="quota in account.usage.quotas" :key="`${account.account.key}:${quota.id}`" :quota="quota" :id-prefix="account.account.key" />
        </div>
      </article>
    </div>
  </section>
</template>

<style scoped>
.usage-panel {
  width: min(100%, 396px); max-width: 100%; overflow: hidden; border: 1px solid #252525;
  border-radius: 0; background: #101010; box-shadow: 0 20px 54px #0008, inset 0 1px #ffffff10;
  animation: panel-in 180ms cubic-bezier(0.16, 1, 0.3, 1) both;
}
.usage-panel__content { display: grid; gap: 10px; padding: 10px 12px 12px; }
.usage-panel__account { min-width: 0; border: 1px solid #252525; border-radius: 10px; background: #151515; }
.usage-panel__account-header { display: flex; min-width: 0; align-items: start; justify-content: space-between; gap: 8px; padding: 10px 10px 0; }
.usage-panel__account-copy { min-width: 0; }
.usage-panel__email, .usage-panel__account-label, .usage-panel__status, .usage-panel__root-error { margin: 0; overflow-wrap: anywhere; }
.usage-panel__email { color: #f7f7f8; font-size: 0.8rem; font-weight: 650; line-height: 1.3; }
.usage-panel__account-label { margin-top: 2px; color: #9898a1; font-size: 0.68rem; line-height: 1.3; text-transform: capitalize; }
.usage-panel__account-status { flex: 0 0 auto; color: #bae6fd; font-size: 0.66rem; font-weight: 700; letter-spacing: 0.04em; text-transform: uppercase; }
.usage-panel__status { padding: 5px 10px 0; color: #aaaab2; font-size: 0.68rem; line-height: 1.35; }
.usage-panel__status--partial, .usage-panel__status--stale { color: #fde68a; }
.usage-panel__status--error, .usage-panel__root-error { color: #fecaca; }
.usage-panel__root-error { padding: 0 2px; font-size: 0.7rem; line-height: 1.35; }
.usage-panel__grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; padding: 6px 4px 4px; }
.usage-panel__empty {
  display: grid; min-height: 268px; padding: 32px; place-items: center; align-content: center;
  gap: 12px; color: #c5c5cc; text-align: center;
}
.usage-panel__empty--error { color: #fecaca; }
.usage-panel__empty-icon { display: grid; width: 42px; height: 42px; place-items: center; border: 1px solid currentColor; border-radius: 50%; font-size: 1.15rem; opacity: 0.8; }
.usage-panel__empty p { max-width: 240px; margin: 0; overflow-wrap: anywhere; font-size: 0.84rem; line-height: 1.5; }
.usage-panel__empty button { padding: 7px 12px; border: 1px solid #3a3a3a; border-radius: 7px; background: #1b1b1b; color: #fff; cursor: pointer; }
.usage-panel__empty button:focus-visible { outline: 2px solid #7dd3fc; outline-offset: 2px; }
.usage-panel__loading { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 6px; padding: 8px 12px; }
.usage-panel__skeleton { display: grid; min-width: 0; min-height: 244px; padding: 22px 12px; place-items: center; align-content: center; gap: 14px; border: 1px solid #252525; border-radius: 12px; background: #151515; }
.usage-panel__skeleton-ring, .usage-panel__skeleton-line { background: linear-gradient(90deg, #181818 25%, #242424 50%, #181818 75%); background-size: 200% 100%; animation: shimmer 1.3s linear infinite; }
.usage-panel__skeleton-ring { width: 104px; aspect-ratio: 1; border-radius: 50%; }
.usage-panel__skeleton-line { width: 82%; height: 9px; border-radius: 999px; }
.usage-panel__skeleton-line--short { width: 56%; }
.sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0, 0, 0, 0); clip-path: inset(50%); white-space: nowrap; }
@keyframes panel-in { from { opacity: 0; transform: translateY(5px) scale(0.985); } }
@keyframes shimmer { to { background-position: -200% 0; } }
@media (max-width: 330px) { .usage-panel__grid, .usage-panel__loading { grid-template-columns: 1fr; } }
@media (prefers-reduced-motion: reduce) { .usage-panel, .usage-panel__skeleton-ring, .usage-panel__skeleton-line { animation: none; } }
</style>
