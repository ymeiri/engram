param(
    [string] $Repo = $(if ($env:GITHUB_REPOSITORY) { $env:GITHUB_REPOSITORY } else { "ymeiri/engram" }),
    [string] $Tag = "",
    [string] $HostTriple = "",
    [string] $ExpectedGitHead = "",
    [ValidateSet("true", "false")]
    [string] $ExpectedTrackedChangesPresent = "false",
    [string] $AssetDir = ""
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $RepoRoot

$ReleasePackageTriples = @(
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "x86_64-pc-windows-msvc"
)

function Require-Binary {
    param([string] $Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "required tool is missing: $Name"
    }
}

function Sha256-File {
    param([string] $Path)
    (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLowerInvariant()
}

function Archive-Extension {
    param([string] $Triple)
    if ($Triple.EndsWith("-pc-windows-msvc")) {
        return "zip"
    }
    return "tar.gz"
}

function Expected-AssetNames {
    $Names = @()
    foreach ($Triple in $ReleasePackageTriples) {
        $Extension = Archive-Extension $Triple
        $Archive = "engram-$PackageVersion-$Triple.$Extension"
        $Names += $Archive
        $Names += "$Archive.sha256"
    }
    return $Names
}

Require-Binary "cargo"
Require-Binary "rustc"
Require-Binary "git"

$PackageId = (& cargo pkgid --locked -p engram-cli)
if ($LASTEXITCODE -ne 0) {
    throw "could not determine workspace package version for engram-cli"
}
$PackageVersion = ($PackageId -split "#")[-1]
if ($PackageVersion -notmatch "^[0-9]+\.[0-9]+\.[0-9]+(-[A-Za-z0-9][A-Za-z0-9.-]*)?$") {
    throw "workspace package version must be x.y.z with an optional prerelease suffix, got $PackageVersion"
}
if ([string]::IsNullOrWhiteSpace($Tag)) {
    $Tag = "v$PackageVersion"
}
if ($Tag -ne "v$PackageVersion") {
    throw "release tag version mismatch: expected v$PackageVersion, got $Tag"
}
if ([string]::IsNullOrWhiteSpace($HostTriple)) {
    $HostTriple = (& rustc -vV | Select-String "^host: " | ForEach-Object {
        $_.Line.Substring("host: ".Length)
    })
}
if ($HostTriple -notmatch "^[A-Za-z0-9_.+-]+(-[A-Za-z0-9_.+-]+)+$") {
    throw "--host-triple must be a Rust target triple, got $HostTriple"
}
if ($HostTriple -notmatch "pc-windows-msvc$") {
    throw "Windows published install verification requires a Windows MSVC host triple, got $HostTriple"
}
$ActualHostTriple = (& rustc -vV | Select-String "^host: " | ForEach-Object {
    $_.Line.Substring("host: ".Length)
})
if ($ActualHostTriple -ne $HostTriple) {
    throw "host triple mismatch: expected $HostTriple, current rustc host is $ActualHostTriple"
}
if ([string]::IsNullOrWhiteSpace($ExpectedGitHead)) {
    $ExpectedGitHead = (& git rev-parse HEAD).Trim()
}
if ($ExpectedGitHead -notmatch "^[0-9a-f]{40}$") {
    throw "--expected-git-head must be a 40-character Git SHA, got $ExpectedGitHead"
}

$ArchiveExtension = Archive-Extension $HostTriple
$ArchiveName = "engram-$PackageVersion-$HostTriple"
$ArchiveAsset = "$ArchiveName.$ArchiveExtension"
$ChecksumAsset = "$ArchiveAsset.sha256"
$DownloadedAssets = $false
$WorkDir = ""

try {
    if ([string]::IsNullOrWhiteSpace($AssetDir)) {
        Require-Binary "gh"
        $WorkDir = Join-Path ([System.IO.Path]::GetTempPath()) "engram-release-install-$([guid]::NewGuid().ToString('N'))"
        $AssetDir = Join-Path $WorkDir "assets"
        New-Item -ItemType Directory -Path $AssetDir -Force | Out-Null

        Write-Host ""
        Write-Host "==> inspect GitHub release"
        $Release = (& gh release view $Tag --repo $Repo --json tagName,isDraft,isPrerelease,assets) |
            ConvertFrom-Json
        if ($Release.tagName -ne $Tag) {
            throw "release tag mismatch: expected $Tag, got $($Release.tagName)"
        }
        if ($Release.isDraft) {
            throw "release is still a draft: $Tag"
        }

        $ExpectedAssets = @(Expected-AssetNames | Sort-Object)
        $ActualAssets = @($Release.assets | ForEach-Object { $_.name } | Sort-Object)
        if (($ExpectedAssets -join "`n") -ne ($ActualAssets -join "`n")) {
            throw "GitHub release assets do not match expected platform archives/checksums"
        }
        foreach ($Asset in $Release.assets) {
            if ($Asset.state -ne "uploaded" -or $Asset.size -le 0 -or $Asset.digest -notmatch "^sha256:[0-9a-f]{64}$") {
                throw "GitHub release asset metadata is invalid for $($Asset.name)"
            }
        }

        Write-Host ""
        Write-Host "==> download release assets"
        & gh release download $Tag --repo $Repo --pattern "engram-*" --dir $AssetDir
        if ($LASTEXITCODE -ne 0) {
            throw "failed to download release assets"
        }
        $DownloadedAssets = $true

        Write-Host ""
        Write-Host "==> verify GitHub release asset digests"
        foreach ($Asset in $Release.assets) {
            $Path = Join-Path $AssetDir $Asset.name
            if (-not (Test-Path $Path)) {
                throw "release asset is missing after download: $Path"
            }
            $ActualDigest = "sha256:$(Sha256-File $Path)"
            if ($ActualDigest -ne $Asset.digest) {
                throw "GitHub asset digest mismatch for $($Asset.name): expected $($Asset.digest), got $ActualDigest"
            }
        }
    } else {
        if (-not (Test-Path $AssetDir)) {
            throw "asset directory does not exist: $AssetDir"
        }
    }

    if (-not (Test-Path (Join-Path $AssetDir $ArchiveAsset))) {
        throw "release archive asset is missing or empty: $(Join-Path $AssetDir $ArchiveAsset)"
    }
    if (-not (Test-Path (Join-Path $AssetDir $ChecksumAsset))) {
        throw "release checksum asset is missing or empty: $(Join-Path $AssetDir $ChecksumAsset)"
    }

    Write-Host ""
    Write-Host "==> verify release install"
    $env:ALLOW_PACKAGE_DIST_DIR_OVERRIDE = "1"
    $env:ALLOW_PACKAGE_BUILD_SKIP = "1"
    $env:ALLOW_PACKAGE_IDENTITY_OVERRIDE = "1"
    $env:DIST_DIR = $AssetDir
    $env:SKIP_PACKAGE_BUILD = "1"
    $env:EXPECTED_PACKAGE_GIT_HEAD = $ExpectedGitHead
    $env:EXPECTED_TRACKED_CHANGES_PRESENT = $ExpectedTrackedChangesPresent
    & (Join-Path $RepoRoot "scripts/package-install-smoke-windows.ps1")
    if ($LASTEXITCODE -ne 0) {
        throw "Windows release install verification failed"
    }

    Write-Host ""
    Write-Host "Windows release install evidence collected:"
    Write-Host "  repo: $Repo"
    Write-Host "  tag: $Tag"
    Write-Host "  version: $PackageVersion"
    Write-Host "  host_triple: $HostTriple"
    Write-Host "  assets_source: $(if ($DownloadedAssets) { 'github_release' } else { 'asset_dir' })"
    Write-Host "  archive: $ArchiveAsset"
    Write-Host "  checksum: $ChecksumAsset"
    Write-Host "  asset_install_verified: true"
    Write-Host "  published_install_verified: $($DownloadedAssets.ToString().ToLowerInvariant())"
} finally {
    if (-not [string]::IsNullOrWhiteSpace($WorkDir)) {
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue $WorkDir
    }
}
