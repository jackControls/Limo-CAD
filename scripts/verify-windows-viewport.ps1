param(
  [Parameter(Mandatory = $true)][string]$PackageDirectory,
  [Parameter(Mandatory = $true)][string]$DiagnosticsDirectory
)

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Packaged OS input checks require a disposable GitHub-hosted desktop'
}
$executable = Join-Path $PackageDirectory 'Limo-CAD.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw "Missing package: $executable" }


$previousArmEvidence = $env:LIMO_CAD_HOSTED_ARM_ACCOUNT_EVIDENCE
try {


  $env:LIMO_CAD_HOSTED_ARM_ACCOUNT_EVIDENCE = if ($env:RUNNER_ARCH -eq 'ARM64') {
    Join-Path $DiagnosticsDirectory 'runner-account-dialog.json'
  } else { $null }
  & cargo xtask test-mcp native-platform --desktop-input --server $executable --out (Join-Path $DiagnosticsDirectory 'native-platform')
  if ($LASTEXITCODE -ne 0) { throw 'Packaged native input verification failed' }
} finally {
  $env:LIMO_CAD_HOSTED_ARM_ACCOUNT_EVIDENCE = $previousArmEvidence
}
