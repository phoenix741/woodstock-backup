# Golden image for the Debian client/server of the Woodstock e2e suite.
#
# Bare trixie + SSH + the packages the tests need — everything Woodstock-
# specific happens in provision/. See e2e/images/packer/debian/http/preseed.cfg
# for the actual install recipe and its rationale.

packer {
  required_version = ">= 1.9.0"
  required_plugins {
    qemu = {
      version = ">= 1.0.9"
      source  = "github.com/hashicorp/qemu"
    }
  }
}

variable "debian_iso" {
  type        = string
  description = "Path to the Debian netinst ISO. Fournir via PKR_VAR_debian_iso."
}

variable "debian_iso_checksum" {
  type        = string
  default     = "none"
  description = "sha256:<hex> of debian_iso. Fournir via PKR_VAR_debian_iso_checksum."
}

variable "ssh_private_key_file" {
  type        = string
  description = "Harness SSH private key (images/cache/id_e2e) — never Packer's ephemeral key, lib/remote.sh depends on this exact file."
}

variable "ssh_public_key" {
  type        = string
  description = "Contents of images/cache/id_e2e.pub, embedded into the preseed by templatefile() — no HTTP round-trip, no generated file on disk."
}

variable "output_dir" {
  type    = string
  default = "output"
}

variable "vm_mem" {
  type    = number
  default = 2048
}

variable "vm_cpus" {
  type    = number
  default = 2
}

variable "vm_disk" {
  type    = string
  default = "12G"
}

source "qemu" "debian" {
  accelerator = "kvm"
  vm_name     = "debian-golden.qcow2"
  cpus        = var.vm_cpus
  memory      = var.vm_mem

  disk_size         = var.vm_disk
  disk_interface    = "virtio"
  disk_compression  = true
  format            = "qcow2"
  output_directory  = var.output_dir

  iso_url      = var.debian_iso
  iso_checksum = var.debian_iso_checksum

  http_content = {
    "/preseed.cfg" = templatefile("${path.root}/http/preseed.cfg", { ssh_pubkey = var.ssh_public_key })
  }
  headless   = true
  net_device = "virtio-net-pci"

  boot_wait = "5s"
  boot_command = [
    "<esc><wait>",
    "install ",
    "auto=true priority=critical ",
    "url=http://{{ .HTTPIP }}:{{ .HTTPPort }}/preseed.cfg ",
    "netcfg/choose_interface=auto ",
    "console=ttyS0,115200n8 --- console=ttyS0,115200n8",
    "<enter>",
  ]

  ssh_username         = "root"
  ssh_private_key_file = var.ssh_private_key_file
  ssh_timeout          = "30m"
  shutdown_command     = "poweroff"

  qemuargs = [
    ["-serial", "file:${var.output_dir}/install-console.log"],
  ]
}

build {
  sources = ["source.qemu.debian"]

  # The harness (lib/qemu.sh golden_for()) reads the golden image from
  # e2e/images/, not from this template's own output/ directory.
  post-processor "shell-local" {
    inline = [
      "mv '${var.output_dir}/debian-golden.qcow2' '${path.root}/../../debian-golden.qcow2'",
    ]
  }
}
