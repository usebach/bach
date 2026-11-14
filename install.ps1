# Installs Bach for the current user, no Rust or admin rights needed:
#
#   irm https://raw.githubusercontent.com/usebach/bach/main/install.ps1 | iex
#
# Downloads the latest release (or $env:BACH_VERSION, like "0.2.0"), checks
# its SHA-256, unpacks bach.exe and bach-setup.exe into
# %LOCALAPPDATA%\Programs\bach-cli (or $env:BACH_INSTALL_DIR) and adds that
# folder to your PATH. Run
# it again to update. Nothing else on your computer changes: if that folder
# holds anything but Bach's own two files, it stops instead of writing.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = 'usebach/bach'
if ($env:BACH_INSTALL_DIR) { $customTarget = $env:BACH_INSTALL_DIR }
# Its own folder: Programs\Bach may be another app called Bach, and
# Windows file names ignore case, so bach.exe would replace Bach.exe.
$target = if ($customTarget) { $customTarget } else { Join-Path $env:LOCALAPPDATA 'Programs\bach-cli' }
$ours = @('bach.exe', 'bach-setup.exe')
# This file marks a folder as this script's own. A folder that has files but
# no mark belongs to something else (even a lone Bach.exe), and stays as is.
$mark = '.bach-cli'
if (Test-Path $target) {
    $contents = @(Get-ChildItem -Force -Path $target)
    $marked = Test-Path (Join-Path $target $mark)
    $foreign = $contents | Where-Object { ($ours + $mark) -notcontains $_.Name.ToLower() }
    if ($foreign -or ($contents.Count -gt 0 -and -not $marked)) {
        $example = if ($foreign) { $foreign[0].Name } else { $contents[0].Name }
        throw "$target already holds files that aren't from this script ($example, ...), so nothing was installed. Set BACH_INSTALL_DIR to another folder."
    }
}

$api = if ($env:BACH_VERSION) {
    "https://api.github.com/repos/$repo/releases/tags/v$($env:BACH_VERSION.TrimStart('v'))"
} else {
    "https://api.github.com/repos/$repo/releases/latest"
}
$headers = @{ 'User-Agent' = 'bach-install' }
try {
    $release = Invoke-RestMethod -UseBasicParsing -Uri $api -Headers $headers
} catch {
    $release = $null
    # "latest" skips pre-releases; while there are only those, take the newest.
    if (-not $env:BACH_VERSION) {
        try {
            $release = Invoke-RestMethod -UseBasicParsing -Uri "https://api.github.com/repos/$repo/releases?per_page=1" -Headers $headers |
                Select-Object -First 1
        } catch {}
    }
    if (-not $release) {
        throw "Couldn't reach GitHub to find the Bach release ($api). Check your connection, or download the zip from https://github.com/$repo/releases."
    }
}
# Windows on ARM gets its own build when the release has one; the x64 build
# runs there too.
$zip = $null
if ($env:PROCESSOR_ARCHITECTURE -eq 'ARM64') {
    $zip = $release.assets | Where-Object { $_.name -like 'bach-*-windows-arm64.zip' } | Select-Object -First 1
}
if (-not $zip) {
    $zip = $release.assets | Where-Object { $_.name -like 'bach-*-windows-x64.zip' } | Select-Object -First 1
}
if (-not $zip) { throw "Release $($release.tag_name) has no Windows download." }
$sum = $release.assets | Where-Object { $_.name -eq "$($zip.name).sha256" } | Select-Object -First 1
if (-not $sum) { throw "Release $($release.tag_name) has no checksum for $($zip.name), so it isn't installed." }

$work = Join-Path ([IO.Path]::GetTempPath()) ("bach-install-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
try {
    $zipPath = Join-Path $work $zip.name
    Write-Host "Downloading $($zip.name)"
    Invoke-WebRequest -UseBasicParsing -Uri $zip.browser_download_url -OutFile $zipPath
    # Saved to a file: Windows PowerShell hands back binary downloads as bytes.
    $sumPath = Join-Path $work "$($zip.name).sha256"
    Invoke-WebRequest -UseBasicParsing -Uri $sum.browser_download_url -OutFile $sumPath
    $expected = ((Get-Content -Raw $sumPath).Trim() -split '\s+')[0].ToLower()
    $actual = (Get-FileHash -Algorithm SHA256 $zipPath).Hash.ToLower()
    if ($expected -ne $actual) { throw "The download doesn't match its published SHA-256; nothing was installed." }

    $unpacked = Join-Path $work 'unpacked'
    Expand-Archive -Path $zipPath -DestinationPath $unpacked
    New-Item -ItemType Directory -Force -Path $target | Out-Null
    Set-Content -Path (Join-Path $target $mark) -Value 'Installed by https://github.com/usebach/bach/blob/main/install.ps1'
    foreach ($file in $ours) {
        $from = Get-ChildItem -Path $unpacked -Recurse -Filter $file | Select-Object -First 1
        if (-not $from) { throw "The download has no $file." }
        try {
            Copy-Item -Force $from.FullName (Join-Path $target $file)
        } catch {
            throw "Couldn't replace $file in $target. Close any running bach, then run this again."
        }
    }
} finally {
    Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
}

# Add the folder to the user's PATH (unless BACH_NO_PATH is set, as in CI),
# keeping the value's type so entries like %USERPROFILE%\bin stay as they are.
if (-not $env:BACH_NO_PATH) {
    $key = [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    $path = [string]$key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
    $kind = if ($key.GetValueNames() -contains 'Path') { $key.GetValueKind('Path') } else { [Microsoft.Win32.RegistryValueKind]::ExpandString }
    $entries = $path -split ';' | Where-Object { $_ -ne '' }
    if (-not ($entries | Where-Object { $_.TrimEnd('\') -ieq $target })) {
        $new = if ($path -eq '' -or $path.EndsWith(';')) { "$path$target" } else { "$path;$target" }
        $key.SetValue('Path', $new, $kind)
        # Tells Explorer the environment changed, so new terminals see it.
        [Environment]::SetEnvironmentVariable('BACH_INSTALL_REFRESH', $null, 'User')
        Write-Host "Added $target to your PATH. Open a new terminal to use it."
    }
    $key.Close()
}
$env:Path = "$env:Path;$target"

& (Join-Path $target 'bach.exe') --version
Write-Host "Bach is installed in $target. Start with: bach init"
