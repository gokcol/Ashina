# Produces the native offline runtime. Models are downloaded separately only when
# the user requests a language; none are bundled into the installer.
[CmdletBinding()]
param(
    [ValidateSet('x64', 'arm64')][string]$Architecture = 'x64',
    [string]$OutputDirectory = '',
    [string]$BuildDirectory = '',
    [string]$Generator = 'Visual Studio 17 2022',
    [string]$Toolset = ''
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
if ($env:OS -ne 'Windows_NT') { throw 'Build the Windows native runtime on Windows with Visual Studio C++ tools.' }
$repo = Split-Path $PSScriptRoot -Parent
Push-Location $repo
try {
    if ($Generator -eq 'auto') {
        $vswhere = "${env:ProgramFiles(x86)}/Microsoft Visual Studio/Installer/vswhere.exe"
        $version = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.ARM64 -property installationVersion
        $Generator = switch (([version]$version).Major) {
            17 { 'Visual Studio 17 2022' }
            18 { 'Visual Studio 18 2026' }
            default { throw "Unsupported Visual Studio version: $version" }
        }
    }
    if (-not $OutputDirectory) { $OutputDirectory = "dist/translation/windows-$Architecture" }
    if (-not $BuildDirectory) { $BuildDirectory = "target/translation-windows-$Architecture" }
    $output = [System.IO.Path]::GetFullPath($OutputDirectory)
    $platform = if ($Architecture -eq 'arm64') { 'ARM64' } else { 'x64' }
    $options = @('-DCMAKE_POLICY_VERSION_MINIMUM=3.5')
    if ($Toolset) { $options += @('-T', $Toolset) }
    & cmake -S native/translation-ct2 -B $BuildDirectory -G $Generator -A $platform "-DCMAKE_INSTALL_PREFIX=$output" @options
    if ($LASTEXITCODE -ne 0) { throw 'Translation CMake configuration failed.' }
    & cmake --build $BuildDirectory --config Release --target mypdf_ct2 --parallel 4
    if ($LASTEXITCODE -ne 0) { throw 'Translation native build failed.' }
    & cmake --install $BuildDirectory --config Release --component MyPDFTranslation
    if ($LASTEXITCODE -ne 0) { throw 'Translation runtime installation failed.' }
    $dll = Join-Path $output 'mypdf_ct2.dll'
    if (-not (Test-Path $dll)) { throw 'The translation runtime DLL is missing.' }
    $hash = (Get-FileHash $dll -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  mypdf_ct2.dll" | Out-File -Encoding ascii (Join-Path $output 'mypdf_ct2.dll.sha256')
    Write-Host "Native offline translation runtime: $output"
} finally { Pop-Location }
