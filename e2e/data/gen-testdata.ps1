# Guest-side (Windows): the Windows counterpart of gen-testdata.sh.
#
# Same file names and same roles, so the platform-independent assertions in
# tests/40-backup.sh hold everywhere:
#
#   big-*.bin        volume, and incompressible so pool size is meaningful
#   twin-a/b.bin     byte-identical -> intra-backup deduplication
#   mutable.txt      rewritten between backup #1 and #2 -> incremental
#   xattr.txt        carries an NTFS alternate data stream, the closest thing
#                    Windows has to a user.* extended attribute
#   acl.txt          carries an explicit NTFS ACE
#   link-to-*        symlink (needs Administrator, which the harness runs as)
#   sparse.bin       sparse file, via fsutil
#   "space and é"    non-ASCII and spaces in the name
#   skip.nobackup    matches the share exclude -> must NOT appear in the manifest
#
# Deliberately NOT mirrored: nothing here tries to reproduce POSIX xattr/ACL
# semantics. tests/40-backup.sh only asserts those two on the Debian client; on
# Windows the discriminating assertions are the VSS snapshot and NTUSER.DAT
# (tests/50-snapshot.sh), which need no help from this script.
#
# Usage: gen-testdata.ps1 <directory>

param([Parameter(Mandatory = $true)][string]$Dir)

$ErrorActionPreference = 'Stop'
$BigMB = if ($env:BIG_MB) { [int]$env:BIG_MB } else { 64 }

if (Test-Path $Dir) { Remove-Item -Recurse -Force $Dir }
New-Item -ItemType Directory -Force -Path (Join-Path $Dir 'nested\deeper') | Out-Null

Write-Host "generating in $Dir (large files: $BigMB MiB each)"

function New-RandomFile {
    param([string]$Path, [int]$MiB)
    # Incompressible content, written in 1 MiB blocks so a 64 MiB file does not
    # need 64 MiB of managed memory at once.
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    $buffer = New-Object byte[] (1MB)
    $stream = [System.IO.File]::Create($Path)
    try {
        for ($i = 0; $i -lt $MiB; $i++) {
            $rng.GetBytes($buffer)
            $stream.Write($buffer, 0, $buffer.Length)
        }
    } finally {
        $stream.Dispose()
        $rng.Dispose()
    }
}

foreach ($n in 1, 2, 3) {
    New-RandomFile -Path (Join-Path $Dir "big-$n.bin") -MiB $BigMB
}

# Identical content in two places: the pool must store the chunks once.
New-RandomFile -Path (Join-Path $Dir 'twin-a.bin') -MiB 8
Copy-Item (Join-Path $Dir 'twin-a.bin') (Join-Path $Dir 'nested\twin-b.bin')

Set-Content -LiteralPath (Join-Path $Dir 'mutable.txt') -Value 'version 1' -Encoding utf8

$xattr = Join-Path $Dir 'xattr.txt'
Set-Content -LiteralPath $xattr -Value 'has an extended attribute' -Encoding utf8
Set-Content -LiteralPath $xattr -Stream 'user.woodstock' -Value 'e2e'

$acl = Join-Path $Dir 'acl.txt'
Set-Content -LiteralPath $acl -Value 'has an ACL' -Encoding utf8
$rules = Get-Acl -LiteralPath $acl
$rules.AddAccessRule((New-Object System.Security.AccessControl.FileSystemAccessRule(
    'Users', 'Read', 'Allow')))
Set-Acl -LiteralPath $acl -AclObject $rules

# Symbolic links need SeCreateSymbolicLinkPrivilege — Administrator has it, but
# say so rather than dying if the harness is ever run as someone else.
try {
    New-Item -ItemType SymbolicLink -Path (Join-Path $Dir 'link-to-big-1') `
             -Target (Join-Path $Dir 'big-1.bin') -Force | Out-Null
} catch {
    Write-Warning "could not create the symlink: $_"
}

$sparse = Join-Path $Dir 'sparse.bin'
New-Item -ItemType File -Path $sparse -Force | Out-Null
& fsutil sparse setflag $sparse | Out-Null
& fsutil file seteof $sparse 33554432 | Out-Null   # 32 MiB

Set-Content -LiteralPath (Join-Path $Dir 'space and éàü.txt') `
            -Value 'accents and spaces' -Encoding utf8

Set-Content -LiteralPath (Join-Path $Dir 'skip.nobackup') `
            -Value 'this must be excluded' -Encoding utf8

Set-Content -LiteralPath (Join-Path $Dir 'nested\deeper\leaf.txt') `
            -Value 'deep file' -Encoding utf8

& (Join-Path $PSScriptRoot 'gen-checksums.ps1') $Dir

Write-Host "generated:"
'{0:N0} MiB' -f ((Get-ChildItem -Recurse -File $Dir |
    Measure-Object -Property Length -Sum).Sum / 1MB)
