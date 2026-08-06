# End-to-end test suite

Automates [`docs/testing/packaging-test-plan.md`](../docs/testing/packaging-test-plan.md):
provisions real virtual machines with QEMU, installs Woodstock on them
**from CI-produced packages**, and replays what a user would do
— installation, certificate generation, agent enrollment, backup,
snapshot, restore.

Golden images are built with **Packer** (`images/packer/`); the
test runner is **bats-core** (`tests/*.bats`).

## Prerequisites

```bash
sudo apt install packer qemu-system-x86 qemu-utils swtpm openssh-client curl jq unzip python3 bats bats-assert bats-support
```

`swtpm`: required for Windows client (build **and** run) — a real emulated TPM 2.0,
see "Golden Images" below.

Plus write access to `/dev/kvm` (otherwise the suite falls back to software emulation,
much slower), approximately 30 GB disk space, and installation ISOs for
Debian, FreeBSD, and Windows.

## Getting Started

```bash
cp e2e.conf.example e2e.conf     # adjust ISO paths
./images/build-all.sh            # golden images, one time only (~5 min Debian/FreeBSD, ~25 min Windows)
./run.sh --server debian --clients debian
```

Packages to test are read from `artifacts/`: place CI artifact zips there
(`debian-packages.zip`, `freebsd-packages.zip`,
`x86_64-pc-windows-msvc.zip`). To download them automatically from a
Gitea run:

```bash
ARTIFACTS_SOURCE=gitea GITEA_RUN_ID=1368 GITEA_TOKEN=... ./run.sh --clients debian
```

## Options

| Command | Effect |
|---|---|
| `./run.sh --server debian --clients debian,freebsd,windows` | full suite |
| `./run.sh --server freebsd --clients debian,freebsd` | FreeBSD server, cross-interop |
| `./run.sh --only 40-backup` | replay one test file on VMs from the previous run |
| `./run.sh --keep` | leave VMs running at the end |
| `./run.sh --destructive` | add `80-upgrade` and `90-uninstall`, which uninstall |
| `./images/build-all.sh debian` | build only one image |
| `FORCE=1 ./images/build-all.sh` | rebuild all golden images |
| `E2E_DEBUG=1 ./run.sh …` | detailed traces |

## Results

Each run writes to `run/<timestamp>/`:

- `report.md` — human-readable report, one checkbox per assertion, derived from
  bats TAP (see `write_reports` in `run.sh`)
- `results.tap` — native bats-core TAP output, for continuous integration
- `console.log` — the same run as displayed on screen (see
  below), to replay the progress of a past run
- `state.json` — state shared between test files (backup uuid, etc. —
  `lib/state.sh`), not an artifact to read directly
- `logs/` — systemd journals, application logs, and VM serial consoles
- `manifest-*.txt` — `ws_console read-protobuf` dumps of backups

During execution, `run.sh` displays progress test by test (test name,
✓/✗, sliding count) — bats-core's `pretty` formatter in a terminal, `tap` otherwise
(a background run remains readable via `tail -f run/<timestamp>/console.log`).
This is a separate stream from TAP written to disk: the same execution is always
replayable by `write_reports` even if the display formatter is interrupted.

`run.sh`'s exit code is nonzero as soon as any assertion fails —
including if the suite stopped before the end (see below, "the report doesn't lie").

## How it works

### Networking

Each VM has two network interfaces:

| NIC | QEMU Type | Role |
|---|---|---|
| 0 | `user` + `hostfwd` | internet access during installation, SSH from host |
| 1 | `socket,mcast=…` | private L2 segment between VMs, fixed addresses `10.66.0.x` |

The second interface is essential: the Woodstock server **connects to**
the agent (`https://<ip>:3657`); simple port forwarding would not be enough.
QEMU multicast requires neither root, nor a bridge, nor
`qemu-bridge-helper`.

The host is not on this network; it drives VMs via SSH through port forwarding,
and the management API via `127.0.0.1:13000`.

### Golden Images

Each platform has its own Packer template under `images/packer/<os>/`.
Unattended installation is done once per platform, and the
result is cached in `images/*.qcow2`. Each `run.sh` execution
starts from a copy-on-write overlay: runs are reproducible and disposable.

| VM | Source | Mechanism |
|---|---|---|
| Debian | netinst ISO | `boot_command` typed at isolinux prompt, preseed served by Packer's built-in HTTP server (`http_content`) |
| FreeBSD | pre-installed cloud-init image | NoCloud seed (`cd_content`), `disk_size` grows the disk in-place |
| Windows | installation ISO | `autounattend.xml` + `setup.ps1` delivered via a second CD (`cd_content`), UEFI with real vTPM, **no** Secure Boot |

**Windows has a real vTPM, but no Secure Boot.** `autounattend.xml` still applies
the standard bypass (`HKLM\SYSTEM\Setup\LabConfig` —
`BypassTPMCheck`/`BypassSecureBootCheck`/`BypassRAMCheck`): it remains useful
for RAM/CPU checks, and costs nothing to keep. But the TPM itself,
testing one night proved it genuinely necessary: on `q35`, without
TPM at all, Setup crash-loops during installation — the LabConfig bypass only skips
the verification screen, not what Setup does next with TPM APIs. `vtpm = true` /
`tpm_device_type = "tpm-crb"` (native fields of the `qemu` plugin, `swtpm`
managed by Packer itself) solve the problem, and had a welcome side effect:
Packer's native `boot_command` VNC, which never reliably passed
"Press any key to boot from CD or DVD" without TPM present
(tested at various cadences, `-vga qxl`, various disk layouts — always the same EFI shell),
now passes on every build, without supervision. The exact mechanism remains unexplained beyond
"empirically, with vTPM it always works, never does not."

**`lib/qemu.sh` must stay consistent with the build, on both counts**:
it boots the Windows guest on `OVMF_CODE_4M.fd` (not the
`.secboot` variant — Secure Boot is truly absent), and now also provides
`swtpm`/`tpm-crb` at runtime, started by `vm_start` before `qemu` (the
control channel must be listening before `qemu` connects to it). TPM state
and UEFI variable store are both per-run: nothing is sealed to
a specific TPM identity (no BitLocker, no credential guard), so
starting from scratch on each run is correct, not just convenient.

### Sequence

1. `artifacts_prepare` — unpacks packages into `run/<ts>/packages/`
2. `boot_vms` — overlays, boot, wait for SSH
3. `provision` — network setup, package installation, generate
   backup data
4. `tests/*.bats`, all invoked in a single bats command (see below)
5. reports and log collection

The agent is **not** enrolled during provisioning: downloading the bundle
from `GET /api/hosts/{name}/client` is a user step that
`tests/30-certs.bats` executes and verifies.

### The Runner: bats-core

`run.sh` invokes bats **exactly once**, with the ordered and filtered list of
`tests/*.bats` files (`run_bats_suite`, in `run.sh`). Two properties follow:

- **Each `@test` is a separate process.** An unguarded nonzero return fails
  *that test*, not the whole suite — unlike the old model where
  `run.sh` would `source tests/*.sh` under `set -e` in its own process.
- **The TAP plan (`1..N`) is declared before the first test runs**, by
  counting `@test` blocks across all filtered files. `write_reports`
  compares this declared total to the number of `ok`/`not ok` lines actually seen
  in `results.tap`: if bats is killed mid-run (timeout of
  `SUITE_TIMEOUT`, or any other crash), the gap becomes an explicit
  **FAILURE** line rather than a smaller but still-green count. This is
  exactly the bug a historical `run.sh` comment documented
  ("82 PASS — no failures" for a run that died in `60-incremental` without reaching `70-restore`)
  — now impossible by construction, not just by convention.

Shared state between tests (`backup_ok`, `backup_uuid`, …) can no longer live
in bash associative arrays — each `@test` is a process that sees nothing
of what another assigned to memory. `lib/state.sh` persists it
in `${RUN_DIR}/state.json` instead; this is also what makes `--only
60-incremental` (or `70-restore`) truly usable alone, without relying
on having run `40-backup` in the same process just before.

`tests/test_helper.bash` loads `bats-support`/`bats-assert` and all
`lib/*.sh`, then calls `load_config` — sourced by each `.bats` file via
`load test_helper`. `assert_not_infra_failure` (in `test_helper.bash`)
distinguishes an SSH timeout (`lib/remote.sh`, code 124 — "VM no longer responding")
from an ordinary product failure; grep `report.md` for `TIMEOUT (infra)` to
isolate infrastructure failures from product bugs.

## Non-Obvious Points Encoded in the Suite

These behaviors trip up naive scripts; they are documented where they are worked around.

- **`qemuargs` replaces, it does not add.** Whenever a `qemuargs` entry
  shares a flag with what Packer generates itself, Packer substitutes
  **all** occurrences of that flag. This is directly why Windows has no TPM
  (see above): `-device tpm-tis` would have erased the `-device` of
  the network card.
- **`disk_image = true` (FreeBSD) must actually grow the disk**, not create
  a new one at `disk_size`: verified afterward with
  `qemu-img info` (expected virtual size 12 GiB), otherwise cloud-init's `growfs`
  has nothing to extend.
- **Neither FreeBSD seed nor `setup.ps1` (Windows) stop themselves.**
  The old bash mechanism treated QEMU's output as a completion signal;
  under Packer, `shutdown_command` must be what powers off
  the guest, and only after the SSH communicator has confirmed sshd
  responds with the harness key. An internal `power_state: poweroff` or
  `Stop-Computer` would race this check — and might win, causing the build to fail
  with "never became reachable" instead of completing cleanly. This is
  the main quality gain of the migration: the old builder validated image quality only on
  weak signals (QEMU exited, disk is >2 GiB); Packer refuses to produce
  an image if it is not reachable on the exact channel the harness will use next.
- **Hammering `boot_command` (Windows) can accidentally click Cancel.**
  The 40 space keypresses sent to get past "Press any key to boot from CD or DVD"
  continue being sent afterward — harmless most of the time, but a stray keypress
  can land on the Cancel button of the installer screen, opening "Are you sure you want
  to quit?" with "No" pre-selected — installation hangs
  until an Enter key is sent. Diagnosis: add `-qmp
  unix:.../qmp.sock,server,nowait` to `qemuargs`, `screendump` to see
  the screen, then `send-key` with `{"type": "qcode", "data": "ret"}` to
  unstick. Seen once total across an entire night of builds.
- **Host certificates are created lazily.** Only
  `GET /api/hosts/{name}/client` calls `generate_host_certificate`. Running a
  backup before this call fails at gRPC connect.
- **Configuration is cached 24 hours in Redis.** Any change to
  `hosts.yml` or `<hostname>.yml` must be followed by
  `POST /api/server/cache/clear`.
- **`POST /backups` is deduplicated for 30 s** and responds `202` without creating a
  job. The suite tolerates `202` and waits out the window before the second backup.
- **UUID identifies a backup**, not the sequential number: it names the
  directory on disk and the routes `/backups/{id}/…`. The number is just a
  display label.
- **`ws_restore` wants a `SocketAddr`**: the address must include the port.
- **Generated `config.yaml` derives its URL from the request `Host:` header**.
  The suite sends it explicitly, otherwise the agent would receive the port-forward
  address.
- **btrfs snapshots the mountpoint**, not a subdirectory: the backup share
  must thus be a complete btrfs volume — hence the extra disk
  mounted on `/home`.
- **`xattr` and `acl` default to `false`** and the enrollment bundle does not
  enable them: the suite adds them to `config.yaml` after deployment.
- **Retention purges the previous backup** when two backups fall
  in the same hour — counting backups does not work, the suite
  compares UUIDs.
- **A skipped backup is still marked `completed`**, with zero
  files: the suite checks `fileCount`, not just the status.
- **`ws_console read-protobuf` wants an absolute path**, and the backup
  directory is named by UUID (not by number).
- **`BACKUP_PATH` defaults to `/var/lib/woodstock` on all
  platforms**, including FreeBSD where data lives in `/var/db/woodstock`.
  Services receive it from `server.env`, but a command run via SSH does not
  read that file: every manual call to `ws_console`/`ws_restore` must
  pass the variable (`ws_cli`, `lib/woodstock.sh`).
- **`pkg add` does not resolve dependencies**, it stops on
  `Missing dependency 'valkey'`. The server package thus installs with
  `pkg install` on the local file, after a `pkg update` that primes the
  catalog — it is also the only way to verify the dependency actually resolves
  on its own.
- **`pgrep -f <pattern>` finds the shell sshd just spawned**, whose argv
  contains the pattern: an assertion "this process is no longer running" can never
  pass. Patterns are written `'[w]s_client_daemon'` for this reason.
- **`pkg delete` reclaims empty directories it declared**: a FreeBSD client does lose
  `/var/db/woodstock`. What survives — and what truly distinguishes
  `pkg delete` from `apt purge` — are files created after
  installation: `config.yaml` and enrollment certificates.
- **There is no Windows package**: CI delivers a bare `ws_client_daemon.exe`,
  and the agent registers itself as a service via its
  `install-service` subcommand. This reads `config.yaml` **before** processing the
  subcommand — so a file must already exist — and writes
  `--config-dir` into the service command line. Without this `--config-dir`,
  the service would run as LocalSystem and look for config in
  `%APPDATA%\woodstock` of the system profile.
- **Windows client declares two shares.** `NTUSER.DAT` is held open
  by the interactive session: finding it in the manifest is the
  strongest proof that VSS actually worked. But it lives in
  `C:\Users`, not the data share — hence `client_shares`, which adds
  this second share for Windows only.
- **The lab NIC doesn't exist yet when `setup.ps1` runs under Packer.**
  This script runs once, at first logon, *during the build* — and
  Packer's build VM has only the one card it manages for its own
  communicator. The second card, the lab network one, is attached only at
  *run time*, by `vm_start` (`lib/qemu.sh`). A MAC lookup in
  `setup.ps1` finds nothing (each build transcript logs
  `WARNING: no lab NIC found`), and its static IP is never set
  at that point — this was true for `images/build-windows.sh` (its own
  qemu invocation included both cards from the build) but not for
  Packer. Config thus lives in `provision/client-windows.ps1`
  instead: this script runs on *every* run, when the card actually exists.
  
  Second gotcha once you are in the right place: disable DHCP *before* setting
  the static IP, not after. Left enabled (the default), the DHCP client
  keeps retrying in the background on this interface — there is no
  DHCP server on the lab multicast segment, so it eventually
  falls back to an APIPA address 169.254.x.x and never installs the
  on-link route to the `/24` of the static IP. Confirmed empirically:
  `Get-NetRoute` showed only broadcast/multicast/APIPA routes on
  this interface, no 10.66.0.0/24 route, and the agent registration request
  left via the *other* NIC via its default route (source
  10.0.2.15, next hop 10.0.2.2) — silent wrong destination, not a
  timeout. `Set-NetIPInterface -Dhcp Disabled` before `New-NetIPAddress`
  fixes it.
- **Windows guest clock is wrong at startup.** QEMU presents an RTC
  as UTC, and Windows reads that value as local time: its UTC
  ends up offset. All tokens issued by the server appear expired
  (`Failed to authenticate: ExpiredSignature`), and since the agent retries
  authentication *before* listening, the visible symptom is "nothing listening on
  3657". `setup.ps1` fixes it with `Set-TimeZone -Id 'UTC'` at first logon.
- **PowerShell 5.1 reads `.ps1` files as ANSI unless they have a UTF-8 BOM**, and
  writes a BOM with `Set-Content -Encoding utf8`. Both matter: the
  generated `config.yaml` has none (otherwise the agent rejects it with a message pointing to
  the wrong field).
- **Windows output lines end in CRLF.** `ssh_run` strips carriage returns for the
  windows role, otherwise any exact comparison fails with
  "expected 'Automatic', got 'Automatic'".
- **`ws_client_daemon.exe` does not start on a bare Windows** system: it imports
  `VCRUNTIME140.dll`, so the Visual C++ 2015-2022 redistributable is required.
  Without it, any invocation dies with `0xC0000135` with no message.

## Status

| Phase | Content | Status |
|---|---|---|
| 1 | Foundation, Debian image, Debian server and client, tests `00`→`70` | **done** |
| 2 | FreeBSD (server and client) | **done** |
| 3 | Windows (client, VSS, `NTUSER.DAT`) | **done** |
| 4 | Cross-matrix, `80-upgrade`, `90-uninstall` | **done** (Windows excluded from `80`/`90` — no package) |
| 5 | Golden images migrated to Packer (`images/packer/`) | **done for all three platforms** — Windows required a real vTPM (`vtpm`/`tpm_device_type`, native fields of the plugin) for `boot_command` to reliably pass; `lib/qemu.sh` provides the same TPM at runtime (see "Golden Images" above) |
| 6 | Runner migrated to bats-core (`tests/*.bats`) | **done** |

Matrix SKIPs are known limitations, not gaps:
no FreeBSD snapshot backend (only `btrfs.rs` and `vss.rs` exist), UFS
not mounted with `acls` option on the cloud-init image, xattr/ACL absent from
NTFS, and the `dpkg` merge prompt would require a second hand-built package.

FreeBSD server does not install with `pkg add` but with `pkg install` on
the local file: see the previous section.
