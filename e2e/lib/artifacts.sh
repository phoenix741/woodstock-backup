#!/usr/bin/env bash
# Resolve the packages under test into $PKG_DIR.
#
# Two sources:
#   local  — CI artifact zips already sitting in $ARTIFACTS_DIR
#   gitea  — downloaded from a Gitea Actions run via the same API the release
#            workflow itself uses (.gitea/workflows/on-release.yml)
#
# Package file names are never hardcoded: cargo-deb rewrites semver prerelease
# separators (2.0.0-alpha.51 becomes 2.0.0~alpha.51) and the version moves with
# every release, so everything downstream globs.

# Download one artifact of a Gitea Actions run into $1.
_gitea_fetch_artifact() {
    local name="$1" dest="$2"
    [[ -n "${GITEA_TOKEN}" ]] || die "ARTIFACTS_SOURCE=gitea requires GITEA_TOKEN"
    [[ -n "${GITEA_RUN_ID}" ]] || die "ARTIFACTS_SOURCE=gitea requires GITEA_RUN_ID"

    local api="${GITEA_URL}/api/v1/repos/${GITEA_REPO}/actions/runs/${GITEA_RUN_ID}/artifacts"
    local id
    # Two matrix entries upload under the same artifact name
    # (x86_64-unknown-linux-gnu, once for the full build and once for "lite"),
    # so pick the highest id — this is what the release workflow does too.
    id="$(curl -fsSL -H "Authorization: token ${GITEA_TOKEN}" "${api}" \
          | jq -r --arg n "${name}" '[.artifacts[]? // .[]? | select(.name == $n)] | max_by(.id) | .id')"
    [[ -n "${id}" && "${id}" != "null" ]] || die "artifact '${name}' not found in run ${GITEA_RUN_ID}"

    log "downloading artifact ${name} (id ${id})"
    curl -fsSL -H "Authorization: token ${GITEA_TOKEN}" "${api}/${id}/zip" -o "${dest}"
}

# Put an artifact zip in $WORK/<name>.zip, from wherever it comes from.
_artifact_zip() {
    local name="$1" work="$2"
    local dest="${work}/${name}.zip"
    [[ -f "${dest}" ]] && { echo "${dest}"; return; }

    case "${ARTIFACTS_SOURCE}" in
        local)
            local src="${ARTIFACTS_DIR}/${name}.zip"
            [[ -f "${src}" ]] || die "missing ${src} (drop the CI artifacts there, or set ARTIFACTS_SOURCE=gitea)"
            cp "${src}" "${dest}"
            ;;
        gitea)
            _gitea_fetch_artifact "${name}" "${dest}"
            ;;
        *)
            die "unknown ARTIFACTS_SOURCE: ${ARTIFACTS_SOURCE}"
            ;;
    esac
    echo "${dest}"
}

# Populate $PKG_DIR with:
#   deb/*.deb              Debian packages
#   pkg/*.pkg              FreeBSD packages
#   windows/*.exe          Windows agent binaries
artifacts_prepare() {
    PKG_DIR="${RUN_DIR}/packages"
    local work="${RUN_DIR}/artifacts"
    mkdir -p "${PKG_DIR}"/{deb,pkg,windows} "${work}"

    log "resolving packages (source: ${ARTIFACTS_SOURCE})"

    unzip -qo "$(_artifact_zip debian-packages "${work}")"  -d "${PKG_DIR}/deb"
    unzip -qo "$(_artifact_zip freebsd-packages "${work}")" -d "${PKG_DIR}/pkg"

    # The Windows artifact is a zip inside a zip: the outer one is the Gitea
    # artifact envelope, the inner one holds the binaries.
    local outer="${work}/win-outer"
    mkdir -p "${outer}"
    unzip -qo "$(_artifact_zip x86_64-pc-windows-msvc "${work}")" -d "${outer}"
    local inner
    inner="$(find "${outer}" -name '*.zip' -print -quit)"
    [[ -n "${inner}" ]] || die "no inner zip in the Windows artifact"
    unzip -qo "${inner}" -d "${PKG_DIR}/windows"

    # Locally built packages win over the CI ones, so a fix can be validated
    # before it is pushed. Matching is by package name, not file name, since the
    # version differs between a local build and the CI artifact.
    if [[ -n "${LOCAL_DEB_DIR:-}" ]]; then
        local local_deb name
        while IFS= read -r local_deb; do
            name="$(dpkg-deb -f "${local_deb}" Package)"
            warn "overriding ${name} with the locally built $(basename "${local_deb}")"
            find "${PKG_DIR}/deb" -maxdepth 1 -name "${name}_*.deb" -delete
            cp "${local_deb}" "${PKG_DIR}/deb/"
        done < <(find "${LOCAL_DEB_DIR}" -maxdepth 1 -name '*.deb')
    fi

    local n_deb n_pkg
    n_deb="$(find "${PKG_DIR}/deb" -name '*.deb' | wc -l)"
    n_pkg="$(find "${PKG_DIR}/pkg" -name '*.pkg' | wc -l)"
    log "packages ready: ${n_deb} .deb, ${n_pkg} .pkg, agent $(ls "${PKG_DIR}/windows/ws_client_daemon.exe" 2>/dev/null && echo present || echo MISSING)"

    export PKG_DIR
}

# Path of a single package matching a glob, failing loudly on 0 or >1 matches.
# Usage: pkg_one "${PKG_DIR}/deb" 'woodstock-client_*.deb'
pkg_one() {
    local dir="$1" glob="$2"
    local -a found
    mapfile -t found < <(find "${dir}" -maxdepth 1 -name "${glob}" | sort)
    case "${#found[@]}" in
        1) echo "${found[0]}" ;;
        0) die "no package matching ${glob} in ${dir}" ;;
        *) die "ambiguous ${glob} in ${dir}: ${found[*]}" ;;
    esac
}
