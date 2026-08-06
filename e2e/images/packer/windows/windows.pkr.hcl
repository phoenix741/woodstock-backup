# Golden image for the Windows client of the Woodstock e2e suite.
#
# Windows 11 normally requires TPM 2.0 and Secure Boot. Secure Boot is not
# provided — cd/autounattend.xml sets the LabConfig bypass keys instead
# (BypassTPMCheck / BypassSecureBootCheck / BypassRAMCheck) — but TPM *is*,
# via a real vTPM. That is not redundant with the bypass: LabConfig only
# skips Setup's hardware-compatibility *check screen*. Without a TPM present
# on q35 at all, Setup itself crash-reboot-loops partway through installation
# — confirmed empirically (q35 + LabConfig bypass + no TPM: reliable crash
# loop; q35 + vtpm, or plain i440fx with no TPM device to probe at all:
# neither crashes). The bypass stays anyway, since RAM/CPU checks are still
# worth skipping and a real TPM does nothing for those.
#
# vtpm/tpm_device_type are native qemu-plugin fields — Packer manages the
# swtpm process itself, so this does not hit the qemuargs-replaces-not-append
# trap that ruled out a hand-rolled `-tpmdev`/`-device tpm-tis` (any qemuargs
# entry sharing a flag with what Packer generates replaces *every*
# occurrence of that flag, silently dropping the NIC's own `-device` along
# with it — still true, just not relevant to vtpm).
#
# ⚠️ lib/qemu.sh has to match: it boots the golden image on plain
# OVMF_CODE_4M.fd (not the .secboot variant — Secure Boot really is off) and
# now also needs to provide the same vtpm/tpm-crb device the image was
# installed against.

packer {
  required_version = ">= 1.9.0"
  required_plugins {
    qemu = {
      version = ">= 1.0.9"
      source  = "github.com/hashicorp/qemu"
    }
  }
}

variable "windows_iso" {
  type        = string
  description = "Path to the Windows 11 installation ISO. Fournir via PKR_VAR_windows_iso."
}

variable "windows_iso_checksum" {
  type        = string
  default     = "none"
  description = "sha256:<hex> of windows_iso. Fournir via PKR_VAR_windows_iso_checksum."
}

variable "ssh_private_key_file" {
  type        = string
  description = "Harness SSH private key (images/cache/id_e2e) — never Packer's ephemeral key, lib/remote.sh depends on this exact file."
}

variable "ssh_public_key" {
  type        = string
  description = "Contents of images/cache/id_e2e.pub, embedded into setup.ps1 by templatefile()."
}

variable "admin_password" {
  type        = string
  default     = "W00dstock-e2e!"
  description = "Administrator/wsuser password. Plaintext in the seed ISO by design — disposable VM, private lab network, no secret value to protect."
}

variable "output_dir" {
  type    = string
  default = "output"
}

source "qemu" "windows" {
  accelerator = "kvm"
  vm_name     = "windows-golden.qcow2"
  cpus        = 4
  memory      = 4096

  # Windows Setup carries no virtio-blk/virtio-net driver. q35's AHCI
  # controller and e1000e NIC are both driven by in-box drivers, so the
  # installer sees a disk and gets a network without any extra step. lib/qemu.sh
  # boots the produced image the same way.
  machine_type     = "q35"
  disk_size        = "48G"
  disk_interface   = "ide"
  net_device       = "e1000e"
  format           = "qcow2"
  output_directory = var.output_dir
  cpu_model        = "host"

  vtpm            = true
  tpm_device_type = "tpm-crb"

  efi_boot          = true
  efi_firmware_code = "/usr/share/OVMF/OVMF_CODE_4M.fd"
  efi_firmware_vars = "/usr/share/OVMF/OVMF_VARS_4M.fd"

  iso_url      = var.windows_iso
  iso_checksum = var.windows_iso_checksum

  cd_label = "UNATTEND"
  cd_content = {
    "autounattend.xml" = templatefile("${path.root}/cd/autounattend.xml", {
      admin_password = var.admin_password
    })
    "setup.ps1" = templatefile("${path.root}/cd/setup.ps1", {
      ssh_pubkey = var.ssh_public_key
    })
  }

  headless = true

  # "Press any key to boot from CD or DVD" opens within the first few seconds
  # and closes around eight; start immediately and hammer for the whole
  # window rather than trying to time a single keystroke. Confirmed reliable
  # unattended (no manual keypress, no QMP send-key) once vtpm was added —
  # every earlier attempt without a TPM present on q35 landed in the EFI
  # Shell instead, TPM presence apparently shifts q35's device-init timing
  # enough to matter here, though the exact mechanism was never pinned down
  # further than "empirically, with vtpm this works every time and without
  # it it never does."
  #
  # ⚠️ A stray space can land on the "Cancel" button once Setup starts
  # copying files (observed once, during an early isolated test): if so,
  # "Are you sure you want to quit?" opens with "No" focused and the install
  # stalls until something presses Enter. boot_command has no way to wait for
  # that state conditionally — occasional manual-recovery case, screendump
  # over QMP to check (add `-qmp` to qemuargs), send a `ret` key to dismiss
  # it. Not observed more than once across every build tonight.
  boot_wait = "1s"
  boot_command = [for i in range(40) : "<spacebar><wait1>"]

  communicator          = "ssh"
  ssh_username           = "Administrator"
  ssh_private_key_file   = var.ssh_private_key_file
  ssh_timeout            = "90m"
  shutdown_command       = "shutdown /s /t 10 /f"
  shutdown_timeout       = "15m"

  qemuargs = [
    ["-serial", "file:${var.output_dir}/install-console.log"],
  ]
}

build {
  sources = ["source.qemu.windows"]

  # efi_boot's own mutated variable store shows up as ${output_dir}/efivars.fd
  # — not "<vm_name>_OVMF_VARS.fd" as an earlier version of this file assumed
  # (confirmed empirically: renaming that guess to the real name is the fix,
  # not a coincidence). No `|| true` here on purpose: a missing vars file
  # means the image cannot boot at runtime (contract: lib/qemu.sh needs the
  # exact store the image was installed against), so a failed copy must fail
  # the build, not degrade into a silently stale images/windows-OVMF_VARS.fd.
  post-processor "shell-local" {
    inline = [
      "mv '${var.output_dir}/windows-golden.qcow2' '${path.root}/../../windows-golden.qcow2'",
      "mv '${var.output_dir}/efivars.fd' '${path.root}/../../windows-OVMF_VARS.fd'",
    ]
  }
}
