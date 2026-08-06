# Testing Plan — Debian & FreeBSD Packages (client + server)

Manual validation protocol for `.deb` and `.pkg` packages generated for
Woodstock Backup, following the packaging audit and fixes of August 1, 2026
(commit `9f0bc06`). Execute on clean VMs (never previously tested) for each platform.

Check `[x]`, note `PASS`/`FAIL`, and the tested version in each section.
Any `FAIL` entry must be accompanied by a note (log, command, output).

## This Protocol is Automated

The [`e2e/`](../../e2e/) suite replays this plan on real QEMU VMs, using packages
produced by CI. It does not replace this document — it executes most of its test
cases, and this table indicates which:

| § | E2E File | Coverage |
|---|---|---|
| 1 Installation | `tests/00-install.sh` | automated (Debian + FreeBSD, client and server) |
| 2 Post-installation | `tests/00-install.sh` | automated, **except** §2.1 UID/GID 565 collision (requires manually prepared VM) |
| 3 Service Management | `tests/10-service.sh` | automated, **except** full VM reboot and restart via `daemon -r` after `kill -9` |
| 4 Configuration | `tests/20-config.sh` | automated (hosts.yml + `MANAGEMENT_API_PORT`) ; ACL/xattr warning on FreeBSD client not asserted |
| 5 Upgrade | `tests/80-upgrade.sh` | automated via package reinstallation; dpkg merge prompt remains manual (requires second hand-built package) |
| 6 Uninstallation | `tests/90-uninstall.sh` | automated (remove/purge Debian, `pkg delete` FreeBSD) |
| 7 End-to-end | `tests/30-certs.sh` → `tests/70-restore.sh` | automated, plus points not covered by this plan: lazy PKI, btrfs snapshot, deduplication, restore |
| — | `tests/50-snapshot.sh` | outside plan: snapshot backends (btrfs, VSS, FreeBSD absence) |

Run: `cd e2e && ./run.sh --server debian --clients debian,freebsd --destructive`
(`--destructive` adds §5 and §6, which uninstall). The test cases remaining
manual above are the only ones to check by hand.

## 0. VM Matrix and Priority

| # | Server VM | Client VM | Purpose |
|---|---|---|---|
| 1 | Debian | Debian | Baseline — must not break existing functionality |
| 2 | FreeBSD | FreeBSD | New — validates entire FreeBSD chain end-to-end |
| 3 | Debian | FreeBSD | Cross-platform interop (most likely real-world use case) |
| 4 | FreeBSD | Debian | Cross-platform interop, reverse direction |

Recommendation: perform #1 and #2 first (validate each platform in isolation),
then #3/#4 if time permits — since gRPC/mTLS protocol is identical on both sides,
interop has little reason to fail if #1 and #2 pass, but it is worth verifying
at least once.

Versions to note before starting:
- Debian: `_______` (e.g., Bookworm 12)
- FreeBSD: `_______` (e.g., 14.2-RELEASE — consistent with `arch:
  freebsd:14:x86:64` in manifests `+MANIFEST.in`)

---

## 1. Installation

### 1.1 Debian (`apt install ./woodstock-*.deb` or Gitea repository)

- [ ] `apt install` resolves dependencies (`ca-certificates`, and for server
      `valkey | redis-server`) without manual intervention
- [ ] No dpkg errors/warnings during `postinst`
- [ ] `dpkg -s woodstock-client` / `dpkg -s woodstock-server` : `Maintainer:`
      field properly formatted as `Ulrich Vandenhekke <ulrich.vdh@gmail.com>`
      *(regression test: bug identified August 1, 2026, `authors` missing
      email in `Cargo.toml`)*
- [ ] `dpkg -L woodstock-client` lists `/usr/bin/ws_client_daemon`,
      `/etc/woodstock/config.yaml`
- [ ] `dpkg -L woodstock-server` lists all 4 server binaries +
      `/etc/woodstock/server.env` + frontend static files under
      `/usr/share/woodstock/static/`

### 1.2 FreeBSD (`pkg add woodstock-*.pkg` or repository)

- [ ] `pkg install` (or `pkg add ./woodstock-*.pkg`) installs without errors
- [ ] For server: the `valkey` dependency (`databases/valkey`) is
      resolved/installed automatically
- [ ] `pkg info woodstock-client` / `pkg info woodstock-server` : `Maintainer`
      = `ulrich.vdh@gmail.com`, `WWW` = `https://woodstock.shadoware.org/`
- [ ] `pkg info -l woodstock-client` properly lists
      `/usr/local/bin/ws_client_daemon`, `/usr/local/etc/rc.d/woodstock_client`,
      `/usr/local/etc/woodstock/config.yaml.sample`
- [ ] The `+POST_INSTALL` script executes without errors (see `pkg install` output,
      must display message "Woodstock Client/Server installed successfully")

---

## 2. Post-Installation Verification (users, permissions)

### 2.1 Server — must run as non-privileged user on both OSes

- [ ] Debian: `getent passwd woodstock` exists, shell `/usr/sbin/nologin`,
      no home directory created (`--no-create-home`)
- [ ] Debian: `ls -ld /var/lib/woodstock` → `woodstock:woodstock`, `750`
- [ ] Debian: `ls -l /etc/woodstock/server.env` → `root:woodstock`, `640`
- [ ] FreeBSD: `pw usershow woodstock` exists, `-s /usr/sbin/nologin`
- [ ] FreeBSD: `ls -ld /var/db/woodstock` → `woodstock:woodstock`, `750`
- [ ] FreeBSD: `ls -l /usr/local/etc/woodstock/server.env` → `root:woodstock`, `640`
- [ ] **FreeBSD, UID/GID collision case** *(regression test for UID fallback fix)*:
      on a dedicated test VM, pre-create a group with `pw groupadd collision -g 565`
      then install the server package → installation must **not** fail, and
      `pw groupshow woodstock` must show a GID different from 565 (auto-allocated)

### 2.2 Client — must run as root on both OSes *(change as of August 1, 2026)*

- [ ] Debian: **no** `woodstock` user is created by the client package (it has
      no `postinst`); `ls -l /etc/woodstock/config.yaml` → `root:root`, `600`
- [ ] FreeBSD: **no** `woodstock` user is created by the client package (removed
      from `post-install.sh` on August 1, 2026); `ls -l
      /usr/local/etc/woodstock/config.yaml` → `root:wheel`, `600`
- [ ] FreeBSD: if the **server** package is also installed on the same VM,
      verify that its `woodstock` user still exists normally (user creation
      removal applies only to the client package)

---

## 3. Service Management

### 3.1 Debian (systemd)

- [ ] `systemctl status woodstock-client` → active after `apt install`
      (no `User=`, runs as root — verify with
      `systemctl show woodstock-client -p User` → empty)
- [ ] `systemctl status woodstock-api/-client-api/-worker/-scheduler` →
      all active, `User=woodstock`
- [ ] `systemctl show woodstock-api -p Documentation` →
      `https://woodstock.shadoware.org/doc/` *(regression test for URL fix)*
- [ ] `systemctl status woodstock.target` → active, properly groups 4 units
      (`systemctl list-dependencies woodstock.target`)
- [ ] `systemctl stop woodstock-api && systemctl start woodstock-api` : restarts cleanly
- [ ] `journalctl -u woodstock-client -n 50` : no startup errors

### 3.2 FreeBSD (rc.d) — **critical area, test thoroughly**

- [ ] `sysrc woodstock_client_enable=YES` then `service woodstock_client start`
      → returns immediately without blocking (no foreground hang)
- [ ] `service woodstock_client status` → correctly reports "is running as pid
      NNNNN" with valid PID *(critical regression test for pidfile fix —
      before the fix, `ws_client_daemon` wrote no pidfile while the
      script declared one; this is THE test validating `daemon(8) -P` wrapper)*
- [ ] `cat /var/run/woodstock_client.pid` contains a PID, and
      `ps -p $(cat /var/run/woodstock_client.pid) -o command=` correctly points to
      `/usr/local/bin/ws_client_daemon`
- [ ] `ps aux | grep ws_client_daemon` → runs as **root** user
      *(regression test for privilege model change)*
- [ ] `service woodstock_client stop` → process actually disappears
      (`pgrep ws_client_daemon` returns nothing), no orphaned process left behind
- [ ] Kill process manually (`kill -9 <pid>`) then wait: `daemon(8) -r` wrapper
      must auto-restart it (equivalent to systemd `Restart=on-failure`)
- [ ] `cat /var/log/woodstock/client.log` contains daemon logs
      *(the `-o` field of `daemon(8)`, variable `woodstock_client_logfile`
      which was declared but unused before the fix)*
- [ ] Same for 4 server services (`woodstock_api`, `woodstock_client_api`,
      `woodstock_worker`, `woodstock_scheduler`) : `service <name> start/stop/status`,
      `sysrc <name>_enable=YES`, verify startup order at boot
      (`REQUIRE: ... woodstock_worker` for the other 3 — reboot VM
      and verify `woodstock_worker` starts before the others)
- [ ] Full FreeBSD VM reboot → all enabled services auto-restart in correct order

---

## 4. Configuration

- [ ] Edit `config.yaml` (client) with real hostname/password →
      restart service → config is properly applied
- [ ] Edit `server.env` (server): change `MANAGEMENT_API_PORT` →
      restart → server properly listens on new port
- [ ] FreeBSD client: enable `acl: true` in `config.yaml` → on startup,
      warning appears in logs indicating ACLs are not supported on this platform
      *(behavior added in `client-rs/src/server.rs`, commit `2945f77`)*
- [ ] FreeBSD client: enable `xattr: true` → **no** warning (unlike ACL,
      xattr is now natively supported on FreeBSD, provided the tested binary
      was built with `xattr` feature — verify package date is after CI fix)

---

## 5. Package Upgrade — validation of conf-files fix

This test specifically targets the bug fixed on August 1, 2026
(`conf-files` for Debian client pointed to `config.yml` instead of
`config.yaml`, so dpkg never protected the actual file).

- [ ] Install a version of the Debian client package, edit
      `/etc/woodstock/config.yaml` (e.g., change `backup_timeout`)
- [ ] Install a newer version of the same package (`apt install
      ./woodstock-client_<new-version>.deb`)
- [ ] Verify that the local modification is **preserved** (not silently overwritten)
      — if the default file also changed between versions, dpkg must present
      a merge/conflict prompt rather than overwriting without asking
- [ ] On FreeBSD, same test with `config.yaml` vs `config.yaml.sample`:
      reinstalling/upgrading the package must not touch the already-customized
      `config.yaml` (the `post-install.sh` regenerates it only if absent)

---

## 6. Uninstallation

### 6.1 Debian

- [ ] `apt remove woodstock-server` : services stopped, but
      `/var/lib/woodstock` and `/etc/woodstock` **preserved**
- [ ] `apt purge woodstock-server` : `/var/lib/woodstock`, `/etc/woodstock`
      removed, `woodstock` user/group removed
      (`getent passwd woodstock` → empty)
- [ ] Same for `woodstock-client` (remove/purge — no user to remove since
      it creates none)

### 6.2 FreeBSD

- [ ] `pkg delete woodstock-server` : `+PRE_DEINSTALL` properly stops 4
      services before file removal (verify no services still running after:
      `service woodstock_api status`)
- [ ] `pkg delete woodstock-client` : `+PRE_DEINSTALL` properly stops
      `woodstock_client` (depends on pidfile fix — if `service
      woodstock_client status` cannot find the process, `pkg delete`
      could leave the daemon running in background while files are removed)
- [ ] Note: unlike Debian, `pkg delete` does **not** remove
      `/var/db/woodstock`, `/var/log/woodstock`, or the server's
      `woodstock` user/group — this is FreeBSD standard (no `purge` equivalent),
      should be documented for end users if not already in
      `docs/website/doc/installation-freebsd.md`

---

## 7. End-to-End Functional Test

Perform at least once per combination from matrix §0 (prioritize #1 and #2):

1. [ ] Configure server (host declared in `hosts.yml`, mTLS certificates
       generated/deployed)
2. [ ] Start client, verify it appears reachable on server side (heartbeat via
       `client_api_server`)
3. [ ] Run full backup of a share containing:
       - files with ACLs (Linux only)
       - files with `user.*` xattr (Linux and FreeBSD)
       - symbolic links, special files if relevant
4. [ ] Verify generated manifest (`ws_console read-protobuf
       hosts/<host>/<n>/%2Fshare.manifest file-manifest`) contains
       expected metadata
5. [ ] Run restore (`ws_restore`) to test directory, verify content, xattr,
       and (on Linux) ACLs are restored identically
6. [ ] Run second incremental backup, verify deduplication (`statistics.yml`)

---

## 8. Summary of Fixes from August 1, 2026

This section serves as a quick checklist to confirm each bug identified in
the audit is fixed by the tested packages:

| Fix | Validated By | Status |
|---|---|---|
| Missing maintainer email | §1.1, §1.2 | ☐ |
| `conf-files` config.yml/.yaml | §5 | ☐ |
| `xattr` feature missing from FreeBSD CI build | §4 (no xattr warning) | ☐ |
| Obsolete Documentation= URL | §3.1 | ☐ |
| FreeBSD client non-root → root | §2.2, §3.2 | ☐ |
| FreeBSD client pidfile never written | §3.2 | ☐ |
| UID/GID 565 without fallback | §2.1 | ☐ |

---

*Document created August 1, 2026 following Debian/FreeBSD packaging audit.
Update if new packaging fixes are implemented.*
