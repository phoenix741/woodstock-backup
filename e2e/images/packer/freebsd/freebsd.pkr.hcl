# Golden image for the FreeBSD client/server of the Woodstock e2e suite.
#
# Starts from the official BASIC-CLOUDINIT qcow2 (already installed, cloud-init
# enabled) and configures it declaratively via a NoCloud seed — no scripted
# bsdinstall, no ISO remastering. See cd/user-data for the actual setup and,
# in particular, the IPv6/DAD workaround (the densest piece of hard-won
# knowledge in this tree — do not simplify it away).

packer {
  required_version = ">= 1.9.0"
  required_plugins {
    qemu = {
      version = ">= 1.0.9"
      source  = "github.com/hashicorp/qemu"
    }
  }
}

variable "freebsd_image_url" {
  type        = string
  description = "URL of the FreeBSD BASIC-CLOUDINIT qcow2.xz. Fournir via PKR_VAR_freebsd_image_url."
}

variable "freebsd_image_checksum" {
  type        = string
  default     = "none"
  description = "checksum for freebsd_image_url, e.g. file:<url-to-CHECKSUM.SHA256>. Fournir via PKR_VAR_freebsd_image_checksum."
}

variable "ssh_private_key_file" {
  type        = string
  description = "Harness SSH private key (images/cache/id_e2e) — never Packer's ephemeral key."
}

variable "ssh_public_key" {
  type        = string
  description = "Contents of images/cache/id_e2e.pub, embedded into the cloud-init seed by templatefile()."
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

source "qemu" "freebsd" {
  accelerator = "kvm"
  vm_name     = "freebsd-golden.qcow2"
  cpus        = var.vm_cpus
  memory      = var.vm_mem

  # Starting point is an already-installed disk image, not an ISO: Packer
  # boots it directly and disk_size grows it in place (cloud-init's growfs
  # extends the filesystem into the new space on first boot).
  iso_url      = var.freebsd_image_url
  iso_checksum = var.freebsd_image_checksum
  disk_image   = true
  disk_size    = var.vm_disk

  disk_interface   = "virtio"
  disk_compression = true
  format           = "qcow2"
  output_directory = var.output_dir

  cd_label = "cidata"
  cd_content = {
    "meta-data" = file("${path.root}/cd/meta-data")
    "user-data" = templatefile("${path.root}/cd/user-data", { ssh_pubkey = var.ssh_public_key })
  }

  headless   = true
  net_device = "virtio-net"

  # No boot_command: the image is already installed, and cloud-init runs
  # unattended from the NoCloud seed. QEMU exiting is the completion signal
  # (cd/user-data ends with `power_state: poweroff`).
  boot_wait         = "5s"
  ssh_username      = "root"
  ssh_private_key_file = var.ssh_private_key_file
  ssh_timeout       = "15m"
  shutdown_command  = "poweroff"

  qemuargs = [
    ["-serial", "file:${var.output_dir}/install-console.log"],
  ]
}

build {
  sources = ["source.qemu.freebsd"]

  post-processor "shell-local" {
    inline = [
      "mv '${var.output_dir}/freebsd-golden.qcow2' '${path.root}/../../freebsd-golden.qcow2'",
    ]
  }
}
