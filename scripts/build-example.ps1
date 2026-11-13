# Builds the example installer, dist\Hello-Setup-1.0.0.exe, end to end:
# the release binaries, the tiny Hello app, and the setup file around it.
#
#   powershell -ExecutionPolicy Bypass -File scripts\build-example.ps1

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    cargo build --release -p bach-cli -p bach-setup -p hello
    if ($LASTEXITCODE -ne 0) { throw 'The release build failed.' }

    $app = 'examples\hello\app'
    New-Item -ItemType Directory -Force -Path $app | Out-Null
    Copy-Item -Force 'target\release\hello.exe' -Destination $app

    & 'target\release\bach.exe' build 'examples\hello\bach.toml'
    if ($LASTEXITCODE -ne 0) { throw 'bach build failed.' }
}
finally {
    Pop-Location
}
