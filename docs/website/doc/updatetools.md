# Update tools

You can update the tools defined by default. All you need is to create a file with the following sections and extend one of the command.

The complete file look like this (https://gogs.shadoware.org/ShadowareOrg/woodstock-backup/src/branch/feature/woodstock/server/config/tools.yml) :

```yaml
---
tools:
  df: /bin/df
  btrfs: /bin/btrfs
  compsize: /usr/sbin/compsize
  ping: /bin/ping
  nmblookup: /usr/bin/nmblookup
  ssh: /usr/bin/ssh
  rsync: /usr/bin/rsync
  stat: /usr/bin/stat

command:
  rsh: "${ssh} -o stricthostkeychecking=no -o userknownhostsfile=/dev/null -o batchmode=yes -o passwordauthentication=no"
  ping: "${ping} -c 1 ${ip}"
  resolveNetbiosFromHostname: "${nmblookup} ${hostname}"
  resolveNetbiosFromIP: "${nmblookup} -A ${ip}"
  getFilesystem: "${stat} -f --format=%T ${hostPath}"
  statsSpaceUsage: "${df} -k --print-type ${hostPath}"
  statsDiskUsage: "${btrfs} qgroup show --raw ${hostPath}"
  btrfsQGroupEnable: "${btrfs} quota enable ${hostPath}"
  btrfsBackupQGroupCreate: "${btrfs} qgroup create 1/${qGroupId} ${hostPath}"
  btrfsBackupQGroupDestroy: "${btrfs} qgroup destroy 1/${qGroupId} ${hostPath}"
  btrfsListSubvolume: "${btrfs} subvolume list -t ${hostPath}"
  btrfsCreateSubvolume: "${btrfs} subvolume create -i 1/${qGroupId} ${destBackupPath}"
  btrfsCreateSnapshot: "${btrfs} subvolume snapshot -i 1/${qGroupId} ${srcBackupPath} ${destBackupPath}"
  btrfsDeleteSnapshot: "${btrfs} subvolume delete ${destBackupPath}"
  btrfsMarkROSubvolume: "${btrfs} property set -ts ${destBackupPath} ro true"
  btrfsMarkRWSubvolume: "${btrfs} property set -ts ${destBackupPath} ro false"
  btrfsGetCompressionSize: "${compsize} -b ${destBackupPath}"

paths:
  hostnamePath: "${hostPath}/${hostname}"
  qgroupHostPath: "${hostPath}/${hostname}/qgroup"
  srcBackupPath: "${hostPath}/${hostname}/${srcBackupNumber}"
  destBackupPath: "${hostPath}/${hostname}/${destBackupNumber}"
```
