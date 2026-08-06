# Guest-side (Windows): write checksums.sha256 over a directory tree.
#
# The format has to match GNU sha256sum exactly — lowercase hex, TWO spaces,
# then a ./-prefixed relative path with forward slashes — because it is produced
# here and consumed by tests/70-restore.sh, and because the Unix guests generate
# the very same file. `sha256sum -c` silently skips lines it cannot parse, so a
# near-miss would "verify" nothing rather than fail.
#
# *.nobackup is excluded on purpose: the share excludes it, so a restore must not
# bring it back and listing it here would turn correct behaviour into a failure.
#
# Usage: gen-checksums.ps1 <directory>

param([Parameter(Mandatory = $true)][string]$Dir)

$ErrorActionPreference = 'Stop'

$root = (Resolve-Path $Dir).Path.TrimEnd('\')
$lines = Get-ChildItem -Path $root -Recurse -File |
    Where-Object { $_.Name -ne 'checksums.sha256' -and $_.Extension -ne '.nobackup' } |
    ForEach-Object {
        $relative = $_.FullName.Substring($root.Length + 1).Replace('\', '/')
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $_.FullName).Hash.ToLower()
        "$hash  ./$relative"
    } | Sort-Object

# No BOM and Unix line endings: sha256sum treats a BOM as part of the first
# hash, and a trailing CR as part of the last filename.
$text = ($lines -join "`n") + "`n"
[System.IO.File]::WriteAllText((Join-Path $root 'checksums.sha256'), $text,
                               (New-Object System.Text.UTF8Encoding $false))

Write-Host "files checksummed: $($lines.Count)"
