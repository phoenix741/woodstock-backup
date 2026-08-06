# Runs once, at first logon, from the seed ISO. autounattend.xml locates it by
# searching every filesystem drive for setup.ps1 rather than hard-coding a
# letter, since which drive the UNATTEND CD lands on is not guaranteed stable.
#
# ${ssh_pubkey} is substituted by Packer's templatefile() (windows.pkr.hcl's
# cd_content block). No lab-IP placeholder here: the lab NIC does not exist
# yet at build time under Packer, its static IP is set at run time instead
# — see provision/client-windows.ps1.
#
# Its job is to make the VM drivable exactly like the Debian and FreeBSD guests:
# OpenSSH Server, the harness key, and the lab NIC configured statically.

$ErrorActionPreference = 'Stop'
Start-Transcript -Path 'C:\Windows\Temp\e2e-setup.log' -Append

Write-Host '=== clock ==='
# QEMU hands the guest a UTC hardware clock. Left on the installer's default
# timezone, Windows reads that as local time and its UTC ends up hours off,
# which makes the server's JWTs look expired to the agent. Pin it to UTC.
Set-TimeZone -Id 'UTC'

Write-Host '=== wait for network ==='
# FirstLogonCommands runs this before NIC0's DHCP lease necessarily lands —
# ipconfig confirmed empty during Setup, populated (10.0.2.15) once fully
# booted, so it is a startup race, not a genuine connectivity gap. Poll
# instead of assuming it is already up: Add-WindowsCapability below has no
# timeout of its own and, run too early, gets stuck retrying against
# Windows Update for a very long time rather than failing fast.
$netReady = $false
for ($i = 0; $i -lt 60; $i++) {
    if (Test-Connection -ComputerName 8.8.8.8 -Count 1 -Quiet -ErrorAction SilentlyContinue) {
        $netReady = $true
        break
    }
    Start-Sleep -Seconds 2
}
if (-not $netReady) {
    Write-Host 'WARNING: no network after 120s, continuing anyway'
}

Write-Host '=== OpenSSH Server ==='
# The payload comes from Windows Update, so NIC0 (user-mode, with internet)
# must be up at this point.
Add-WindowsCapability -Online -Name 'OpenSSH.Server~~~~0.0.1.0'
Set-Service -Name sshd -StartupType Automatic
Start-Service sshd

Write-Host '=== harness key ==='
# Administrators authenticate through this file, not through the per-user
# authorized_keys — that is what the default sshd_config on Windows enforces.
$adminKeys = 'C:\ProgramData\ssh\administrators_authorized_keys'
Set-Content -Path $adminKeys -Value '${ssh_pubkey}' -Encoding ascii
icacls $adminKeys /inheritance:r /grant 'Administrators:F' /grant 'SYSTEM:F'

# PowerShell as the login shell keeps ssh_script/ps_script symmetrical with the
# Unix guests.
New-ItemProperty -Path 'HKLM:\SOFTWARE\OpenSSH' -Name DefaultShell `
    -Value 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe' `
    -PropertyType String -Force

Write-Host '=== firewall ==='
New-NetFirewallRule -Name sshd -DisplayName 'OpenSSH Server' -Enabled True `
    -Direction Inbound -Protocol TCP -Action Allow -LocalPort 22 -ErrorAction SilentlyContinue
# The server dials the agent on 3657; without this the connection is dropped
# before ws_client_daemon ever sees it.
New-NetFirewallRule -Name woodstock-agent -DisplayName 'Woodstock Agent' -Enabled True `
    -Direction Inbound -Protocol TCP -Action Allow -LocalPort 3657 -ErrorAction SilentlyContinue

Write-Host '=== firewall profiles ==='
# NOT the lab NIC's static IP here — deliberately. This script runs once, at
# first logon, *during the Packer build* — and the build VM only ever has
# the one NIC Packer manages for its own communicator. The lab NIC is a
# second one that vm_start (lib/qemu.sh) attaches only at *run* time, so it
# does not exist yet when this line runs; a MAC-matching Get-NetAdapter
# lookup here always finds nothing (confirmed: every build transcript logs
# "WARNING: no lab NIC found"). Setting its static IP is
# provision/client-windows.ps1's job instead, every boot, where the
# interface actually exists — see that script for why DHCP has to be
# disabled there too, and what silently broke when it wasn't.
Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True

Write-Host '=== VSS ==='
# The Volume Shadow Copy service is manual-start by default; the agent needs it
# running to snapshot C:, which is what makes NTUSER.DAT backupable.
Set-Service -Name VSS -StartupType Automatic
Start-Service VSS -ErrorAction SilentlyContinue

Write-Host '=== power settings ==='
# Never sleep: a suspended VM looks exactly like a hung one to the harness.
powercfg /change standby-timeout-ac 0
powercfg /change hibernate-timeout-ac 0
powercfg /change monitor-timeout-ac 0

# Left for a human glancing at the filesystem, not polled by any builder.
New-Item -Path 'C:\e2e-setup-done' -ItemType File -Force | Out-Null

Stop-Transcript

# build-windows.sh's completion gate is this shutdown, not an SSH check: it
# watches QMP for the guest-initiated SHUTDOWN event this emits, and dies if
# it never arrives (see that script's tail). No race to worry about here,
# unlike Packer's communicator-driven builds (Debian/FreeBSD) — this script
# never waits on SSH at all.
Stop-Computer -Force
