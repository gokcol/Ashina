# Exercises the real built runtime and failure paths on a disposable runner.
# No models are downloaded and no PDF content is used.
[CmdletBinding()]
param([ValidateSet('x64', 'arm64')][string]$Architecture = 'x64')
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path $PSScriptRoot -Parent
$source = Join-Path $repo "dist/translation/windows-$Architecture"
$copy = Join-Path $PSScriptRoot 'copy-translation-runtime.ps1'
$scratch = Join-Path ([System.IO.Path]::GetTempPath()) ('mypdf-runtime-qa-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $scratch
try {
    $runtime = Join-Path $scratch 'source'
    Copy-Item $source $runtime -Recurse
    $destination = Join-Path $scratch 'package'
    function Expect-Rejection([string]$Pattern) {
        $rejected = $false
        try { & $copy -SourceDirectory $runtime -DestinationDirectory $destination -Architecture $Architecture } catch {
            if ($_.Exception.Message -notmatch $Pattern) { throw }
            $rejected = $true
        }
        if (-not $rejected) { throw "Expected runtime packaging rejection: $Pattern" }
        Write-Host "PASS runtime packaging rejection: $Pattern"
    }
    $dll = Join-Path $runtime 'mypdf_ct2.dll'
    Move-Item $dll "$dll.hidden"
    Expect-Rejection 'runtime or license missing: mypdf_ct2.dll'
    Move-Item "$dll.hidden" $dll
    $license = Join-Path $runtime 'licenses/CTranslate2.txt'
    Move-Item $license "$license.hidden"
    Expect-Rejection 'runtime or license missing: licenses/CTranslate2.txt'
    Move-Item "$license.hidden" $license
    $checksum = Get-Content "$dll.sha256" -Raw
    (('0' * 64) + '  mypdf_ct2.dll') | Set-Content "$dll.sha256" -Encoding ascii
    Expect-Rejection 'checksum does not match'
    $checksum | Set-Content "$dll.sha256" -Encoding ascii
    # A correct checksum alone must not allow an x64 DLL in an ARM64 package,
    # or vice versa. Change only the machine field of the disposable copy.
    $original = [IO.File]::ReadAllBytes($dll)
    $wrong = [byte[]]$original.Clone()
    $pe = [BitConverter]::ToInt32($wrong, 0x3c)
    $otherMachine = [uint16]$(if ($Architecture -eq 'arm64') { 0x8664 } else { 0xaa64 })
    [BitConverter]::GetBytes($otherMachine).CopyTo($wrong, $pe + 4)
    [IO.File]::WriteAllBytes($dll, $wrong)
    ((Get-FileHash $dll -Algorithm SHA256).Hash.ToLowerInvariant() + '  mypdf_ct2.dll') | Set-Content "$dll.sha256" -Encoding ascii
    Expect-Rejection "must be a Windows $Architecture DLL"
    [IO.File]::WriteAllBytes($dll, $original)
    $checksum | Set-Content "$dll.sha256" -Encoding ascii
    $model = Join-Path $runtime 'model.bin'
    Set-Content $model 'synthetic model sentinel' -Encoding ascii
    Expect-Rejection 'language models must not be bundled'
    Remove-Item $model
    & $copy -SourceDirectory $runtime -DestinationDirectory $destination -Architecture $Architecture
    if ((Get-FileHash $dll).Hash -ne (Get-FileHash (Join-Path $destination 'translation/mypdf_ct2.dll')).Hash) {
        throw 'Packaged native runtime changed.'
    }
    Expect-Rejection 'destination already exists'
    Write-Host 'PASS native runtime packaging, exact copies, mandatory notices, checksums, and model exclusion.'
} finally { Remove-Item $scratch -Recurse -Force }
