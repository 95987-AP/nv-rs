[CmdletBinding()]
param(
    [string]$DataPath = $env:NV_RS_DATA,
    [string]$Place = 'GSDocMitchellHouse',
    [switch]$NewGame,
    [switch]$UseActivePlugins,
    [switch]$Diagnostics,
    [string]$DiagnosticsDir,
    [switch]$ValidateOnly
)
$ErrorActionPreference = 'Stop'
if (!$DataPath) { $DataPath = Read-Host 'Enter your Fallout New Vegas Data folder (containing FalloutNV.esm)' }
$DataPath = $DataPath.Trim().Trim('"')
if (!(Test-Path -LiteralPath (Join-Path $DataPath 'FalloutNV.esm') -PathType Leaf)) {
    throw 'That folder does not contain FalloutNV.esm. Supply -DataPath or set NV_RS_DATA.'
}
$DataPath = (Resolve-Path -LiteralPath $DataPath).Path
$executable = $env:NV_RS_VIEWER
if (!$executable) { $executable = Join-Path $PSScriptRoot 'app/nv-viewer.exe' }
if (!(Test-Path -LiteralPath $executable -PathType Leaf)) { throw 'app/nv-viewer.exe is missing. Extract the entire playtest ZIP.' }
$viewerArguments = @($DataPath)
if ($NewGame) { $viewerArguments += '--new-game' } else { $viewerArguments += $Place }
if (!$UseActivePlugins) { $viewerArguments += '--official' }
if ($DiagnosticsDir) { $viewerArguments += @('--diagnostics-dir', $DiagnosticsDir) } elseif ($Diagnostics) { $viewerArguments += '--diagnostics' }
if ($ValidateOnly) {
    [pscustomobject]@{ Executable=$executable; Arguments=$viewerArguments; SaveFolder=(Join-Path $PSScriptRoot 'userdata') }
    return
}
$userFolder = Join-Path $PSScriptRoot 'userdata'
New-Item -ItemType Directory -Force -Path $userFolder | Out-Null
if (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'BUILD.txt')) { Get-Content -LiteralPath (Join-Path $PSScriptRoot 'BUILD.txt') }
Push-Location $userFolder
try {
    & $executable @viewerArguments
    $viewerExitCode = $LASTEXITCODE
} finally { Pop-Location }
exit $viewerExitCode
