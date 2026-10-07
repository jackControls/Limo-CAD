param(
  [Parameter(Mandatory = $true)][string]$PackageDirectory,
  [Parameter(Mandatory = $true)][string]$DiagnosticsDirectory
)
$ErrorActionPreference = 'Stop'

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Packaged OS input checks require a disposable GitHub-hosted desktop'
}
$executable = Join-Path $PackageDirectory 'Limo-CAD.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw "Missing package: $executable" }


if ($env:RUNNER_ARCH -eq 'ARM64') {
  New-Item -ItemType Directory -Path $DiagnosticsDirectory -Force | Out-Null
  & (Join-Path $PSScriptRoot 'prepare-hosted-arm-desktop.ps1') -EvidencePath (Join-Path $DiagnosticsDirectory 'runner-account-dialog.json')
}
& cargo run --quiet --locked -p xtask --features native-control-harness -- test-mcp native-platform --desktop-input --server $executable --out (Join-Path $DiagnosticsDirectory 'native-platform')
if ($LASTEXITCODE -ne 0) { throw 'Packaged native input verification failed' }
