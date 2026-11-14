# Installs Bach for the current user, no Rust or admin rights needed:
#
#   irm https://raw.githubusercontent.com/usebach/bach/main/install.ps1 | iex
#
# Downloads the latest release (or $env:BACH_VERSION: "0.2.0" for exactly
# that one, "0.2" or "0" for the newest of that line, "latest" for the
# newest), checks its SHA-256, unpacks bach.exe and bach-setup.exe into
# %LOCALAPPDATA%\Programs\bach-cli (or $env:BACH_INSTALL_DIR) and adds that
# folder to your PATH. Run
# it again to update. Nothing else on your computer changes: if that folder
# holds anything but Bach's own two files, it stops instead of writing.
#
# It never asks for administrator rights unless you ask for a system-wide
# install: set $env:BACH_SCOPE = 'system' first. Then bach goes to
# %ProgramFiles%\bach-cli for every user, and Windows' own prompt (UAC) asks
# for permission once, for that copy only. If you decline, Bach is installed
# for you alone, as without BACH_SCOPE.

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$repo = 'usebach/bach'
$scope = "$($env:BACH_SCOPE)".Trim().ToLower()
if (-not $scope) { $scope = 'user' }
if ($scope -ne 'user' -and $scope -ne 'system') { throw "BACH_SCOPE must be user or system, not $scope." }
if ($env:BACH_INSTALL_DIR) { $customTarget = $env:BACH_INSTALL_DIR }
function Get-Target {
    # Its own folder: Programs\Bach may be another app called Bach, and
    # Windows file names ignore case, so bach.exe would replace Bach.exe.
    if ($customTarget) { return $customTarget }
    if ($scope -eq 'system') { return (Join-Path $env:ProgramFiles 'bach-cli') }
    return (Join-Path $env:LOCALAPPDATA 'Programs\bach-cli')
}
$target = Get-Target

# A system-wide install writes to Program Files and the machine's PATH, which
# takes administrator rights. Ask for them the way Windows does (UAC), run
# only this script with them, and carry on for the user alone if they're
# refused.
$isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
if ($scope -eq 'system' -and -not $isAdmin) {
    $safe = '^[\w .:\\/()-]*$'
    foreach ($name in 'BACH_VERSION', 'BACH_INSTALL_DIR', 'BACH_NO_PATH') {
        if ("$([Environment]::GetEnvironmentVariable($name))" -notmatch $safe) { throw "$name has characters that can't be handed to an elevated install." }
    }
    # The elevated copy is given this script's text itself (compressed into its
    # command line), never the path of a file: nothing in a folder the user can
    # write to is run with administrator rights, so there is no file to swap
    # between the prompt and the run. Run from the web (irm | iex) there is no
    # file, so the same script is fetched, into memory.
    $text = if ($PSCommandPath -and (Test-Path $PSCommandPath)) {
        Get-Content -Raw -Path $PSCommandPath
    } else {
        (Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/$repo/main/install.ps1").Content
    }
    $bytes = [Text.Encoding]::UTF8.GetBytes($text)
    $memory = New-Object IO.MemoryStream
    $zip = New-Object IO.Compression.GZipStream($memory, [IO.Compression.CompressionMode]::Compress)
    $zip.Write($bytes, 0, $bytes.Length)
    $zip.Close()
    $packed = [Convert]::ToBase64String($memory.ToArray())
    $settings = ('BACH_VERSION', 'BACH_INSTALL_DIR', 'BACH_NO_PATH' | ForEach-Object {
        $value = [Environment]::GetEnvironmentVariable($_)
        if ($value) { "`$env:$_ = '$($value -replace "'", "''")'; " }
    }) -join ''
    # The window stays up on an error, so it can be read; the parent checks the result.
    $command = "$settings`$env:BACH_SCOPE = 'system'; try { `$in = New-Object IO.MemoryStream(,[Convert]::FromBase64String('$packed')); `$text = (New-Object IO.StreamReader((New-Object IO.Compression.GZipStream(`$in, [IO.Compression.CompressionMode]::Decompress)), [Text.Encoding]::UTF8)).ReadToEnd(); Invoke-Expression `$text } catch { Write-Host `$_.Exception.Message; Read-Host 'Press Enter to close'; exit 1 }"
    $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($command))
    Write-Host "Installing for every user needs administrator rights. Windows will ask for permission."
    $elevated = $null
    try {
        $shell = (Get-Process -Id $PID).Path
        $elevated = Start-Process -FilePath $shell -Verb RunAs -Wait -PassThru -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-EncodedCommand', $encoded
    } catch {
        Write-Host "No administrator rights, so Bach is installed just for you instead."
        $scope = 'user'
        $target = Get-Target
    }
    if ($elevated) {
        if ($elevated.ExitCode -ne 0 -or -not (Test-Path (Join-Path $target 'bach.exe'))) {
            throw "The system-wide install didn't finish (exit code $($elevated.ExitCode)); nothing was changed for your user."
        }
        $env:Path = "$env:Path;$target"
        Write-Host "Bach is installed in $target for every user. Start with: bach init"
        return
    }
}
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

$wanted = "$($env:BACH_VERSION)".Trim().TrimStart('v', 'V')
if ($wanted -ieq 'latest') { $wanted = '' }
$headers = @{ 'User-Agent' = 'bach-install' }
if ($env:GITHUB_TOKEN) { $headers['Authorization'] = "Bearer $($env:GITHUB_TOKEN)" }
$release = $null

if ($wanted -match '^\d+(\.\d+)?$') {
    # "0" or "0.2": the newest release of that line. 0.2 is 0.2.0, 0.2.1, ...
    # and never 0.20; a pre-release is only taken when the line has no other.
    try {
        # Piped, so Windows PowerShell hands the releases over one by one.
        $all = @(Invoke-RestMethod -UseBasicParsing -Uri "https://api.github.com/repos/$repo/releases?per_page=100" -Headers $headers | ForEach-Object { $_ })
    } catch {
        throw "Couldn't reach GitHub to find Bach $wanted ($($_.Exception.Message)). Check your connection, or download the zip from https://github.com/$repo/releases."
    }
    $line = '^v' + [regex]::Escape($wanted) + '(\.|$)'
    $matching = @($all | Where-Object { -not $_.draft -and $_.tag_name -match $line } | ForEach-Object {
        $parsed = $null
        if ([version]::TryParse(($_.tag_name -replace '^v', '' -replace '-.*$', ''), [ref]$parsed)) {
            [pscustomobject]@{ Release = $_; Version = $parsed }
        }
    })
    $stable = @($matching | Where-Object { -not $_.Release.prerelease })
    $pick = if ($stable) { $stable } else { $matching }
    $release = ($pick | Sort-Object Version -Descending | Select-Object -First 1).Release
    if (-not $release) { throw "Bach has no release $wanted.x. See https://github.com/$repo/releases for the versions there are." }
    Write-Host "Bach $wanted resolves to $($release.tag_name)"
} else {
    $api = if ($wanted) {
        "https://api.github.com/repos/$repo/releases/tags/v$wanted"
    } else {
        "https://api.github.com/repos/$repo/releases/latest"
    }
    try {
        $release = Invoke-RestMethod -UseBasicParsing -Uri $api -Headers $headers
    } catch {
        # "latest" skips pre-releases; while there are only those, take the newest.
        if (-not $wanted) {
            try {
                $release = Invoke-RestMethod -UseBasicParsing -Uri "https://api.github.com/repos/$repo/releases?per_page=1" -Headers $headers |
                    Select-Object -First 1
            } catch {}
        }
        if (-not $release) {
            throw "Couldn't find the Bach release ($api). Check your connection and the version, or download the zip from https://github.com/$repo/releases."
        }
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

# Add the folder to the PATH (unless BACH_NO_PATH is set, as in CI): the
# machine's for a system-wide install, otherwise the user's. The value's type
# is kept, so entries like %USERPROFILE%\bin stay as they are.
if (-not $env:BACH_NO_PATH) {
    $key = if ($scope -eq 'system') {
        [Microsoft.Win32.Registry]::LocalMachine.OpenSubKey('SYSTEM\CurrentControlSet\Control\Session Manager\Environment', $true)
    } else {
        [Microsoft.Win32.Registry]::CurrentUser.OpenSubKey('Environment', $true)
    }
    $path = [string]$key.GetValue('Path', '', 'DoNotExpandEnvironmentNames')
    $kind = if ($key.GetValueNames() -contains 'Path') { $key.GetValueKind('Path') } else { [Microsoft.Win32.RegistryValueKind]::ExpandString }
    $entries = $path -split ';' | Where-Object { $_ -ne '' }
    if (-not ($entries | Where-Object { $_.TrimEnd('\') -ieq $target })) {
        $new = if ($path -eq '' -or $path.EndsWith(';')) { "$path$target" } else { "$path;$target" }
        $key.SetValue('Path', $new, $kind)
        # Tells Explorer the environment changed, so new terminals see it.
        [Environment]::SetEnvironmentVariable('BACH_INSTALL_REFRESH', $null, $(if ($scope -eq 'system') { 'Machine' } else { 'User' }))
        Write-Host "Added $target to $(if ($scope -eq 'system') { 'the system' } else { 'your' }) PATH. Open a new terminal to use it."
    }
    $key.Close()
}
$env:Path = "$env:Path;$target"

& (Join-Path $target 'bach.exe') --version
Write-Host "Bach is installed in $target. Start with: bach init"
