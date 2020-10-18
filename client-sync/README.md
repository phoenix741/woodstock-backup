## Compile

Install the library using [https://github.com/semlanik/qtprotobuf#linux-build](https://github.com/semlanik/qtprotobuf#linux-build)

## Launch daemon

export QT_PLUGIN_PATH=`pwd`/vcpkg_installed/x64-linux/usr/lib/x86_64-linux-gnu/qt5/plugins
export LD_LIBRARY_PATH=`pwd`/vcpkg_installed/x64-linux/usr/lib/x86_64-linux-gnu/:`pwd`/vcpkg_installed/x64-linux/usr/lib/x86_64-linux-gnu/qt5/plugins/servicebackends/

## Benchmark

With gRPC (on same PC):

Server part:
[01:47:03] 360905 files read / 31.82 GiB total / 39.54 GiB transferred / 28.16 GiB compressed

Client part:
[01:47:05] 416943 files read / 56038 error(s) / 41.71 GiB total / 39.53 GiB transferred
[01:47:00] 416867 files read / 55360 error(s) / 41.66 GiB total / 39.45 GiB transferred

With gRPC (on distante PC)

Server part:
[03:01:40] 360905 files read / 31.82 GiB total / 39.54 GiB transferred / 28.16 GiB compressed

Client part:
[03:01:43] 416943 files read / 56038 error(s) / 41.71 GiB total / 39.53 GiB transferred

With rsync on distant

Copy of 82 Go !

real 29m1,480s
user 14m54,049s
sys 3m54,729s

Backuppc

Size = 72 334.2M ???
Temps = 133.1 min = 2h13

With Rest (On Same PC)

Server part:
[02:09:00] 360906 files read / 41.49 GiB total / 41.21 GiB transferred / 29.81 GiB compressed

Client part:
[02:09:09] 416942 files read / 56036 error(s) / 41.71 GiB total / 41.21 GiB transferred

With Rest (On distant)

Server part:
[04:00:48] 360907 files read / 41.49 GiB total / 41.21 GiB transferred / 29.81 GiB compressed

Client part:
[04:00:48] 416943 files read / 56036 error(s) / 41.71 GiB total / 41.21 GiB transferred

==========================

gRPC (Distant PC)

Client (walk)
[01:41:38] 430403 files read / 33 error(s) / 43.50 GiB total / 0 bytes transferred

Server (info)
real 211m38,440s
user 0m0,898s
sys 0m0,968s

---

With tcmalloc
real 193m5,004s
user 0m1,637s
sys 0m0,319s

---

rsync distant PC

time sudo rsync -a --exclude="phoenix/tensorflow" --exclude="phoenix/tmp" --exclude="phoenix/.composer" --exclude="*node_modules" --exclude="*mongodb/db" --exclude="phoenix/.ccache" --exclude="_mongodb/dump" --exclude="phoenix/usr/android-sdk" --exclude="phoenix/.cache" --exclude="phoenix/.CloudStation" --exclude="phoenix/.android" --exclude="phoenix/.AndroidStudio_" --exclude="phoenix/usr/android-studio" --exclude="_.vmdk" --exclude="phoenix/.nvm" --exclude="_.vdi" --exclude="phoenix/.local/share/Trash" --exclude="phoenix/VirtualBox VMs" --exclude="*mongodb/configdb" --exclude="phoenix/.thumbnails" --exclude="phoenix/.VirtualBox" --exclude="phoenix/.vagrant.d" --exclude="phoenix/vagrant" --exclude="phoenix/.npm" --exclude="phoenix/Pictures" --exclude="phoenix/Documents synhronisés" --exclude="phoenix/dwhelper" --exclude="phoenix/snap" --exclude="phoenix/.local/share/flatpak" --exclude="phoenix/usr/AndroidSdk" --exclude="public/kg/gallery" --exclude="*vcpkg" phoenix@192.168.101.14:/home /var/lib/woodstock/rsync/
phoenix@192.168.101.14's password:
rsync: opendir "/home/lost+found" failed: Permission denied (13)
rsync error: some files/attrs were not transferred (see previous errors) (code 23) at main.c(1677) [generator=3.1.3]

real 37m54,868s
user 15m24,425s
sys 13m5,332s

with checksum

time sudo rsync -a --checksum --stats --exclude="phoenix/tensorflow" --exclude="phoenix/tmp" --exclude="phoenix/.composer" --exclude="*node_modules" --exclude="*mongodb/db" --exclude="phoenix/.ccache" --exclude="_mongodb/dump" --exclude="phoenix/usr/android-sdk" --exclude="phoenix/.cache" --exclude="phoenix/.CloudStation" --exclude="phoenix/.android" --exclude="phoenix/.AndroidStudio_" --exclude="phoenix/usr/android-studio" --exclude="_.vmdk" --exclude="phoenix/.nvm" --exclude="_.vdi" --exclude="phoenix/.local/share/Trash" --exclude="phoenix/VirtualBox VMs" --exclude="*mongodb/configdb" --exclude="phoenix/.thumbnails" --exclude="phoenix/.VirtualBox" --exclude="phoenix/.vagrant.d" --exclude="phoenix/vagrant" --exclude="phoenix/.npm" --exclude="phoenix/Pictures" --exclude="phoenix/Documents synhronisés" --exclude="phoenix/dwhelper" --exclude="phoenix/snap" --exclude="phoenix/.local/share/flatpak" --exclude="phoenix/usr/AndroidSdk" --exclude="public/kg/gallery" --exclude="*vcpkg" phoenix@192.168.101.14:/home /var/lib/woodstock/rsync/
phoenix@192.168.101.14's password:
rsync: opendir "/home/lost+found" failed: Permission denied (13)

Number of files: 823,760 (reg: 667,981, dir: 122,584, link: 33,192, special: 3)
Number of created files: 823,760 (reg: 667,981, dir: 122,584, link: 33,192, special: 3)
Number of deleted files: 0
Number of regular files transferred: 667,981
Total file size: 78,758,344,402 bytes
Total transferred file size: 78,757,629,272 bytes
Literal data: 78,757,616,466 bytes
Matched data: 0 bytes
File list size: 41,615,797
File list generation time: 0.001 seconds
File list transfer time: 0.000 seconds
Total bytes sent: 13,446,051
Total bytes received: 78,843,581,662

sent 13,446,051 bytes received 78,843,581,662 bytes 27,918,933.51 bytes/sec
total size is 78,758,344,402 speedup is 1.00
rsync error: some files/attrs were not transferred (see previous errors) (code 23) at main.c(1677) [generator=3.1.3]

real 47m4,521s
user 14m36,993s
sys 12m15,842

---

Second times

time sudo rsync -a --checksum --stats --exclude="phoenix/tensorflow" --exclude="phoenix/tmp" --exclude="phoenix/.composer" --exclude="*node_modules" --exclude="*mongodb/db" --exclude="phoenix/.ccache" --exclude="_mongodb/dump" --exclude="phoenix/usr/android-sdk" --exclude="phoenix/.cache" --exclude="phoenix/.CloudStation" --exclude="phoenix/.android" --exclude="phoenix/.AndroidStudio_" --exclude="phoenix/usr/android-studio" --exclude="_.vmdk" --exclude="phoenix/.nvm" --exclude="_.vdi" --exclude="phoenix/.local/share/Trash" --exclude="phoenix/VirtualBox VMs" --exclude="*mongodb/configdb" --exclude="phoenix/.thumbnails" --exclude="phoenix/.VirtualBox" --exclude="phoenix/.vagrant.d" --exclude="phoenix/vagrant" --exclude="phoenix/.npm" --exclude="phoenix/Pictures" --exclude="phoenix/Documents synhronisés" --exclude="phoenix/dwhelper" --exclude="phoenix/snap" --exclude="phoenix/.local/share/flatpak" --exclude="phoenix/usr/AndroidSdk" --exclude="public/kg/gallery" --exclude="*vcpkg" phoenix@192.168.101.14:/home /var/lib/woodstock/rsync/
[sudo] Mot de passe de phoenix : 
phoenix@192.168.101.14's password:
rsync: opendir "/home/lost+found" failed: Permission denied (13)

Number of files: 823,764 (reg: 667,985, dir: 122,584, link: 33,192, special: 3)
Number of created files: 15 (reg: 15)
Number of deleted files: 0
Number of regular files transferred: 72
Total file size: 78,771,098,639 bytes
Total transferred file size: 763,398,891 bytes
Literal data: 42,849,302 bytes
Matched data: 720,549,733 bytes
File list size: 13,297,695
File list generation time: 0.001 seconds
File list transfer time: 0.000 seconds
Total bytes sent: 465,433
Total bytes received: 81,338,077

sent 465,433 bytes received 81,338,077 bytes 40,769.25 bytes/sec
total size is 78,771,098,639 speedup is 962.93
rsync error: some files/attrs were not transferred (see previous errors) (code 23) at main.c(1677) [generator=3.1.3]

real 33m29,695s
user 5m2,663s
sys 2m1,372s

---

borg

time borg create --stats /home/phoenix/mnt::first --exclude="phoenix/tensorflow" --exclude="phoenix/tmp" --exclude="phoenix/.composer" --exclude="*node_modules" --exclude="*mongodb/db" --exclude="phoenix/.ccache" --exclude="_mongodb/dump" --exclude="phoenix/usr/android-sdk" --exclude="phoenix/.cache" --exclude="phoenix/.CloudStation" --exclude="phoenix/.android" --exclude="phoenix/.AndroidStudio_" --exclude="phoenix/usr/android-studio" --exclude="_.vmdk" --exclude="phoenix/.nvm" --exclude="_.vdi" --exclude="phoenix/.local/share/Trash" --exclude="phoenix/VirtualBox VMs" --exclude="*mongodb/configdb" --exclude="phoenix/.thumbnails" --exclude="phoenix/.VirtualBox" --exclude="phoenix/.vagrant.d" --exclude="phoenix/vagrant" --exclude="phoenix/.npm" --exclude="phoenix/Pictures" --exclude="phoenix/Documents synhronisés" --exclude="phoenix/dwhelper" --exclude="phoenix/snap" --exclude="phoenix/.local/share/flatpak" --exclude="phoenix/usr/AndroidSdk" --exclude="public/kg/gallery" --exclude="*vcpkg" /home

===========================
