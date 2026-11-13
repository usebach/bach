# Builds bach.exe and bach-setup.exe in release mode and packs them into
# dist\bach-<version>-windows-x64.zip, the download people use.
#
#   powershell -ExecutionPolicy Bypass -File scripts\build-release.ps1

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    cargo build --release -p bach-cli -p bach-setup
    if ($LASTEXITCODE -ne 0) { throw 'The release build failed.' }

    $metadata = cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    $version = ($metadata.packages | Where-Object { $_.name -eq 'bach-cli' }).version

    $stage = Join-Path 'dist' "bach-$version"
    if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
    New-Item -ItemType Directory -Force -Path $stage | Out-Null
    Copy-Item 'target\release\bach.exe', 'target\release\bach-setup.exe', 'LICENSE' -Destination $stage
    Copy-Item 'docs\getting-started.md' -Destination (Join-Path $stage 'README.md')

    $zip = Join-Path 'dist' "bach-$version-windows-x64.zip"
    if (Test-Path $zip) { Remove-Item -Force $zip }
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip
    Remove-Item -Recurse -Force $stage

    $size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
    Write-Host "Wrote $zip ($size MB)"
}
finally {
    Pop-Location
}
