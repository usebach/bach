# Installs Bach for the current user, no Rust or admin rights needed:
#
#   irm https://raw.githubusercontent.com/usebach/bach/main/install.ps1 | iex
#
# Downloads the latest release (or $env:BACH_VERSION, like "0.2.0"), checks
# its SHA-256, unpacks bach.exe and bach-setup.exe into
# %LOCALAPPDATA%\Programs\Bach and adds that folder to your PATH. Run it
# again to update. Nothing else on your computer changes.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = 'usebach/bach'
$target = Join-Path $env:LOCALAPPDATA 'Programs\Bach'

$api = if ($env:BACH_VERSION) {
    "https://api.github.com/repos/$repo/releases/tags/v$($env:BACH_VERSION.TrimStart('v'))"
} else {
    "https://api.github.com/repos/$repo/releases/latest"
}
try {
    $release = Invoke-RestMethod -UseBasicParsing -Uri $api -Headers @{ 'User-Agent' = 'bach-install' }
} catch {
    throw "Couldn't reach GitHub to find the Bach release ($api). Check your connection, or download the zip from https://github.com/$repo/releases."
}
$zip = $release.assets | Where-Object { $_.name -like 'bach-*-windows-x64.zip' } | Select-Object -First 1
if (-not $zip) { throw "Release $($release.tag_name) has no Windows download." }
$sum = $release.assets | Where-Object { $_.name -eq "$($zip.name).sha256" } | Select-Object -First 1
if (-not $sum) { throw "Release $($release.tag_name) has no checksum for $($zip.name), so it isn't installed." }

$work = Join-Path ([IO.Path]::GetTempPath()) ("bach-install-" + [Guid]::NewGuid())
New-Item -ItemType Directory -Path $work | Out-Null
try {
    $zipPath = Join-Path $work $zip.name
    Write-Host "Downloading $($zip.name)"
    Invoke-WebRequest -UseBasicParsing -Uri $zip.browser_download_url -OutFile $zipPath
    $expected = ((Invoke-WebRequest -UseBasicParsing -Uri $sum.browser_download_url).Content -split '\s+')[0].Trim().ToLower()
    $actual = (Get-FileHash -Algorithm SHA256 $zipPath).Hash.ToLower()
    if ($expected -ne $actual) { throw "The download doesn't match its published SHA-256; nothing was installed." }

    $unpacked = Join-Path $work 'unpacked'
    Expand-Archive -Path $zipPath -DestinationPath $unpacked
    New-Item -ItemType Directory -Force -Path $target | Out-Null
    foreach ($file in 'bach.exe', 'bach-setup.exe') {
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

# Add the folder to the user's PATH, keeping the value's type so entries
# like %USERPROFILE%\bin stay as they are.
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
$env:Path = "$env:Path;$target"

& (Join-Path $target 'bach.exe') --version
Write-Host "Bach is installed in $target. Start with: bach init"
