[CmdletBinding()]
param(
    [string]$BinaryDirectory = (Join-Path $PSScriptRoot '../target/release/examples'),
    [string]$EvidenceDirectory = (Join-Path $PSScriptRoot '../.audit/prototype-verification'),
    [switch]$AllowPowerSwitch,
    [ValidateRange(1, 10)][int]$Cycles = 3
)

$ErrorActionPreference = 'Stop'
$binaryPath = [IO.Path]::GetFullPath($BinaryDirectory)
$evidencePath = [IO.Path]::GetFullPath($EvidenceDirectory)
$probePath = Join-Path $binaryPath 'windows_probe.exe'
$fixturePath = Join-Path $binaryPath 'demo_target.exe'
foreach ($file in @($probePath, $fixturePath)) {
    if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
        throw "Exécutable manquant : $file. Construire les exemples en release."
    }
}
[IO.Directory]::CreateDirectory($evidencePath) | Out-Null

function Read-ActivePlan {
    $result = & powercfg.exe /getactivescheme
    if ($LASTEXITCODE -ne 0) { throw 'Impossible de lire le plan Windows actif.' }
    $match = [regex]::Match(($result -join "`n"), '[0-9a-fA-F]{8}(?:-[0-9a-fA-F]{4}){3}-[0-9a-fA-F]{12}')
    if (-not $match.Success) { throw 'GUID absent de la réponse Windows.' }
    $match.Value.ToLowerInvariant()
}

$initialPlan = Read-ActivePlan
$initialPlan | Set-Content -LiteralPath (Join-Path $evidencePath 'initial-plan.txt') -Encoding UTF8
$arguments = @('--fixture', $fixturePath)
if ($AllowPowerSwitch) { $arguments += @('--allow-switch', '--cycles', "$Cycles") }

try {
    & $probePath @arguments 2>&1 | Tee-Object -FilePath (Join-Path $evidencePath 'windows-probe.txt')
    if ($LASTEXITCODE -ne 0) { throw "La sonde a échoué avec le code $LASTEXITCODE." }
}
finally {
    if ($AllowPowerSwitch) {
        & powercfg.exe /setactive $initialPlan
        if ($LASTEXITCODE -ne 0) {
            throw "La restauration du plan initial $initialPlan a échoué."
        }
    }
    $finalPlan = Read-ActivePlan
    $finalPlan | Set-Content -LiteralPath (Join-Path $evidencePath 'final-plan.txt') -Encoding UTF8
    if ($finalPlan -ne $initialPlan) {
        throw "Plan final différent du plan initial : $finalPlan au lieu de $initialPlan."
    }
    Write-Output "Plan initial confirmé en fin d'essai : $finalPlan"
}
