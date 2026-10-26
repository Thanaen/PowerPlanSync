#requires -Version 7.5

[CmdletBinding()]
param(
    [string]$ApplicationPath = (Join-Path $PSScriptRoot '../target/release/power-plan-sync.exe'),
    [string]$NoticesPath = (Join-Path $PSScriptRoot '../THIRD_PARTY_NOTICES.md'),
    [string]$LicenseTextsPath = (Join-Path $PSScriptRoot '../notices/THIRD_PARTY_LICENSES.txt'),
    [string]$OutputDirectory = (Join-Path $PSScriptRoot '../dist')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$cargo = Get-Content -LiteralPath (Join-Path $repo 'Cargo.toml') -Raw
$packageSection = [regex]::Match($cargo, '(?ms)^\[package\]\s*(.*?)(?=^\[|\z)')
$versionMatch = [regex]::Match($packageSection.Groups[1].Value, '(?m)^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?)"\s*$')
if (-not $versionMatch.Success) { throw 'Version de paquet introuvable dans Cargo.toml.' }
$version = $versionMatch.Groups[1].Value
$packageName = "PowerPlanSync-$version-windows-x64"
$output = [IO.Path]::GetFullPath($OutputDirectory)
$destination = Join-Path $output $packageName
$zipPath = "$destination.zip"
$zipHashPath = "$zipPath.sha256"
$sources = @(
    [pscustomobject]@{ Source = [IO.Path]::GetFullPath($ApplicationPath); Destination = 'power-plan-sync.exe' },
    [pscustomobject]@{ Source = Join-Path $repo 'docs/guide-utilisateur.md'; Destination = 'README.md' },
    [pscustomobject]@{ Source = [IO.Path]::GetFullPath($NoticesPath); Destination = 'THIRD_PARTY_NOTICES.md' },
    [pscustomobject]@{ Source = [IO.Path]::GetFullPath($LicenseTextsPath); Destination = 'notices/THIRD_PARTY_LICENSES.txt' },
    [pscustomobject]@{ Source = Join-Path $repo 'assets/fonts/OFL.txt'; Destination = 'notices/OFL-IBM-Plex.txt' }
)
foreach ($item in $sources) {
    if (-not (Test-Path -LiteralPath $item.Source -PathType Leaf)) { throw "Fichier requis absent : $($item.Source)" }
}
foreach ($path in @($destination, $zipPath, $zipHashPath)) {
    if (Test-Path -LiteralPath $path) { throw "Destination déjà présente, aucun écrasement autorisé : $path" }
}

$binary = [IO.File]::ReadAllBytes($sources[0].Source)
if ($binary.Length -lt 64 -or $binary[0] -ne 0x4D -or $binary[1] -ne 0x5A) { throw 'Exécutable Windows invalide.' }
$peOffset = [BitConverter]::ToInt32($binary, 60)
if ($peOffset -lt 0 -or $peOffset -gt $binary.Length - 94 -or [BitConverter]::ToUInt32($binary, $peOffset) -ne 0x00004550) { throw 'En-tête PE invalide.' }
if ([BitConverter]::ToUInt16($binary, $peOffset + 4) -ne 0x8664) { throw 'Le paquet exige un exécutable Windows x64.' }
if ([BitConverter]::ToUInt16($binary, $peOffset + 92) -ne 2) { throw 'Le paquet exige le binaire graphique release, sans console.' }

[IO.Directory]::CreateDirectory($output) | Out-Null
$lockPath = Join-Path $output ".$packageName.packaging.lock"
$lock = [IO.File]::Open($lockPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
try {
    foreach ($path in @($destination, $zipPath, $zipHashPath)) {
        if (Test-Path -LiteralPath $path) { throw "Destination déjà présente : $path" }
    }
    [IO.Directory]::CreateDirectory($destination) | Out-Null
    $entries = [Collections.Generic.List[object]]::new()
    foreach ($item in $sources) {
        $target = Join-Path $destination $item.Destination
        [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target)) | Out-Null
        $sourceHash = (Get-FileHash -LiteralPath $item.Source -Algorithm SHA256).Hash.ToLowerInvariant()
        [IO.File]::Copy($item.Source, $target, $false)
        $copiedHash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($sourceHash -ne $copiedHash) { throw "Le fichier a changé pendant l'assemblage : $($item.Source)" }
        $entries.Add([ordered]@{
            path = $item.Destination
            bytes = ([IO.FileInfo]::new($target)).Length
            sha256 = $copiedHash
        })
    }
    $manifest = [ordered]@{
        schema_version = 1
        application = 'PowerPlanSync'
        version = $version
        platform = 'windows-x64'
        packaged_utc = [DateTime]::UtcNow.ToString('o')
        hash_algorithm = 'SHA-256'
        files = $entries.ToArray()
    } | ConvertTo-Json -Depth 6
    $manifestPath = Join-Path $destination 'manifest.json'
    $manifestStream = [IO.File]::Open($manifestPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $manifestBytes = [Text.UTF8Encoding]::new($false).GetBytes($manifest + "`n")
        $manifestStream.Write($manifestBytes)
    } finally { $manifestStream.Dispose() }

    $zipStream = [IO.File]::Open($zipPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    $archive = $null
    try {
        $archive = [IO.Compression.ZipArchive]::new($zipStream, [IO.Compression.ZipArchiveMode]::Create, $true)
        foreach ($file in Get-ChildItem -LiteralPath $destination -Recurse -File | Sort-Object FullName) {
            $entryPath = [IO.Path]::GetRelativePath($output, $file.FullName).Replace('\', '/')
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($archive, $file.FullName, $entryPath, [IO.Compression.CompressionLevel]::Optimal) | Out-Null
        }
    } finally {
        if ($null -ne $archive) { $archive.Dispose() }
        $zipStream.Dispose()
    }
    $zipHash = (Get-FileHash -LiteralPath $zipPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $hashStream = [IO.File]::Open($zipHashPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $hashBytes = [Text.UTF8Encoding]::new($false).GetBytes("$zipHash  $([IO.Path]::GetFileName($zipPath))`n")
        $hashStream.Write($hashBytes)
    } finally { $hashStream.Dispose() }
    [pscustomobject]@{ Directory = $destination; Archive = $zipPath; ArchiveSha256 = $zipHash; Manifest = $manifestPath }
} finally {
    $lock.Dispose()
    [IO.File]::Delete($lockPath)
}
