#!/usr/bin/env bash
# Build every golden image the suite needs, with Packer. One-shot and cached:
# re-running is cheap, and FORCE=1 rebuilds.
#
#   ./build-all.sh                 all three platforms
#   ./build-all.sh debian          just one
#   FORCE=1 ./build-all.sh debian  rebuild from scratch
#
# Each platform's actual build recipe lives in packer/<os>/<os>.pkr.hcl; this
# script only resolves e2e.conf into PKR_VAR_* and invokes packer.
#
# Windows was stuck in the EFI Shell instead of Setup on every earlier
# attempt tonight — Packer's qemu builder only types boot_command over VNC,
# and that never landed reliably here (three timings tried; a human keypress
# and a QMP send-key both worked immediately, which pointed at VNC delivery
# itself). It turned out to be a different cause: q35 without a real TPM
# crash-reboot-loops partway through Setup even with the autounattend
# LabConfig bypass (which only skips the *check screen*). Once
# windows.pkr.hcl added a real vTPM (vtpm/tpm_device_type — native
# qemu-plugin fields, Packer manages the swtpm process itself), the native
# VNC boot_command started landing every time, unattended. See
# windows.pkr.hcl's header for the detail.

set -euo pipefail

E2E_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=../lib/common.sh
source "${E2E_DIR}/lib/common.sh"

load_config
require_tools packer
ensure_ssh_key

golden_for() {
    case "$1" in
        debian)  echo "${IMAGES_DIR}/debian-golden.qcow2"  ;;
        freebsd) echo "${IMAGES_DIR}/freebsd-golden.qcow2" ;;
        windows) echo "${IMAGES_DIR}/windows-golden.qcow2" ;;
    esac
}

targets=("$@")
if (( ${#targets[@]} == 0 )); then
    targets=(debian freebsd windows)
fi

for target in "${targets[@]}"; do
    golden="$(golden_for "${target}")"
    if [[ -f "${golden}" && "${FORCE:-0}" != "1" ]]; then
        log "${target} golden image already built (FORCE=1 to rebuild): ${golden}"
        continue
    fi

    tpl_dir="${IMAGES_DIR}/packer/${target}"
    [[ -d "${tpl_dir}" ]] || { error "no Packer template for '${target}' (${tpl_dir})"; exit 1; }

    log "building the ${target} golden image with Packer"

    export PKR_VAR_ssh_private_key_file="${SSH_KEY}"
    export PKR_VAR_ssh_public_key
    PKR_VAR_ssh_public_key="$(cat "${SSH_KEY}.pub")"

    case "${target}" in
        debian)
            export PKR_VAR_debian_iso="${DEBIAN_ISO}"
            export PKR_VAR_debian_iso_checksum="${DEBIAN_ISO_CHECKSUM:-none}"
            ;;
        freebsd)
            export PKR_VAR_freebsd_image_url="${FREEBSD_IMAGE_URL}"
            # Falls back to the upstream checksum manifest next to the image
            # itself, rather than "none": a stale/incomplete download should
            # fail the build, not silently become the next golden image.
            export PKR_VAR_freebsd_image_checksum="${FREEBSD_IMAGE_CHECKSUM:-file:${FREEBSD_IMAGE_URL%/*}/CHECKSUM.SHA256}"
            ;;
        windows)
            export PKR_VAR_windows_iso="${WINDOWS_ISO}"
            export PKR_VAR_windows_iso_checksum="${WINDOWS_ISO_CHECKSUM:-none}"
            ;;
    esac

    (
        cd "${tpl_dir}"
        packer init . >/dev/null
        packer build -force .
    )
done
