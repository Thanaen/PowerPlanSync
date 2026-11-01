#requires -Version 7.5

<#
.SYNOPSIS
Régénère l'inventaire de `THIRD_PARTY_NOTICES.md` et `notices/THIRD_PARTY_LICENSES.txt` à partir
du graphe verrouillé Windows x64 (dépendances normales et de compilation, sans celles de
développement). Les textes viennent des sources Cargo hors ligne : fichiers `LICENSE*`,
`LICENCE*` et `COPYING*` à la racine du paquet. Les textes identiques sont regroupés.
#>
[CmdletBinding()]
param(
    [string]$Target = 'x86_64-pc-windows-msvc',
    [string]$RepositoryRoot = (Join-Path $PSScriptRoot '..')
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$root = [IO.Path]::GetFullPath($RepositoryRoot)
$noticesPath = Join-Path $root 'THIRD_PARTY_NOTICES.md'
$licensesPath = Join-Path $root 'notices/THIRD_PARTY_LICENSES.txt'
$separator = '=' * 80

Push-Location $root
try {
    $treeLines = & cargo tree --offline --locked --target $Target --edges normal,build --prefix none --format '{p}|{l}'
    if ($LASTEXITCODE -ne 0) { throw 'cargo tree a échoué.' }
    $metadata = (& cargo metadata --offline --locked --format-version 1 --filter-platform $Target) | ConvertFrom-Json -Depth 64
    if ($LASTEXITCODE -ne 0) { throw 'cargo metadata a échoué.' }
} finally {
    Pop-Location
}

$rootId = $metadata.resolve.root
$directories = @{}
foreach ($package in $metadata.packages) {
    if ($package.id -ne $rootId) {
        $directories["$($package.name) v$($package.version)"] = Split-Path -Parent $package.manifest_path
    }
}

$packages = [ordered]@{}
foreach ($line in $treeLines) {
    if ($line -notmatch '^(?<name>\S+) (?<version>v\S+)(?: \([^)]*\))?(?: \(\*\))?\|(?<license>.*)$') { continue }
    $key = "$($Matches.name) $($Matches.version)"
    if (-not $directories.ContainsKey($key) -or $packages.Contains($key)) { continue }
    $packages[$key] = $Matches.license
}
$names = [string[]]@($packages.Keys)
[Array]::Sort($names, [StringComparer]::Ordinal)

$byLicense = @{}
foreach ($name in $names) {
    $license = $packages[$name]
    if (-not $byLicense.ContainsKey($license)) { $byLicense[$license] = [Collections.Generic.List[string]]::new() }
    $byLicense[$license].Add($name)
}
$expressions = [string[]]@($byLicense.Keys)
[Array]::Sort($expressions, [StringComparer]::Ordinal)
$inventory = [Text.StringBuilder]::new()
[void]$inventory.Append("## Locked Windows dependency inventory ($($names.Count) packages)`n")
foreach ($expression in $expressions) {
    [void]$inventory.Append("`n### ``$expression```n`n$($byLicense[$expression] -join ', ').`n")
}

$texts = [ordered]@{}
$unlicensed = [Collections.Generic.List[string]]::new()
foreach ($name in $names) {
    $directory = $directories[$name]
    $files = @(Get-ChildItem -LiteralPath $directory -File | Where-Object Name -Match '^(licen[cs]e|copying)')
    if ($files.Count -eq 0) { $unlicensed.Add($name); continue }
    $sorted = [string[]]@($files.Name)
    [Array]::Sort($sorted, [StringComparer]::Ordinal)
    foreach ($file in $sorted) {
        $text = [IO.File]::ReadAllText((Join-Path $directory $file)).Replace("`r`n", "`n").TrimEnd()
        if (-not $texts.Contains($text)) {
            $texts[$text] = [pscustomobject]@{
                Packages = [Collections.Generic.List[string]]::new()
                Sources = [Collections.Generic.List[string]]::new()
            }
        }
        $entry = $texts[$text]
        if (-not $entry.Packages.Contains($name)) { $entry.Packages.Add($name) }
        $entry.Sources.Add("$name/$file")
    }
}

$output = [Text.StringBuilder]::new()
[void]$output.Append("THIRD-PARTY LICENSE TEXTS`n`nGenerated from the Cargo package sources used by the locked Windows x64 normal and build dependency graph.`n")
foreach ($text in $texts.Keys) {
    $entry = $texts[$text]
    [void]$output.Append("`n$separator`nPACKAGES`n$($entry.Packages -join ', ')`nSOURCE FILES`n$($entry.Sources -join ', ')`n$separator`n`n$text`n")
}
if ($unlicensed.Count -gt 0) {
    [void]$output.Append("`n$separator`nPACKAGES WITHOUT A LICENSE FILE IN THE PUBLISHED CRATE`n$separator`n`n$($unlicensed -join ', ')`n`nTheir SPDX expressions remain listed in THIRD_PARTY_NOTICES.md.`n")
}
$utf8 = [Text.UTF8Encoding]::new($false)
[IO.File]::WriteAllText($licensesPath, $output.ToString(), $utf8)

$notices = [IO.File]::ReadAllText($noticesPath).Replace("`r`n", "`n")
$start = $notices.IndexOf('## Locked Windows dependency inventory')
if ($start -lt 0) { throw "Section d'inventaire absente de THIRD_PARTY_NOTICES.md." }
[IO.File]::WriteAllText($noticesPath, $notices.Substring(0, $start) + $inventory.ToString(), $utf8)
Write-Output "$($names.Count) paquets, $($texts.Count) textes de licence, $($unlicensed.Count) sans fichier de licence."
