#!/bin/sh
# create-freebsd-pkg.sh — Build FreeBSD .pkg packages for Woodstock Backup
#
# Usage:
#   ./scripts/create-freebsd-pkg.sh <version> <binaries_dir> <output_dir> [front_dist_dir]
#
# Arguments:
#   version        — Package version (e.g. 2.1.0)
#   binaries_dir   — Directory containing the pre-compiled FreeBSD binaries
#   output_dir     — Directory where .pkg files will be written
#   front_dist_dir — (optional) Path to the pre-compiled Vue.js frontend (front/dist)
#
# Example:
#   ./scripts/create-freebsd-pkg.sh 2.1.0 ./zipfile ./pkg-out ./front/dist

set -e

VERSION="${1:?Usage: $0 <version> <binaries_dir> <output_dir> [front_dist_dir]}"
BINARIES_DIR="${2:?Usage: $0 <version> <binaries_dir> <output_dir> [front_dist_dir]}"
OUTPUT_DIR="${3:?Usage: $0 <version> <binaries_dir> <output_dir> [front_dist_dir]}"
FRONT_DIST="${4:-}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Package ABI — must match the system the packages are built on, otherwise
# `pkg add` rejects them with "wrong architecture". Ask pkg(8) first; fall back
# to the running major release so a manual run outside a FreeBSD host with a
# configured pkg still produces something sensible. Override with ABI=... to
# cross-build for another release.
ABI="${ABI:-$(pkg config ABI 2>/dev/null || true)}"
if [ -z "${ABI}" ]; then
    ABI="freebsd:$(uname -r | cut -d. -f1):x86:64"
fi

mkdir -p "${OUTPUT_DIR}"

# ────────────────────────────────────────────────────────────────────────────
# Helper: build a single FreeBSD package
# Args: <pkg_name> <manifest_in> <staging_dir_suffix>
# ────────────────────────────────────────────────────────────────────────────
build_pkg() {
    PKG_NAME="$1"
    MANIFEST_IN="$2"
    STAGING_SUFFIX="$3"

    STAGING="${OUTPUT_DIR}/staging-${STAGING_SUFFIX}"
    rm -rf "${STAGING}"
    mkdir -p "${STAGING}"

    echo "  → Building ${PKG_NAME}-${VERSION}.pkg"

    # Read script contents for inline embedding
    if [ "${STAGING_SUFFIX}" = "client" ]; then
        SCRIPTS_DIR="${REPO_ROOT}/client-rs/freebsd"
    else
        SCRIPTS_DIR="${REPO_ROOT}/server-rs/freebsd"
    fi

    # Generate the manifest (version and ABI substitution)
    sed -e "s|@@VERSION@@|${VERSION}|g" \
        -e "s|@@ABI@@|${ABI}|g" \
        "${MANIFEST_IN}" > "${STAGING}/+MANIFEST"

    # Lifecycle scripts have to be embedded in the manifest under `scripts:`.
    # Dropping +POST_INSTALL / +PRE_DEINSTALL next to it does nothing when
    # `pkg create` is driven by --manifest: the files are simply ignored, and the
    # resulting package silently installs without ever creating config.yaml or
    # stopping the services on delete.
    _embed_script() {
        _key="$1"
        _file="$2"
        [ -f "${_file}" ] || return 0
        printf '  %s: <<EOS\n' "${_key}" >> "${STAGING}/+MANIFEST"
        cat "${_file}" >> "${STAGING}/+MANIFEST"
        printf '\nEOS\n' >> "${STAGING}/+MANIFEST"
    }

    printf '\nscripts: {\n' >> "${STAGING}/+MANIFEST"
    _embed_script "post-install"  "${SCRIPTS_DIR}/post-install.sh"
    _embed_script "pre-deinstall" "${SCRIPTS_DIR}/pre-deinstall.sh"
    printf '}\n' >> "${STAGING}/+MANIFEST"

    # pkg create reads files relative to the rootdir argument
    # We create a rootdir with the full filesystem layout expected by the manifest
    ROOTDIR="${STAGING}/root"
    mkdir -p "${ROOTDIR}"

    if [ "${STAGING_SUFFIX}" = "client" ]; then
        _install_client_files "${ROOTDIR}" "${SCRIPTS_DIR}"
    else
        _install_server_files "${ROOTDIR}" "${SCRIPTS_DIR}"
    fi

    # Dynamically generate files: section by scanning rootdir
    # This captures frontend static files and avoids maintaining a static list
    {
        printf '\nfiles: {\n'
        find "${ROOTDIR}" -type f | sort | while IFS= read -r f; do
            printf '  %s: ""\n' "${f#${ROOTDIR}}"
        done
        printf '}\n'
    } >> "${STAGING}/+MANIFEST"

    # Build the package
    pkg create \
        --format txz \
        --manifest "${STAGING}/+MANIFEST" \
        --root-dir "${ROOTDIR}" \
        --out-dir "${OUTPUT_DIR}"

    echo "  ✓ ${OUTPUT_DIR}/${PKG_NAME}-${VERSION}.pkg"
}

# ────────────────────────────────────────────────────────────────────────────
# Install client files into rootdir
# ────────────────────────────────────────────────────────────────────────────
_install_client_files() {
    ROOTDIR="$1"
    SCRIPTS_DIR="$2"

    mkdir -p "${ROOTDIR}/usr/local/bin"
    mkdir -p "${ROOTDIR}/usr/local/etc/rc.d"
    mkdir -p "${ROOTDIR}/usr/local/etc/woodstock"
    mkdir -p "${ROOTDIR}/var/db/woodstock"
    mkdir -p "${ROOTDIR}/var/log/woodstock"

    # Binaries
    install -m 755 "${BINARIES_DIR}/ws_client_daemon"   "${ROOTDIR}/usr/local/bin/ws_client_daemon"
    install -m 755 "${BINARIES_DIR}/ws_client_console"  "${ROOTDIR}/usr/local/bin/ws_client_console"

    # rc.d script
    install -m 755 "${SCRIPTS_DIR}/woodstock_client.rc" "${ROOTDIR}/usr/local/etc/rc.d/woodstock_client"

    # Sample config
    install -m 644 "${SCRIPTS_DIR}/config.yaml.sample" "${ROOTDIR}/usr/local/etc/woodstock/config.yaml.sample"
}

# ────────────────────────────────────────────────────────────────────────────
# Install server files into rootdir
# ────────────────────────────────────────────────────────────────────────────
_install_server_files() {
    ROOTDIR="$1"
    SCRIPTS_DIR="$2"

    mkdir -p "${ROOTDIR}/usr/local/bin"
    mkdir -p "${ROOTDIR}/usr/local/etc/rc.d"
    mkdir -p "${ROOTDIR}/usr/local/etc/woodstock"
    mkdir -p "${ROOTDIR}/usr/local/share/woodstock/static"
    mkdir -p "${ROOTDIR}/var/db/woodstock/certs"
    mkdir -p "${ROOTDIR}/var/db/woodstock/config"
    mkdir -p "${ROOTDIR}/var/db/woodstock/hosts"
    mkdir -p "${ROOTDIR}/var/db/woodstock/logs"
    mkdir -p "${ROOTDIR}/var/db/woodstock/pool"
    mkdir -p "${ROOTDIR}/var/db/woodstock/events"
    mkdir -p "${ROOTDIR}/var/db/woodstock/jobs"
    mkdir -p "${ROOTDIR}/var/log/woodstock"

    # Binaries
    install -m 755 "${BINARIES_DIR}/api_server"          "${ROOTDIR}/usr/local/bin/api_server"
    install -m 755 "${BINARIES_DIR}/client_api_server"   "${ROOTDIR}/usr/local/bin/client_api_server"
    install -m 755 "${BINARIES_DIR}/job_worker"          "${ROOTDIR}/usr/local/bin/job_worker"
    install -m 755 "${BINARIES_DIR}/scheduler"           "${ROOTDIR}/usr/local/bin/scheduler"

    # Admin CLI — same tools as the woodstock-cli Debian package. Without these
    # a FreeBSD server has no way to inspect manifests or run a restore.
    install -m 755 "${BINARIES_DIR}/ws_console"          "${ROOTDIR}/usr/local/bin/ws_console"
    install -m 755 "${BINARIES_DIR}/ws_restore"          "${ROOTDIR}/usr/local/bin/ws_restore"
    install -m 755 "${BINARIES_DIR}/ws_sync"             "${ROOTDIR}/usr/local/bin/ws_sync"

    # rc.d scripts
    install -m 755 "${SCRIPTS_DIR}/woodstock_api.rc"         "${ROOTDIR}/usr/local/etc/rc.d/woodstock_api"
    install -m 755 "${SCRIPTS_DIR}/woodstock_client_api.rc"  "${ROOTDIR}/usr/local/etc/rc.d/woodstock_client_api"
    install -m 755 "${SCRIPTS_DIR}/woodstock_worker.rc"      "${ROOTDIR}/usr/local/etc/rc.d/woodstock_worker"
    install -m 755 "${SCRIPTS_DIR}/woodstock_scheduler.rc"   "${ROOTDIR}/usr/local/etc/rc.d/woodstock_scheduler"

    # Sample env config
    install -m 640 "${SCRIPTS_DIR}/server.env.sample" "${ROOTDIR}/usr/local/etc/woodstock/server.env.sample"

    # Frontend static files (optional)
    if [ -n "${FRONT_DIST}" ] && [ -d "${FRONT_DIST}" ]; then
        cp -R "${FRONT_DIST}/." "${ROOTDIR}/usr/local/share/woodstock/static/"
        find "${ROOTDIR}/usr/local/share/woodstock/static" -type f -exec chmod 644 {} \;
        find "${ROOTDIR}/usr/local/share/woodstock/static" -type d -exec chmod 755 {} \;
        echo "  Frontend files copied from ${FRONT_DIST}"
    else
        echo "  WARNING: No frontend dist provided (FRONT_DIST='${FRONT_DIST}'). Static files directory will be empty."
    fi
}

# ────────────────────────────────────────────────────────────────────────────
# Main
# ────────────────────────────────────────────────────────────────────────────
echo "Building FreeBSD packages v${VERSION}"
echo "  Binaries: ${BINARIES_DIR}"
echo "  Output:   ${OUTPUT_DIR}"
echo ""

build_pkg "woodstock-client" "${REPO_ROOT}/client-rs/freebsd/+MANIFEST.in" "client"
build_pkg "woodstock-server" "${REPO_ROOT}/server-rs/freebsd/+MANIFEST.in" "server"

echo ""
echo "Done. Packages written to ${OUTPUT_DIR}/"
ls -lh "${OUTPUT_DIR}"/*.pkg 2>/dev/null || true
