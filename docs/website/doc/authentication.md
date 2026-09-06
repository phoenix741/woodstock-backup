# Authentication

By default, _Woodstock backup_'s web interface and API are open: anyone who can reach the server can see every
host, every backup, and trigger maintenance operations. For a deployment reachable by more than one trusted person
(or exposed beyond your own machine), you can turn on login via an external identity provider.

_Woodstock backup_ does not implement its own login system. It delegates authentication to any
[OpenID Connect](https://openid.net/connect/) (OIDC) provider — [Keycloak](https://www.keycloak.org/),
Azure AD / Entra ID, Authentik, or anything else that speaks the standard. Signing in, checking passwords,
multi-factor authentication, etc. is entirely the identity provider's job; the server only asks it "who is this,
and what role do they have?".

Authentication is **optional** and **disabled by default** — enabling it does not change anything for an existing
deployment until you set `OIDC_ENABLED=true`.

## Roles and host ownership

Once enabled, every signed-in person is one of:

- **admin** — sees and manages everything: every host, every backup, every event, and maintenance operations
  (pool cleanup, fsck, cache clearing, log access).
- **user** — sees and manages only the hosts they own. The *Pool* and *Archive* sections of the web interface
  (maintenance and multi-host archiving, both admin-only) aren't shown in their menu at all.

Ownership is a list of identities on each host's configuration file (see [Configuration](/doc/configuration)):

```yaml
owners:
  - alice
  - bob
```

- A host can have several owners.
- A host with no `owners` listed (the default, including every host configured before this feature existed) is
  visible to admins only — not a security regression when you turn OIDC on.
- The identity compared against `owners` is read from a claim in the token the identity provider issues (by
  default `preferred_username`); it's compared case-insensitively.
- There is no screen to manage `owners` yet — it's edited by hand in the host's YAML file, like the rest of its
  configuration. As with any configuration change, [refresh the cache](/doc/configuration#refresh-cache)
  afterward so it takes effect immediately instead of waiting for the daily cache expiry.

## Enabling it

Set these environment variables on the `server-api` process (see [Configuration](/doc/configuration) for how
environment variables are supplied in your deployment):

| Variable                    | Default                 | Description                                                                                     |
| ---------------------------- | ------------------------ | ------------------------------------------------------------------------------------------------ |
| `OIDC_ENABLED`               | `false`                  | Turns the whole feature on. Leave unset for an open, login-free deployment.                      |
| `OIDC_ISSUER_URL`             |                          | Base URL of your identity provider's realm/tenant (its `.well-known/openid-configuration` must resolve under it). |
| `OIDC_CLIENT_ID`              |                          | Client ID registered on the identity provider for Woodstock Backup.                              |
| `OIDC_CLIENT_SECRET`          | _(unset)_                | Client secret, only if the client is registered as confidential. Leave unset for a public client (PKCE only). |
| `OIDC_REDIRECT_BASE_URL`      |                          | The URL your browser uses to reach Woodstock Backup (e.g. `https://backup.example.com`). Must match a redirect URI allowed on the identity provider. |
| `OIDC_ADMIN_ROLE_CLAIM_PATH`  | `realm_access.roles`     | Dotted path to the claim holding role information in the token (handles both nested objects, e.g. Keycloak's `realm_access.roles`, and flat arrays like a plain `groups` claim). |
| `OIDC_ADMIN_ROLE_VALUES`      | `admin`                  | Comma-separated list of values in that claim that grant the admin role. Anyone without one of these values is a restricted user. |
| `OIDC_IDENTITY_CLAIM`         | `preferred_username`     | Claim used as the identity to match against a host's `owners` list.                              |
| `OIDC_SESSION_TTL_SECS`       | `28800` (8h)             | How long a login stays valid before you need to sign in again. Also the upper bound on how long revoking someone's admin role on the identity provider takes to apply — the admin/non-admin decision is made at login time, not re-checked on every request. |
| `OIDC_COOKIE_NAME`            | `woodstock_session`      | Name of the session cookie.                                                                       |
| `OIDC_COOKIE_SECURE`          | `true`                   | Marks the session cookie `Secure` (HTTPS only). Set to `false` only for plain-HTTP local testing — never over a real network. |

Once `OIDC_ENABLED=true`, the web interface shows a *Login* button; signing out is available from the same menu
(click your username). Logging out ends your Woodstock Backup session **and**, if the identity provider supports
it, its own single sign-on session — so the next login asks you to sign in again instead of silently letting you
back in.

## How it works, briefly

Woodstock Backup never hands your browser an identity-provider token. It uses the standard
[Authorization Code flow with PKCE](https://oauth.net/2/pkce/): the server itself talks to the identity provider,
and your browser only ever receives an opaque session cookie for the Woodstock Backup site. All the security-
sensitive work — checking token signatures, expiry, and issuer — is delegated to a widely-used OIDC client
library rather than done by hand, the same way you'd expect any serious piece of software to handle logins it
didn't invent itself.

## Keycloak example

[Keycloak](https://www.keycloak.org/) is a free, self-hosted identity provider. This section sets one up for
real use (not a disposable local instance) and points Woodstock Backup at it.

### Running Keycloak

You need a Keycloak instance reachable by both the Woodstock Backup server and your users' browsers, over
HTTPS. Keycloak's own [server guide](https://www.keycloak.org/server/) is the reference for a proper setup
(hostname, TLS, database, clustering); the short version if you're running it with Docker:

```yaml
services:
  keycloak:
    image: quay.io/keycloak/keycloak:26.0
    command: start --hostname=https://auth.example.com --proxy-headers=xforwarded
    environment:
      - KC_BOOTSTRAP_ADMIN_USERNAME=admin
      - KC_BOOTSTRAP_ADMIN_PASSWORD=${KEYCLOAK_ADMIN_PASSWORD}
      - KC_DB=postgres
      - KC_DB_URL=jdbc:postgresql://keycloak-db:5432/keycloak
      - KC_DB_USERNAME=keycloak
      - KC_DB_PASSWORD=${KEYCLOAK_DB_PASSWORD}
    ports:
      - "8443:8443"
    depends_on:
      - keycloak-db
```

A few things that matter here, and why:

- **`start`, not `start-dev`.** Dev mode keeps everything in memory/ephemeral storage (every realm, user, and
  role is lost on restart) and turns off protections a server reachable by real users needs.
- **A real database** (Postgres above) survives container restarts and recreations. Dev mode's storage does not.
- **`KEYCLOAK_ADMIN_PASSWORD`/`KEYCLOAK_DB_PASSWORD`** come from your own secret store or `.env` file — never a
  password written into a file you'd commit. Set them once, keep them out of version control.
- **`--hostname`** must be the URL your browser and the Woodstock Backup server will actually reach, over TLS —
  terminate it with a reverse proxy (`--proxy-headers=xforwarded` above assumes one) or Keycloak's own
  certificate options if you're not fronting it with one.

### Configure the realm, client, and admin role

With Keycloak running, do this once in its admin console:

1. **Create a realm** (or reuse an existing one) dedicated to your organization.
2. **Create a client** for Woodstock Backup: enable the **Standard flow**, set **Valid redirect URIs** to
   `<OIDC_REDIRECT_BASE_URL>/auth/callback` (e.g. `https://backup.example.com/auth/callback`), and set
   **Valid post logout redirect URIs** to `<OIDC_REDIRECT_BASE_URL>/` (e.g. `https://backup.example.com/`) —
   this is a separate field, defaulting to `+` ("same as Valid redirect URIs"), which won't match the site's
   root and would leave logout unable to send you back. Set **Web origins** to your frontend's origin if it
   differs. Leave **Client authentication** off for a public client (PKCE only, no `OIDC_CLIENT_SECRET`
   needed), or turn it on and copy the generated secret into `OIDC_CLIENT_SECRET` for a confidential client.
3. **Create a realm role** named `admin` (Realm roles → Create role).
4. **Create accounts** for your users (Users → Add user, then set a password under the **Credentials** tab), and
   assign the `admin` role to whoever should administer Woodstock Backup (Users → that user → Role mapping →
   Assign role — pick **Filter by realm roles**, not "Filter by clients", or `admin` won't show up in the list).
   Anyone without that role is a restricted "user" — see [Roles and host ownership](#roles-and-host-ownership)
   above for granting them access to specific hosts.
5. **If you assigned the role but you're still not treated as admin, check this first**: Keycloak's built-in
   `roles` client scope adds realm roles to the **access token** by default, but not always to the **ID
   token** — and Woodstock Backup only ever reads the ID token. Go to **Client scopes → roles → Mappers →
   realm roles** and make sure **"Add to ID token"** is switched on (not just "Add to access token"). This
   `roles` scope is normally shared by every client in the realm, so the change applies realm-wide; if that's
   not what you want, add the same mapper on your client's own dedicated scope instead (**Clients → your
   client → Client scopes → `<client-id>-dedicated` → Add mapper → By configuration → User Realm Role**). If
   you were already logged in when you fix this, log out and back in — the role you have is decided once, at
   login.
6. Check that the `admin` role actually shows up in the claim `OIDC_ADMIN_ROLE_CLAIM_PATH` points at — Keycloak's
   default is `realm_access.roles`; other identity providers often use a flat `groups` claim instead, in which
   case set `OIDC_ADMIN_ROLE_CLAIM_PATH=groups`.
7. Confirm which user attribute is stable and human-readable enough to type into `owners:` lists — the default,
   `preferred_username`, works well for most setups.
