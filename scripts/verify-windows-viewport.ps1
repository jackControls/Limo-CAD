param(
  [Parameter(Mandatory = $true)][string]$PackageDirectory,
  [Parameter(Mandatory = $true)][string]$DiagnosticsDirectory,
  [Parameter(Mandatory = $true)][switch]$ControlledSourceHost
)

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Controlled source host OS input checks require a disposable GitHub-hosted desktop'
}
if (-not $ControlledSourceHost) { throw 'Use a separately built control-enabled source host, never the default release artifact' }
$executable = Join-Path $PackageDirectory 'Limo-CAD.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw "Missing package: $executable" }


$previousArmEvidence = $env:LIMO_CAD_HOSTED_ARM_ACCOUNT_EVIDENCE
try {


  $env:LIMO_CAD_HOSTED_ARM_ACCOUNT_EVIDENCE = if ($env:RUNNER_ARCH -eq 'ARM64') {
    Join-Path $DiagnosticsDirectory 'runner-account-dialog.json'
  } else { $null }
  & cargo run --quiet --locked -p xtask --features native-control-harness -- test-mcp native-platform --desktop-input --server $executable --out (Join-Path $DiagnosticsDirectory 'native-platform')
  if ($LASTEXITCODE -ne 0) { throw 'Controlled source host native input verification failed' }
} finally {
  $env:LIMO_CAD_HOSTED_ARM_ACCOUNT_EVIDENCE = $previousArmEvidence
}
