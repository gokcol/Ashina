# Reject mixed-architecture payloads before packaging a portable app or installer.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string[]]$Path,
    [Parameter(Mandatory = $true)][ValidateSet('x64', 'arm64')][string]$Architecture
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$machine = if ($Architecture -eq 'arm64') { 0xaa64 } else { 0x8664 }
foreach ($file in $Path) {
    $reader = [IO.BinaryReader]::new([IO.File]::OpenRead([IO.Path]::GetFullPath($file)))
    try {
        if ($reader.BaseStream.Length -lt 64 -or $reader.ReadUInt16() -ne 0x5a4d) {
            throw "Invalid Windows executable: $file"
        }
        $reader.BaseStream.Position = 0x3c
        $offset = $reader.ReadUInt32()
        if ($offset -lt 64 -or $offset -gt $reader.BaseStream.Length - 6) {
            throw "Invalid PE header: $file"
        }
        $reader.BaseStream.Position = $offset
        if ($reader.ReadUInt32() -ne 0x00004550 -or $reader.ReadUInt16() -ne $machine) {
            throw "Expected a native Windows $Architecture binary: $file"
        }
    } finally { $reader.Dispose() }
    Write-Host "Verified native $Architecture binary: $file"
}
