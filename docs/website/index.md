---
home: true
layout: home
hero:
  name: Woodstock Backup
  text: Centralized Backup Solution
  tagline: Data protection with advanced deduplication and FUSE filesystem access
  image:
    src: /images/hosts.png
    alt: Woodstock Backup
  actions:
    - theme: brand
      text: Documentation →
      link: /doc/
    - theme: alt
      text: View on Gitea
      link: https://gogs.shadoware.org/ShadowareOrg/woodstock-backup
    - theme: alt
      text: View on Github
      link: https://github.com/phoenix741/woodstock-backup

features:
  - icon: 📡
    title: Centralized Architecture
    details: The server contacts clients to initiate backups, simplifying configuration and enhancing security
  - icon: 🪟
    title: Native Windows Compatibility
    details: No need for rsync or SSH on Windows, works natively with all operating systems
  - icon: 📂
    title: FUSE Filesystem Access
    details: Mount your backups as a regular filesystem to easily browse and access files from any backup point
  - icon: 🧩
    title: Chunk-based Deduplication
    details: Optimized storage based on unique blocks with compression, significantly reducing required disk space
  - icon: 🛡️
    title: Guaranteed Integrity
    details: Cryptographic verification of each data block and integrated maintenance tools
  - icon: 🌐
    title: Modern Web Interface
    details: Complete control of backups, restorations and task monitoring through an intuitive interface
footer: MIT Licensed | Copyright © 2024
---

<script setup>
import LatestVersion from './components/LatestVersionComponent.vue'
</script>

## Latest version

<LatestVersion></LatestVersion>

## Why Woodstock Backup?

Woodstock Backup was developed to solve two major limitations of traditional backup solutions:
- **Simplified Windows backups** without dependency on rsync or SSH
- **Easy access to backup data** through FUSE filesystem mounting

## Features

- **Server-initiated architecture**: the server contacts clients to initiate backups, simplifying configuration and administration
- **Modern web interface** to easily manage backups, view ongoing tasks, restore files, and check statistics
- **Optimized incremental backup** with chunk-based deduplication and compression to reduce storage space
- **Complete metadata preservation** including permissions, timestamps, extended attributes, and ACLs
- **FUSE filesystem integration** allowing you to mount and browse any backup point as a regular directory
- **Integrity verification** based on cryptographic hashes (SHA-256, Blake3, SHA3-256)
- **Complete API** for automation and integration with other systems
- **Open-source** distributed under MIT license
