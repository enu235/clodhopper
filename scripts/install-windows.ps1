param(
    [string]$InstallDir = $(if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $HOME ".local\bin" }),
    [switch]$Help
)

if ($Help) {
    Write-Host @"
Install the Clodhopper harness binary from an extracted release package.

Usage:
  .\install.ps1 [-InstallDir INSTALL_DIR]

Examples:
  .\install.ps1
  .\install.ps1 -InstallDir "$HOME\bin"
"@
    exit 0
}

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Binary = Join-Path $ScriptDir "bin\harness.exe"

if (!(Test-Path $Binary)) {
    Write-Error "expected release binary at $Binary. Extract the release archive, then run install.ps1 from the extracted directory."
    exit 1
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$Destination = Join-Path $InstallDir "harness.exe"
Copy-Item -Force $Binary $Destination

Write-Host "installed harness to $Destination"

$PathEntries = $env:PATH -split ';'
if ($PathEntries -notcontains $InstallDir) {
    Write-Warning "$InstallDir is not on PATH"
    Write-Host "Add it to your user PATH if needed."
}

& $Destination --help | Out-Null
if ($LASTEXITCODE -ne 0) {
    Write-Error "installed harness did not run successfully"
    exit $LASTEXITCODE
}

Write-Host "verified harness executable"
Write-Host "next: harness doctor"

