// Authentication state for the OIDC BFF flow — see docs/developer_guide/AUTHENTICATION.md.
//
// The SPA never handles an IdP token: `api_server` performs the whole Authorization Code +
// PKCE exchange itself and hands back an opaque HttpOnly session cookie, so this composable
// only ever talks to the server's own `/api/auth/*` endpoints and to `/auth/login` /
// `/auth/logout`. `isAdmin` here is purely cosmetic (hiding admin-only buttons) — the real
// enforcement happens server-side on every request.
import { ref, readonly } from 'vue';

interface AuthConfig {
  enabled: boolean;
}

interface AuthMe {
  authenticated: boolean;
  identity: string | null;
  isAdmin: boolean;
}

const enabled = ref(false);
const authenticated = ref(true);
const identity = ref<string | null>(null);
const isAdmin = ref(true);
const loaded = ref(false);

async function refresh() {
  try {
    const configRes = await fetch('/api/auth/config', { credentials: 'include' });
    const config: AuthConfig = await configRes.json();
    enabled.value = config.enabled;

    if (!config.enabled) {
      authenticated.value = true;
      isAdmin.value = true;
      identity.value = null;
      return;
    }

    const meRes = await fetch('/api/auth/me', { credentials: 'include' });
    const me: AuthMe = await meRes.json();
    authenticated.value = me.authenticated;
    identity.value = me.identity;
    isAdmin.value = me.isAdmin;
  } catch (e) {
    console.warn('[auth] failed to load authentication state', e);
  } finally {
    loaded.value = true;
  }
}

// The first `refresh()` call, shared across every caller. `redirectToLogin` (called from
// Apollo's 401 error link, possibly before any component has mounted `useAuth` itself) must
// not decide "auth is disabled, don't redirect" just because `enabled` hasn't been set yet
// — it awaits this instead of racing it. `apollo.ts` also kicks this off eagerly at module
// load, ahead of any GraphQL query, to keep that window as small as possible in practice.
let initialLoad: Promise<void> | null = null;
function ensureLoaded(): Promise<void> {
  if (!initialLoad) {
    initialLoad = refresh();
  }
  return initialLoad;
}

function login() {
  const returnTo = window.location.pathname + window.location.search;
  window.location.href = `/auth/login?return_to=${encodeURIComponent(returnTo)}`;
}

async function logout() {
  // The server may hand back the identity provider's own end-session URL instead of '/' —
  // clearing only the local Woodstock session cookie would leave the IdP's SSO session
  // alive, silently re-authenticating on the next login.
  try {
    const res = await fetch('/auth/logout', { method: 'POST', credentials: 'include' });
    const { redirect_to: redirectTo } = (await res.json()) as { redirect_to: string };
    window.location.href = redirectTo;
  } catch (e) {
    console.warn('[auth] logout request failed, redirecting locally', e);
    window.location.href = '/';
  }
}

/** Redirects to the login page — called by the Apollo/fetch 401 handlers. */
async function redirectToLogin() {
  await ensureLoaded();
  if (enabled.value) {
    login();
  }
}

export function useAuth() {
  return {
    enabled: readonly(enabled),
    authenticated: readonly(authenticated),
    identity: readonly(identity),
    isAdmin: readonly(isAdmin),
    loaded: readonly(loaded),
    refresh,
    ensureLoaded,
    login,
    logout,
    redirectToLogin,
  };
}
