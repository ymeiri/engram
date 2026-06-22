Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $RepoRoot

$DefaultDistDir = Join-Path $RepoRoot "dist"
$DistDir = if ($env:DIST_DIR) { $env:DIST_DIR } else { $DefaultDistDir }
$AllowDistDirOverride = if ($env:ALLOW_PACKAGE_DIST_DIR_OVERRIDE) {
    $env:ALLOW_PACKAGE_DIST_DIR_OVERRIDE
} else {
    "0"
}
$SkipPackageBuild = if ($env:SKIP_PACKAGE_BUILD) { $env:SKIP_PACKAGE_BUILD } else { "0" }
$AllowPackageBuildSkip = if ($env:ALLOW_PACKAGE_BUILD_SKIP) {
    $env:ALLOW_PACKAGE_BUILD_SKIP
} else {
    "0"
}
$AllowPackageIdentityOverride = if ($env:ALLOW_PACKAGE_IDENTITY_OVERRIDE) {
    $env:ALLOW_PACKAGE_IDENTITY_OVERRIDE
} else {
    "0"
}
$EmbedCacheDir = if ($env:ENGRAM_EMBED_CACHE_DIR) {
    $env:ENGRAM_EMBED_CACHE_DIR
} else {
    Join-Path $RepoRoot ".fastembed_cache"
}

function Require-Binary {
    param([string] $Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "required tool is missing: $Name"
    }
}

function Assert-Flag {
    param([string] $Name, [string] $Value)
    if ($Value -ne "0" -and $Value -ne "1") {
        throw "$Name must be 0 or 1, got $Value"
    }
}

function Sha256-File {
    param([string] $Path)
    (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLowerInvariant()
}

function Run-Step {
    param([string] $Name, [scriptblock] $Command)
    Write-Host ""
    Write-Host "==> $Name"
    & $Command
}

function Test-PortInUse {
    param([int] $Port)
    $Client = [System.Net.Sockets.TcpClient]::new()
    try {
        $Async = $Client.BeginConnect("127.0.0.1", $Port, $null, $null)
        if (-not $Async.AsyncWaitHandle.WaitOne(200)) {
            return $false
        }
        $Client.EndConnect($Async)
        return $true
    } catch {
        return $false
    } finally {
        $Client.Close()
    }
}

function Choose-Port {
    if ($env:SMOKE_PORT) {
        if ($env:SMOKE_PORT -notmatch "^[0-9]+$") {
            throw "SMOKE_PORT must be numeric, got $env:SMOKE_PORT"
        }
        $Port = [int] $env:SMOKE_PORT
        if ($Port -lt 1 -or $Port -gt 65535) {
            throw "SMOKE_PORT must be between 1 and 65535, got $Port"
        }
        if (Test-PortInUse $Port) {
            throw "SMOKE_PORT is already in use on 127.0.0.1: $Port"
        }
        return $Port
    }
    foreach ($Port in 8765..8774) {
        if (-not (Test-PortInUse $Port)) {
            return $Port
        }
    }
    throw "no free local smoke-test port found in 8765-8774"
}

Require-Binary "cargo"
Require-Binary "rustc"
Require-Binary "git"
Assert-Flag "ALLOW_PACKAGE_DIST_DIR_OVERRIDE" $AllowDistDirOverride
Assert-Flag "SKIP_PACKAGE_BUILD" $SkipPackageBuild
Assert-Flag "ALLOW_PACKAGE_BUILD_SKIP" $AllowPackageBuildSkip
Assert-Flag "ALLOW_PACKAGE_IDENTITY_OVERRIDE" $AllowPackageIdentityOverride

if ($SkipPackageBuild -eq "1" -and $AllowPackageBuildSkip -ne "1") {
    throw "SKIP_PACKAGE_BUILD=1 requires explicit package build-skip approval"
}

$ResolvedDistDir = [System.IO.Path]::GetFullPath($DistDir)
$ResolvedDefaultDistDir = [System.IO.Path]::GetFullPath($DefaultDistDir)
if ($ResolvedDistDir -ne $ResolvedDefaultDistDir -and $AllowDistDirOverride -ne "1") {
    throw "DIST_DIR override requires explicit package approval; expected $ResolvedDefaultDistDir, got $ResolvedDistDir"
}

$PackageId = (& cargo pkgid --locked -p engram-cli)
if ($LASTEXITCODE -ne 0) {
    throw "could not determine workspace package version for engram-cli"
}
$PackageVersion = ($PackageId -split "#")[-1]
if ($PackageVersion -notmatch "^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9][A-Za-z0-9.-]*)?$") {
    throw "workspace package version must be x.y.z with an optional prerelease suffix, got $PackageVersion"
}

$HostTriple = (& rustc -vV | Select-String "^host: " | ForEach-Object {
    $_.Line.Substring("host: ".Length)
})
if ([string]::IsNullOrWhiteSpace($HostTriple)) {
    throw "host triple could not be determined from rustc -vV"
}
if ($HostTriple -notmatch "pc-windows-msvc$") {
    throw "package-install-smoke-windows.ps1 must run on a Windows MSVC Rust host, got $HostTriple"
}

$ArchiveName = "engram-$PackageVersion-$HostTriple"
$Archive = Join-Path $ResolvedDistDir "$ArchiveName.zip"
$Checksum = "$Archive.sha256"
$DefaultExpectedGitHead = (& git -C $RepoRoot rev-parse HEAD).Trim()
$DefaultCargoLockSha256 = Sha256-File (Join-Path $RepoRoot "Cargo.lock")
$ExpectedGitHead = if ($env:EXPECTED_PACKAGE_GIT_HEAD) {
    $env:EXPECTED_PACKAGE_GIT_HEAD
} else {
    $DefaultExpectedGitHead
}
$ExpectedCargoLockSha256 = if ($env:EXPECTED_CARGO_LOCK_SHA256) {
    $env:EXPECTED_CARGO_LOCK_SHA256
} else {
    $DefaultCargoLockSha256
}
if ($ExpectedGitHead -ne $DefaultExpectedGitHead -and $AllowPackageIdentityOverride -ne "1") {
    throw "EXPECTED_PACKAGE_GIT_HEAD override requires explicit package identity approval"
}
if ($ExpectedCargoLockSha256 -ne $DefaultCargoLockSha256 -and $AllowPackageIdentityOverride -ne "1") {
    throw "EXPECTED_CARGO_LOCK_SHA256 override requires explicit package identity approval"
}

if ($env:EXPECTED_TRACKED_CHANGES_PRESENT) {
    $ExpectedTrackedChangesPresent = $env:EXPECTED_TRACKED_CHANGES_PRESENT
} else {
    & git -C $RepoRoot diff --quiet --ignore-submodules --
    $WorktreeClean = ($LASTEXITCODE -eq 0)
    & git -C $RepoRoot diff --cached --quiet --ignore-submodules --
    $IndexClean = ($LASTEXITCODE -eq 0)
    $ExpectedTrackedChangesPresent = if ($WorktreeClean -and $IndexClean) { "false" } else { "true" }
}
if ($ExpectedTrackedChangesPresent -ne "true" -and $ExpectedTrackedChangesPresent -ne "false") {
    throw "EXPECTED_TRACKED_CHANGES_PRESENT must be true or false, got $ExpectedTrackedChangesPresent"
}

if ($SkipPackageBuild -ne "1") {
    Run-Step "build release package" {
        if ($ResolvedDistDir -ne $ResolvedDefaultDistDir) {
            $env:ALLOW_PACKAGE_DIST_DIR_OVERRIDE = "1"
        }
        & (Join-Path $RepoRoot "scripts/package-release-windows.ps1")
        if ($LASTEXITCODE -ne 0) {
            throw "Windows release package build failed"
        }
    }
}

if (-not (Test-Path $Archive)) {
    throw "release zip not found at $Archive"
}
if (-not (Test-Path $Checksum)) {
    throw "release checksum not found at $Checksum"
}

$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) "engram-install-smoke-$([guid]::NewGuid().ToString('N'))"
$ServerProcess = $null
try {
    New-Item -ItemType Directory -Path $WorkDir, $EmbedCacheDir -Force | Out-Null
    Copy-Item $Archive, $Checksum $WorkDir
    Set-Location $WorkDir

    $ArchiveLeaf = Split-Path -Leaf $Archive
    $ChecksumLeaf = Split-Path -Leaf $Checksum

    Run-Step "inspect checksum file" {
        $ChecksumLines = @(Get-Content $ChecksumLeaf)
        if ($ChecksumLines.Count -ne 1) {
            throw "checksum file must contain exactly one line: $ChecksumLeaf"
        }
        $Parts = $ChecksumLines[0] -split "\s+"
        if ($Parts.Count -ne 2) {
            throw "checksum file must contain digest and filename only: $ChecksumLeaf"
        }
        if ($Parts[0] -notmatch "^[0-9a-f]{64}$") {
            throw "checksum digest is not a SHA-256 hex value: $ChecksumLeaf"
        }
        if ($Parts[1] -ne $ArchiveLeaf) {
            throw "checksum filename mismatch: expected $ArchiveLeaf, got $($Parts[1])"
        }
        $ActualDigest = Sha256-File $ArchiveLeaf
        if ($ActualDigest -ne $Parts[0]) {
            throw "checksum digest mismatch for $ArchiveLeaf"
        }
    }

    Run-Step "extract archive" {
        Expand-Archive -Path $ArchiveLeaf -DestinationPath $WorkDir -Force
    }

    $PackageDir = Join-Path $WorkDir $ArchiveName
    foreach ($RequiredPath in @(
        "engram.exe",
        "README.md",
        "LICENSE",
        "CHANGELOG.md",
        "RELEASE_NOTES.md",
        "MANIFEST.json"
    )) {
        $FullPath = Join-Path $PackageDir $RequiredPath
        if (-not (Test-Path $FullPath)) {
            throw "expected packaged file is missing: $FullPath"
        }
        if ((Get-Item $FullPath).Length -le 0) {
            throw "expected packaged file is empty: $FullPath"
        }
    }

    $Manifest = Get-Content (Join-Path $PackageDir "MANIFEST.json") -Raw | ConvertFrom-Json
    if ($Manifest.package -ne "engram") {
        throw "manifest package mismatch: expected engram, got $($Manifest.package)"
    }
    if ($Manifest.version -ne $PackageVersion) {
        throw "manifest version mismatch: expected $PackageVersion, got $($Manifest.version)"
    }
    if ($Manifest.host_triple -ne $HostTriple) {
        throw "manifest host triple mismatch: expected $HostTriple, got $($Manifest.host_triple)"
    }
    if ($Manifest.archive_name -ne $ArchiveName) {
        throw "manifest archive name mismatch: expected $ArchiveName, got $($Manifest.archive_name)"
    }
    if ($Manifest.git_head -ne $ExpectedGitHead) {
        throw "manifest git head mismatch: expected $ExpectedGitHead, got $($Manifest.git_head)"
    }
    $ManifestTrackedChangesPresent = $Manifest.tracked_changes_present.ToString().ToLowerInvariant()
    if ($ManifestTrackedChangesPresent -ne $ExpectedTrackedChangesPresent) {
        throw "manifest tracked-changes flag mismatch: expected $ExpectedTrackedChangesPresent, got $($Manifest.tracked_changes_present)"
    }
    if ($Manifest.cargo_lock_sha256 -ne $ExpectedCargoLockSha256) {
        throw "manifest Cargo.lock hash mismatch"
    }
    foreach ($PackageFile in @("engram.exe", "README.md", "LICENSE", "CHANGELOG.md", "RELEASE_NOTES.md")) {
        $Entry = $Manifest.files | Where-Object { $_.path -eq $PackageFile } | Select-Object -First 1
        if (-not $Entry) {
            throw "manifest is missing SHA-256 entry for $PackageFile"
        }
        $ActualSha = Sha256-File (Join-Path $PackageDir $PackageFile)
        if ($Entry.sha256 -ne $ActualSha) {
            throw "manifest hash mismatch for $PackageFile"
        }
    }

    $PrefixBin = Join-Path $WorkDir "prefix/bin"
    $HomeDir = Join-Path $WorkDir "home"
    $DataDir = Join-Path $WorkDir "data"
    New-Item -ItemType Directory -Path $PrefixBin, $HomeDir, $DataDir -Force | Out-Null
    $InstalledEngram = Join-Path $PrefixBin "engram.exe"
    Run-Step "install binary in temp prefix" {
        Copy-Item (Join-Path $PackageDir "engram.exe") $InstalledEngram
    }

    $env:PATH = "$PrefixBin;$env:PATH"
    $env:HOME = $HomeDir
    $env:ENGRAM_DATA_DIR = $DataDir
    $env:ENGRAM_EMBED_CACHE_DIR = $EmbedCacheDir

    $ExpectedVersion = "engram $PackageVersion"
    $ActualVersion = (& $InstalledEngram --version).Trim()
    if ($ActualVersion -ne $ExpectedVersion) {
        throw "installed binary version mismatch: expected '$ExpectedVersion', got '$ActualVersion'"
    }

    $Port = Choose-Port
    $ServerOut = Join-Path $WorkDir "server.out.log"
    $ServerErr = Join-Path $WorkDir "server.err.log"

    Write-Host ""
    Write-Host "==> start packaged HTTP server"
    $ServerProcess = Start-Process `
        -FilePath $InstalledEngram `
        -ArgumentList @("serve", "--http", "--memory", "--port", "$Port") `
        -NoNewWindow `
        -PassThru `
        -RedirectStandardOutput $ServerOut `
        -RedirectStandardError $ServerErr

    $Health = $null
    foreach ($Attempt in 1..300) {
        if ($ServerProcess.HasExited) {
            Get-Content $ServerOut, $ServerErr -ErrorAction SilentlyContinue | Write-Error
            throw "packaged HTTP server exited before health check passed"
        }
        try {
            $Response = Invoke-WebRequest -UseBasicParsing -Uri "http://127.0.0.1:$Port/health" -TimeoutSec 1
            if ($Response.StatusCode -eq 200) {
                $Health = $Response.Content
                break
            }
        } catch {
            Start-Sleep -Milliseconds 100
        }
    }
    if (-not $Health) {
        Get-Content $ServerOut, $ServerErr -ErrorAction SilentlyContinue | Write-Error
        throw "packaged HTTP server did not pass health check on port $Port"
    }
    $HealthJson = $Health | ConvertFrom-Json
    if ($HealthJson.status -ne "ok" -or $HealthJson.service -ne "engram" -or $HealthJson.version -ne $PackageVersion) {
        throw "unexpected health response: $Health"
    }

    Write-Host ""
    Write-Host "Package install smoke passed:"
    Write-Host "  $Archive"
    Write-Host "  $Checksum"
    Write-Host "  $ActualVersion"
    Write-Host "  $Health"
} finally {
    if ($ServerProcess -and -not $ServerProcess.HasExited) {
        Stop-Process -Id $ServerProcess.Id -Force -ErrorAction SilentlyContinue
        $ServerProcess.WaitForExit(5000) | Out-Null
    }
    Set-Location $RepoRoot
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $WorkDir
}
