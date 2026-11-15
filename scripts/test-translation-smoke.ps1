# Separate preparation, network access and native inference for useful CI timeouts.
# The model stays outside the packaged app; every input sentence is synthetic.
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][ValidateSet('Build', 'Download', 'Translate')][string]$Stage,
    [ValidateSet('x64', 'arm64')][string]$Architecture = 'arm64'
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path $PSScriptRoot -Parent
Push-Location $repo
try {
    $env:MYPDF_CT2_LIBRARY = Join-Path $repo "dist/translation/windows-$Architecture/mypdf_ct2.dll"
    $env:MYPDF_CT2_TEST_ROOT = Join-Path $repo 'artifacts/Translation Türkçe review'
    $env:MYPDF_CT2_DOWNLOAD_TEST = '1'
    switch ($Stage) {
        'Build' { cargo test --locked --release --lib --no-run }
        'Download' { cargo test --locked --release --lib translation::ctranslate::tests::explicit_language_download_verifies_and_installs_the_approved_pack -- --ignored --nocapture }
        'Translate' { cargo test --locked --release --lib translation::ctranslate::tests::native_translation_smoke_downloaded_pack -- --ignored --nocapture }
    }
    if ($LASTEXITCODE -ne 0) { throw "Native translation smoke stage failed: $Stage" }
} finally { Pop-Location }
