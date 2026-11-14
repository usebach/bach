# End-to-end test of a real setup file: silent install, an update that hits
# a locked file (and must leave the old version intact), a real update, a
# refused downgrade, a damaged setup file, and the silent uninstall.
#
#   cargo build -p bach-cli -p bach-setup -p hello
#   powershell -ExecutionPolicy Bypass -File scripts\e2e.ps1 [-Profile debug|release]
#
# It installs a throwaway app ("Bach E2E", its own id) into a temp folder
# and removes it again; nothing else on the computer is touched.

param([string]$Profile = 'debug')
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$bach = Join-Path $root "target\$Profile\bach.exe"
$hello = Join-Path $root "target\$Profile\hello.exe"
foreach ($file in $bach, $hello) { if (-not (Test-Path $file)) { throw "Build first: $file is missing." } }

$work = Join-Path ([IO.Path]::GetTempPath()) ("bach-e2e-" + [Guid]::NewGuid().ToString('N').Substring(0, 8))
$install = Join-Path $work 'Installed\BachE2E'
$key = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\BachE2E'
$failures = 0

function Check([bool]$ok, [string]$what) {
    if ($ok) { Write-Host "  ok    $what" } else { Write-Host "  FAIL  $what" -ForegroundColor Red; $script:failures++ }
}

function Build([string]$version, [string[]]$extra) {
    $app = Join-Path $work "app-$version"
    New-Item -ItemType Directory -Force -Path $app | Out-Null
    Copy-Item $hello (Join-Path $app 'hello.exe')
    foreach ($name in $extra) { Set-Content -Path (Join-Path $app $name) -Value "$name $version" }
    @"
[app]
name = "Bach E2E"
id = "BachE2E"
version = "$version"
publisher = "Bach"
executable = "hello.exe"
[files]
source = "app-$version"
[installer]
output = "out/E2E-Setup-$version.exe"
desktop_shortcut = false
compression = "fast"
"@ | Set-Content -Encoding utf8 (Join-Path $work "bach-$version.toml")
    & $bach build (Join-Path $work "bach-$version.toml") | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "bach build $version failed." }
    return Join-Path $work "out\E2E-Setup-$version.exe"
}

function Run([string]$exe, [string[]]$arguments) {
    $process = Start-Process -FilePath $exe -ArgumentList $arguments -Wait -PassThru -WindowStyle Hidden
    return $process.ExitCode
}

function Installed-Version { (Get-ItemProperty -Path $key -ErrorAction SilentlyContinue).DisplayVersion }

if (Test-Path $key) { throw "BachE2E is already installed; remove it first." }
New-Item -ItemType Directory -Force -Path $work | Out-Null
try {
    $v1 = Build '1.0.0' @('readme.txt', 'old.txt')
    $v2 = Build '2.0.0' @('readme.txt', 'new.txt')

    Write-Host 'Install 1.0.0'
    Check ((Run $v1 @('--silent', '--dir', "`"$install`"")) -eq 0) 'exits 0'
    Check ((Installed-Version) -eq '1.0.0') 'listed in Settings > Apps as 1.0.0'
    Check (Test-Path (Join-Path $install 'uninstall.exe')) 'uninstaller written'
    Check (Test-Path (Join-Path $install 'old.txt')) 'files installed'

    Write-Host 'Update while a file is locked'
    $lock = [IO.File]::Open((Join-Path $install 'readme.txt'), 'Open', 'Read', 'None')
    try { $code = Run $v2 @('--silent') } finally { $lock.Close() }
    Check ($code -eq 1) 'exits 1'
    Check ((Installed-Version) -eq '1.0.0') 'still 1.0.0'
    Check ((Get-Content (Join-Path $install 'readme.txt')) -eq 'readme.txt 1.0.0') 'old file intact'
    Check (-not (Test-Path (Join-Path $install 'new.txt'))) 'no new file left behind'
    Check (-not (Get-ChildItem $install -Recurse -Filter '*.bach-*')) 'no .bach-new or .bach-old left'

    Write-Host 'Update to 2.0.0'
    Check ((Run $v2 @('--silent')) -eq 0) 'exits 0'
    Check ((Installed-Version) -eq '2.0.0') 'listed as 2.0.0'
    Check (-not (Test-Path (Join-Path $install 'old.txt'))) 'file 2.0.0 dropped is gone'
    Check (Test-Path (Join-Path $install 'new.txt')) 'new file installed'

    Write-Host 'Downgrade'
    Check ((Run $v1 @('--silent')) -eq 1) 'refused without --allow-downgrade'
    Check ((Installed-Version) -eq '2.0.0') 'still 2.0.0'

    Write-Host 'Damaged setup file'
    $damaged = Join-Path $work 'out\Damaged.exe'
    Copy-Item $v1 $damaged
    $bytes = [IO.File]::ReadAllBytes($damaged)
    $end = $bytes.Length
    $padding = [BitConverter]::ToUInt32($bytes, $end - 28)
    $archive = [BitConverter]::ToUInt64($bytes, $end - 24)
    $logo = [BitConverter]::ToUInt64($bytes, $end - 16)
    $manifest = [BitConverter]::ToUInt64($bytes, $end - 8)
    $start = $end - 40 - $padding - $manifest - $logo - $archive
    $bytes[[int]($start + $archive / 2)] = $bytes[[int]($start + $archive / 2)] -bxor 0xFF
    [IO.File]::WriteAllBytes($damaged, $bytes)
    Check ((Run $damaged @('--silent', '--allow-downgrade')) -eq 1) 'refused'
    Check ((Installed-Version) -eq '2.0.0') 'install untouched'

    Write-Host 'Uninstall'
    Set-Content -Path (Join-Path $install 'mine.txt') -Value 'a file the user made'
    Check ((Run (Join-Path $install 'uninstall.exe') @('--silent')) -eq 0) 'exits 0'
    Check (-not (Test-Path $key)) 'gone from Settings > Apps'
    Check (Test-Path (Join-Path $install 'mine.txt')) "the user's own file stays"
    Check (-not (Test-Path (Join-Path $install 'hello.exe'))) 'app files gone'
} finally {
    if (Test-Path $key) {
        $uninstaller = Join-Path $install 'uninstall.exe'
        if (Test-Path $uninstaller) { Run $uninstaller @('--silent') | Out-Null }
    }
    Start-Sleep -Seconds 3
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}

if ($failures -gt 0) { throw "$failures end-to-end check(s) failed." }
Write-Host 'All end-to-end checks passed.' -ForegroundColor Green
