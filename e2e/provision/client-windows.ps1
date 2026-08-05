# Guest-side: turn a bare Windows VM into a Woodstock client.
#
# There is no Windows package: the CI produces a bare ws_client_daemon.exe, and
# the agent registers itself as a service through its own `install-service`
# subcommand (client-rs/src/winserv.rs). That subcommand is the closest thing to
# a documented installation procedure — docs/website/doc/ has installation pages
# for Debian and FreeBSD but none for Windows.
#
# Two things about install-service that shape this script:
#   * main() reads config.yaml BEFORE dispatching to the subcommand, so a
#     config file has to exist first. tests/30-certs.sh later unzips the real
#     enrollment bundle over it.
#   * it bakes `--config-dir <dir> run-service` into the registered command line
#     and runs as LocalSystem. Without --config-dir the service would resolve
#     %APPDATA%\woodstock under systemprofile and never see anything written
#     here.
#
# The lab NIC is configured here, every boot, by MAC — not baked into the
# image (see the "=== lab NIC ===" section below for why the image can't do
# this itself under Packer).
#
# Environment:
#   LAB_IP     static address to assign to the lab NIC
#   HOSTNAME_  name to report to the server (must match hosts.yml)

$ErrorActionPreference = 'Stop'

$hostName = $env:HOSTNAME_
if (-not $hostName) { throw 'HOSTNAME_ is required' }

$installDir = 'C:\Program Files\Woodstock'
$configDir  = 'C:\ProgramData\woodstock'
$dataDir    = 'C:\e2e'
$toolsDir   = 'C:\e2e-tools'

Write-Host '=== clock ==='
# QEMU presents the RTC in UTC, but the image installed itself on the default
# Pacific timezone, so Windows read that UTC value as Pacific local time and its
# own idea of UTC ended up SEVEN HOURS ahead. Every token the server issued then
# looked expired to the agent:
#
#   ERROR woodstock_client_rs::server: Failed to authenticate: ExpiredSignature
#
# The agent retries authentication before it binds, so the symptom is not just a
# failed backup: nothing ever listens on 3657 either. Both assertions failed
# from this single cause.
#
# Two steps, and both are needed. Set-TimeZone alone fixes nothing: Windows
# stores the system time in UTC and derives local time from the zone, so
# changing the zone moves the displayed local time and leaves the wrong UTC
# untouched. Set-Date is what actually corrects it. Setting the zone to UTC as
# well is what keeps it correct across a reboot, since the RTC it re-reads is
# UTC.
Set-TimeZone -Id 'UTC'
if ($env:HOST_UTC) {
    $t = [DateTime]::ParseExact($env:HOST_UTC, 'yyyy-MM-ddTHH:mm:ssZ',
                                [Globalization.CultureInfo]::InvariantCulture,
                                [Globalization.DateTimeStyles]::AdjustToUniversal -bor
                                [Globalization.DateTimeStyles]::AssumeUniversal)
    Set-Date -Date $t.ToLocalTime() | Out-Null
}
Write-Host "guest UTC is now $((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss'))"

Write-Host '=== lab NIC ==='
# NOT already configured, despite what this file used to assume: that was
# true for images/build-windows.sh (its own qemu invocation included the lab
# NIC at build time, matching what setup.ps1's MAC-based Get-NetAdapter
# lookup expected to find), but is not true of images/packer/windows/ —
# Packer's own build VM only ever has the one NIC it manages for its own
# communicator, so setup.ps1's lab-NIC block silently finds nothing
# ("WARNING: no lab NIC found" in every build transcript) and never runs.
# vm_start (lib/qemu.sh) only attaches the second, lab-segment NIC at
# *runtime* — so this is where its static IP has to be set, every boot, not
# once at image build time.
#
# DHCP has to be disabled before the static IP goes on, not after: left
# enabled (the default), the DHCP client keeps retrying in the background on
# this interface — there is no DHCP server on the multicast lab segment, so
# it falls back to an APIPA 169.254.x.x address and never installs the
# on-link route the static IP's /24 needs. Confirmed empirically: with DHCP
# left on, Get-NetRoute showed only broadcast/multicast/APIPA routes on this
# interface, no 10.66.0.0/24 route, and the agent's registration POST to the
# server went out the *other* NIC's default route instead — wrong
# destination, silently, not a timeout.
$lab = Get-NetAdapter | Where-Object { $_.MacAddress -like '52-54-00-E2-E1-*' }
if ($lab) {
    Set-NetIPInterface -InterfaceIndex $lab.ifIndex -Dhcp Disabled
    Get-NetIPAddress -InterfaceIndex $lab.ifIndex -AddressFamily IPv4 -ErrorAction SilentlyContinue |
        Remove-NetIPAddress -Confirm:$false -ErrorAction SilentlyContinue
    New-NetIPAddress -InterfaceIndex $lab.ifIndex -IPAddress $env:LAB_IP `
        -PrefixLength 24 -ErrorAction SilentlyContinue | Out-Null
    Set-NetFirewallProfile -Profile Domain,Public,Private -Enabled True
    Write-Host "lab interface $($lab.Name) = $($env:LAB_IP)"
} else {
    Write-Warning 'no lab NIC found'
}

Write-Host '=== Visual C++ runtime ==='
# ws_client_daemon.exe imports VCRUNTIME140.dll: the CI builds against the MSVC
# target with the dynamic CRT, so the binary cannot start on a clean Windows
# without the Visual C++ 2015-2022 redistributable. Without it, every
# invocation — including `install-service` — dies with exit code 0xC0000135
# (STATUS_DLL_NOT_FOUND) and no message at all.
#
# This is a real deployment requirement of the product, not a quirk of the test
# environment: it is installed here because a user would have to install it too.
# Building with `-C target-feature=+crt-static` would remove the need entirely.
if (-not (Test-Path 'C:\Windows\System32\VCRUNTIME140.dll')) {
    Write-Host 'VCRUNTIME140.dll missing — installing the redistributable'
    $redist = Join-Path $env:TEMP 'vc_redist.x64.exe'
    Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vc_redist.x64.exe' `
                      -OutFile $redist -UseBasicParsing
    $p = Start-Process -FilePath $redist -ArgumentList '/install', '/quiet', '/norestart' `
                       -Wait -PassThru
    # 3010 = success, reboot required. The DLL is usable straight away.
    if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) {
        throw "vc_redist returned $($p.ExitCode)"
    }
    if (-not (Test-Path 'C:\Windows\System32\VCRUNTIME140.dll')) {
        throw 'the redistributable installed but VCRUNTIME140.dll is still absent'
    }
}

Write-Host '=== install the agent ==='
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item 'C:\packages\windows\ws_client_daemon.exe'  $installDir -Force
Copy-Item 'C:\packages\windows\ws_client_console.exe' $installDir -Force

Write-Host '=== hostname ==='
# Renaming needs a reboot to take effect in Windows itself, but the agent
# reports the name from config.yaml, which is what the server matches on.
if ((Get-CimInstance Win32_ComputerSystem).Name -ne $hostName) {
    Rename-Computer -NewName $hostName -Force -ErrorAction SilentlyContinue
}

Write-Host '=== minimal configuration ==='
# install-service refuses to run without it. Only the fields the agent needs to
# start: the enrollment bundle replaces this file wholesale in 30-certs.
New-Item -ItemType Directory -Force -Path $configDir | Out-Null
# WriteAllText with an explicit BOM-less encoding, not `Set-Content -Encoding
# utf8`: PowerShell 5.1 writes a UTF-8 BOM, and the agent's YAML parser fails on
# it with the thoroughly misleading "missing field `password` at line 1 column 2".
$configText = @"
hostname: $hostName
bind: 0.0.0.0:3657
password: e2e-shared-secret
"@ -replace "`r`n", "`n"
[System.IO.File]::WriteAllText((Join-Path $configDir 'config.yaml'), $configText + "`n",
                               (New-Object System.Text.UTF8Encoding $false))

Write-Host '=== generate the data to back up ==='
New-Item -ItemType Directory -Force -Path $toolsDir | Out-Null
Copy-Item 'C:\packages\gen-testdata.ps1'  $toolsDir -Force
Copy-Item 'C:\packages\gen-checksums.ps1' $toolsDir -Force
& (Join-Path $toolsDir 'gen-testdata.ps1') $dataDir

Write-Host '=== register the service ==='
# install-service also adds the firewall rule, and starts the service straight
# away. Starting is expected to fail here — the agent has no certificates until
# enrollment — so the failure is tolerated and 30-certs restarts it. Registering
# is what matters: 10-service checks the service exists, not that it runs.
$exe = Join-Path $installDir 'ws_client_daemon.exe'
try {
    & $exe --config-dir $configDir install-service 2>&1 | Write-Host
} catch {
    Write-Host "install-service reported: $_ (expected before enrollment)"
}
# Report the exit status rather than swallowing it: a silent failure here used
# to surface only as "the service was not registered", with no cause.
Write-Host "install-service exit code: $LASTEXITCODE"

if (-not (Get-Service -Name 'woodstock_client_daemon' -ErrorAction SilentlyContinue)) {
    throw 'the woodstock_client_daemon service was not registered'
}

Write-Host '=== ready ==='
Get-Service -Name 'woodstock_client_daemon' | Format-List Name, Status, StartType
