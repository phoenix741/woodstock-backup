#!/usr/bin/env bash
# QEMU lifecycle: golden images, per-run overlays, boot and shutdown.
#
# Networking rationale — each VM gets two NICs:
#
#   NIC0  user-mode + hostfwd    internet during install, and SSH from the host
#   NIC1  socket multicast       a real L2 segment between the VMs
#
# The second NIC is what makes the suite possible at all: the Woodstock server
# dials each agent at https://<ip>:3657, so a hostfwd-only setup cannot work.
# Multicast sockets need no root, no bridge and no qemu-bridge-helper.

QEMU="qemu-system-x86_64"

# KVM when the host allows it, plain TCG otherwise (much slower but works).
qemu_accel_args() {
    if [[ -w /dev/kvm ]]; then
        echo "-enable-kvm -cpu host"
    else
        warn "/dev/kvm is not writable — falling back to TCG (slow)"
        echo "-cpu max"
    fi
}

# Create a copy-on-write overlay so each run starts from a pristine image and
# leaves the golden image untouched.
vm_overlay() {
    local golden="$1" overlay="$2"
    [[ -f "${golden}" ]] || die "golden image not found: ${golden} (run images/build-all.sh)"
    rm -f "${overlay}"
    qemu-img create -q -f qcow2 -F qcow2 -b "${golden}" "${overlay}" >/dev/null
}

# vm_start <role> <overlay.qcow2> [extra qemu args...]
#
# Writes the QEMU pid to $VM_DIR/<role>.pid and the serial console to
# $VM_DIR/<role>-console.log — the console is the only way to diagnose a guest
# that never reaches SSH.
vm_start() {
    local role="$1" disk="$2"; shift 2
    local port ip
    port="$(vm_port "${role}")"
    ip="$(vm_ip "${role}")"

    local hostfwd="hostfwd=tcp::${port}-:22"
    if [[ "${role}" == "server" ]]; then
        # The management API is plain HTTP on 3000; the suite drives it from the host.
        hostfwd="${hostfwd},hostfwd=tcp::${PORT_API_SERVER}-:3000"
    fi

    # Deterministic MACs keep the guest's interface naming stable across boots.
    local mac_suffix
    case "${role}" in
        server)  mac_suffix="10" ;;
        debian)  mac_suffix="20" ;;
        freebsd) mac_suffix="21" ;;
        windows) mac_suffix="22" ;;
    esac

    # Windows has no in-box virtio-net driver; e1000e is present on every guest.
    local nic_model="virtio-net-pci"
    [[ "${role}" == "windows" ]] && nic_model="e1000e"

    # The Windows guest has to be brought up the way it was installed, or it
    # does not boot at all:
    #   * UEFI, with the variable store the Windows build saved — that is
    #     where the Windows boot entry lives. The default SeaBIOS finds nothing.
    #   * the system disk on q35's AHCI controller, not virtio-blk: Windows
    #     Setup has no virtio driver, so the installed system has none either.
    #   * a real TPM: windows.pkr.hcl builds against vtpm/tpm-crb, and q35
    #     without one crash-reboot-loops partway through first boot (Setup
    #     hit this; nothing rules out the installed system hitting it too on
    #     a cold boot, so the run side provides one unconditionally).
    # The variable store and TPM state are both per-run: the firmware and the
    # TPM write to them, and nothing here depends on carrying prior state
    # forward (no BitLocker, no credential guard — nothing was ever sealed to
    # a specific TPM identity), so starting from blank each run is correct,
    # not just convenient.
    #
    # Plain OVMF, not the Secure Boot variant: the golden image is built with
    # Secure Boot disabled (autounattend.xml sets the LabConfig bypass instead
    # of enrolling MS keys — see images/packer/windows/), so its OVMF_VARS.fd
    # carries no PK/KEK/db. Booting that store under a Secure-Boot-enforcing
    # firmware does not fail loudly; it just fails to find a validated boot
    # entry. The two must always change together.
    local -a machine_args=()
    local system_drive=(-drive "file=${disk},if=virtio,format=qcow2,cache=unsafe")
    if [[ "${role}" == "windows" ]]; then
        local vars="${VM_DIR}/${role}-OVMF_VARS.fd"
        [[ -f "${vars}" ]] || cp "${IMAGES_DIR}/windows-OVMF_VARS.fd" "${vars}"

        # swtpm has to be up and listening before qemu tries to connect to
        # its control socket — started here, synchronously, ahead of the
        # qemu invocation below. -d only forks after the socket is bound.
        #
        # The control socket itself lives under /tmp, not VM_DIR: AF_UNIX
        # paths are capped at ~108 bytes (sun_path), and VM_DIR's own path
        # (repo checkout / run/<timestamp>/vms/...) already used all of it —
        # confirmed empirically ("swtpm: Path for UnioIO socket is too
        # long"). Only the socket needs to be short-pathed; the state
        # directory has no such limit and stays under VM_DIR with everything
        # else this run owns. The tmp dir's own path is recorded so vm_stop
        # can remove it — it is not otherwise reachable from VM_DIR.
        local tpm_state="${VM_DIR}/${role}-tpm-state"
        mkdir -p "${tpm_state}"
        local tpm_dir
        tpm_dir="$(mktemp -d /tmp/woodstock-e2e-tpm.XXXXXX)"
        echo "${tpm_dir}" > "${VM_DIR}/${role}-tpm-dir"
        local tpm_sock="${tpm_dir}/tpm.sock"
        swtpm socket --tpm2 \
            --tpmstate "dir=${tpm_state}" \
            --ctrl "type=unixio,path=${tpm_sock}" \
            --pid "file=${VM_DIR}/${role}-swtpm.pid" \
            -d

        machine_args=(
            -machine q35
            -drive "if=pflash,format=raw,unit=0,file=/usr/share/OVMF/OVMF_CODE_4M.fd,readonly=on"
            -drive "if=pflash,format=raw,unit=1,file=${vars}"
            -chardev "socket,id=chrtpm,path=${tpm_sock}"
            -tpmdev "emulator,id=tpm0,chardev=chrtpm"
            -device "tpm-crb,tpmdev=tpm0"
        )
        system_drive=(
            -drive "file=${disk},if=none,id=hd0,format=qcow2,cache=unsafe"
            -device ide-hd,drive=hd0,bus=ide.0,bootindex=1
        )
    fi

    log "starting VM ${role} (ssh :${port}, lab ${ip})"
    # shellcheck disable=SC2086
    ${QEMU} \
        $(qemu_accel_args) \
        "${machine_args[@]}" \
        -m "${VM_MEM}" -smp "${VM_CPUS}" \
        "${system_drive[@]}" \
        -netdev "user,id=net0,${hostfwd}" \
        -device "${nic_model},netdev=net0,mac=52:54:00:e2:e0:${mac_suffix}" \
        -netdev "socket,id=lab,mcast=${LAB_MCAST}" \
        -device "${nic_model},netdev=lab,mac=52:54:00:e2:e1:${mac_suffix}" \
        -display none \
        -serial "file:${VM_DIR}/${role}-console.log" \
        -pidfile "${VM_DIR}/${role}.pid" \
        -daemonize \
        "$@"
}

# Attach a blank data disk, created on demand. Used for the btrfs /home of the
# Debian client: btrfs snapshots the whole mount point, so the share has to live
# on its own btrfs volume.
vm_data_disk() {
    local role="$1"
    local path="${VM_DIR}/${role}-data.qcow2"
    if [[ ! -f "${path}" ]]; then
        qemu-img create -q -f qcow2 "${path}" "${DATA_DISK}" >/dev/null
    fi
    echo "-drive file=${path},if=virtio,format=qcow2,cache=unsafe"
}

vm_stop() {
    local role="$1"
    local pidfile="${VM_DIR}/${role}.pid"
    if [[ -f "${pidfile}" ]]; then
        local pid
        pid="$(cat "${pidfile}")"
        if kill -0 "${pid}" 2>/dev/null; then
            log "stopping VM ${role} (pid ${pid})"
            # Try a clean ACPI shutdown first so filesystems are flushed.
            ssh_run "${role}" "poweroff" >/dev/null 2>&1 || kill "${pid}" 2>/dev/null || true
            for _ in $(seq 1 30); do
                kill -0 "${pid}" 2>/dev/null || break
                sleep 1
            done
            kill -9 "${pid}" 2>/dev/null || true
        fi
        rm -f "${pidfile}"
    fi

    # windows only: the swtpm process backing vm_start's -tpmdev emulator
    # outlives qemu unless killed separately — it is a standalone process,
    # not a qemu child.
    local tpm_pidfile="${VM_DIR}/${role}-swtpm.pid"
    if [[ -f "${tpm_pidfile}" ]]; then
        kill "$(cat "${tpm_pidfile}")" 2>/dev/null || true
        rm -f "${tpm_pidfile}"
    fi

    # And the /tmp directory holding its control socket (see vm_start for
    # why it lives outside VM_DIR) — not cleaned up by anything else, since
    # nothing else in the run/ tree points at it.
    local tpm_dir_file="${VM_DIR}/${role}-tpm-dir"
    if [[ -f "${tpm_dir_file}" ]]; then
        rm -rf "$(cat "${tpm_dir_file}")"
        rm -f "${tpm_dir_file}"
    fi
}

vm_stop_all() {
    local role
    for role in server debian freebsd windows; do
        vm_stop "${role}"
    done
}

# Kill VMs left over by an interrupted run. They still hold the SSH and API port
# forwards, so a new run would silently talk to the old machines — or fail to
# bind at all. Matches *.pid broadly enough to also catch windows-swtpm.pid —
# without the swtpm branch here, an interrupted run leaks a swtpm daemon that
# nothing ever kills, since it is not a child of the qemu process this
# function otherwise targets.
vm_stop_stale() {
    local pidfile pid exe
    while IFS= read -r pidfile; do
        pid="$(cat "${pidfile}" 2>/dev/null || true)"
        [[ -n "${pid}" ]] || continue
        exe="$(readlink -f "/proc/${pid}/exe" 2>/dev/null)"
        if kill -0 "${pid}" 2>/dev/null && [[ "${exe}" == *qemu-system* || "${exe}" == */swtpm ]]; then
            warn "killing leftover process from a previous run (pid ${pid})"
            kill -9 "${pid}" 2>/dev/null || true
        fi
        rm -f "${pidfile}"
    done < <(find "${RUNS_DIR}" -name '*.pid' 2>/dev/null)

    # swtpm's control socket lives under /tmp, not VM_DIR (see vm_start), so
    # it survives the loop above even once the process behind it is dead.
    find /tmp -maxdepth 1 -name 'woodstock-e2e-tpm.*' -type d 2>/dev/null \
        | while IFS= read -r d; do rm -rf "${d}"; done

    # QEMU needs a moment to release the forwarded ports.
    sleep 2
}
