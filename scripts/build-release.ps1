#requires -Version 7.5

<#
.SYNOPSIS
Construit le binaire release à publier sans les chemins de la machine de construction (T043,
T044). Les emplacements de panique des dépendances deviennent `cargo/…`, ceux du dépôt
`PowerPlanSync/…`, et le chemin du PDB se réduit à son nom de fichier.
#>
[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$cargoHome = if ($env:CARGO_HOME) { $env:CARGO_HOME } else { Join-Path $env:USERPROFILE '.cargo' }
$rustupHome = if ($env:RUSTUP_HOME) { $env:RUSTUP_HOME } else { Join-Path $env:USERPROFILE '.rustup' }

# Séparées par 0x1F, ces options gardent les chemins qui contiennent des espaces. Elles
# remplacent RUSTFLAGS le temps de cette construction.
$flags = @(
    "--remap-path-prefix=$([IO.Path]::GetFullPath($cargoHome))=cargo",
    "--remap-path-prefix=$([IO.Path]::GetFullPath($rustupHome))=rustup",
    "--remap-path-prefix=$repo=PowerPlanSync",
    '-Clink-arg=/PDBALTPATH:%_PDB%'
)
$previous = $env:CARGO_ENCODED_RUSTFLAGS
$env:CARGO_ENCODED_RUSTFLAGS = $flags -join [char]0x1F
try {
    Push-Location -LiteralPath $repo
    try {
        cargo build --locked --release --bin power-plan-sync
        if ($LASTEXITCODE -ne 0) { throw "cargo build a échoué (code $LASTEXITCODE)." }
    } finally {
        Pop-Location
    }
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previous
}
