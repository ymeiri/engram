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
$AllowAssetOverwrite = if ($env:ALLOW_PACKAGE_ASSET_OVERWRITE) {
    $env:ALLOW_PACKAGE_ASSET_OVERWRITE
} else {
    "0"
}
$AllowTrackedChanges = if ($env:ALLOW_TRACKED_CHANGES) {
    $env:ALLOW_TRACKED_CHANGES
} else {
    "0"
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

Require-Binary "cargo"
Require-Binary "rustc"
Require-Binary "git"
Assert-Flag "ALLOW_PACKAGE_DIST_DIR_OVERRIDE" $AllowDistDirOverride
Assert-Flag "ALLOW_PACKAGE_ASSET_OVERWRITE" $AllowAssetOverwrite
Assert-Flag "ALLOW_TRACKED_CHANGES" $AllowTrackedChanges

if ([string]::IsNullOrWhiteSpace($DistDir)) {
    throw "DIST_DIR must not be empty"
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

$ReleaseNotesSlug = ($PackageVersion.ToUpperInvariant() -replace "[^A-Z0-9]", "_")
$ReleaseNotesSource = Join-Path $RepoRoot "docs/RELEASE_NOTES_V$ReleaseNotesSlug.md"
$HostTriple = (& rustc -vV | Select-String "^host: " | ForEach-Object {
    $_.Line.Substring("host: ".Length)
})
if ([string]::IsNullOrWhiteSpace($HostTriple)) {
    throw "host triple could not be determined from rustc -vV"
}
if ($HostTriple -notmatch "^[A-Za-z0-9_.+-]+(-[A-Za-z0-9_.+-]+)+$") {
    throw "host triple must be a Rust target triple, got $HostTriple"
}
if ($HostTriple -notmatch "pc-windows-msvc$") {
    throw "package-release-windows.ps1 must run on a Windows MSVC Rust host, got $HostTriple"
}

$ArchiveName = "engram-$PackageVersion-$HostTriple"
$ZipPath = Join-Path $ResolvedDistDir "$ArchiveName.zip"
$ChecksumPath = "$ZipPath.sha256"
$ExistingOutputs = @(@($ZipPath, $ChecksumPath) | Where-Object { Test-Path $_ })
if ($ExistingOutputs.Count -gt 0 -and $AllowAssetOverwrite -ne "1") {
    throw "release package output already exists; refusing to overwrite: $($ExistingOutputs -join ', ')"
}

$GitHead = (& git rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $GitHead -notmatch "^[0-9a-f]{40}$") {
    throw "could not determine current Git head"
}
& git diff --quiet --ignore-submodules --
$WorktreeClean = ($LASTEXITCODE -eq 0)
& git diff --cached --quiet --ignore-submodules --
$IndexClean = ($LASTEXITCODE -eq 0)
$TrackedChangesPresent = -not ($WorktreeClean -and $IndexClean)
if ($TrackedChangesPresent -and $AllowTrackedChanges -ne "1") {
    throw "tracked working-tree or index changes are present; commit changes first"
}

$WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) "engram-package-$([guid]::NewGuid().ToString('N'))"
$FinalOutputsCommitted = $false
try {
    New-Item -ItemType Directory -Path $WorkDir, $ResolvedDistDir -Force | Out-Null

    Run-Step "build release binary" {
        & cargo build --locked --release -p engram-cli
        if ($LASTEXITCODE -ne 0) {
            throw "cargo release build failed"
        }
    }

    if (-not (Test-Path $ReleaseNotesSource)) {
        throw "release notes not found for version ${PackageVersion}: $ReleaseNotesSource"
    }

    $Binary = Join-Path $RepoRoot "target/release/engram.exe"
    if (-not (Test-Path $Binary)) {
        throw "release binary was not built at $Binary"
    }

    $ExpectedVersion = "engram $PackageVersion"
    $ActualVersion = (& $Binary --version).Trim()
    if ($ActualVersion -ne $ExpectedVersion) {
        throw "binary version mismatch: expected '$ExpectedVersion', got '$ActualVersion'"
    }

    $StagingDir = Join-Path $WorkDir $ArchiveName
    New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null
    Copy-Item $Binary (Join-Path $StagingDir "engram.exe")
    Copy-Item (Join-Path $RepoRoot "README.md") $StagingDir
    Copy-Item (Join-Path $RepoRoot "LICENSE") $StagingDir
    Copy-Item (Join-Path $RepoRoot "CHANGELOG.md") $StagingDir
    Copy-Item $ReleaseNotesSource (Join-Path $StagingDir "RELEASE_NOTES.md")

    $CargoLockSha256 = Sha256-File (Join-Path $RepoRoot "Cargo.lock")
    $ManifestPath = Join-Path $StagingDir "MANIFEST.json"
    $PackageFiles = @("engram.exe", "README.md", "LICENSE", "CHANGELOG.md", "RELEASE_NOTES.md")
    $FileEntries = foreach ($PackageFile in $PackageFiles) {
        [ordered]@{
            path = $PackageFile
            sha256 = Sha256-File (Join-Path $StagingDir $PackageFile)
        }
    }
    $Manifest = [ordered]@{
        package = "engram"
        version = $PackageVersion
        host_triple = $HostTriple
        archive_name = $ArchiveName
        git_head = $GitHead
        tracked_changes_present = $TrackedChangesPresent
        cargo_lock_sha256 = $CargoLockSha256
        files = $FileEntries
    }
    $Manifest | ConvertTo-Json -Depth 5 | Set-Content -Path $ManifestPath -Encoding utf8

    $TmpZip = Join-Path $ResolvedDistDir ".$ArchiveName.zip.$([guid]::NewGuid().ToString('N'))"
    $TmpChecksum = Join-Path $ResolvedDistDir ".$ArchiveName.zip.sha256.$([guid]::NewGuid().ToString('N'))"
    Run-Step "create archive" {
        Compress-Archive -Path $StagingDir -DestinationPath $TmpZip -Force
    }
    $ArchiveSha256 = Sha256-File $TmpZip
    Set-Content -Path $TmpChecksum -Value "$ArchiveSha256  $(Split-Path -Leaf $ZipPath)" -Encoding ascii

    if ((Test-Path $ZipPath) -or (Test-Path $ChecksumPath)) {
        if ($AllowAssetOverwrite -ne "1") {
            throw "release package output appeared during packaging; refusing to overwrite"
        }
        Remove-Item -Force -ErrorAction SilentlyContinue $ZipPath, $ChecksumPath
    }

    Move-Item $TmpZip $ZipPath
    Move-Item $TmpChecksum $ChecksumPath
    $FinalOutputsCommitted = $true

    Write-Host ""
    Write-Host "Release package created:"
    Write-Host "  $ZipPath"
    Write-Host "  $ChecksumPath"
} finally {
    if (-not $FinalOutputsCommitted) {
        Remove-Item -Force -ErrorAction SilentlyContinue $ZipPath, $ChecksumPath
    }
    Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $WorkDir
}
