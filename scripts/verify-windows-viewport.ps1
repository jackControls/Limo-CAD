param(
  [Parameter(Mandatory = $true)][string]$PackageDirectory,
  [Parameter(Mandatory = $true)][string]$DiagnosticsDirectory
)

if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Packaged OS input checks require a disposable GitHub-hosted desktop'
}
$executable = Join-Path $PackageDirectory 'noBS-CAD.exe'
if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) { throw "Missing package: $executable" }
if ($env:RUNNER_ARCH -eq 'ARM64') {
  & (Join-Path $PSScriptRoot 'prepare-hosted-arm-desktop.ps1') -EvidencePath (Join-Path $DiagnosticsDirectory 'runner-account-dialog.json')
}
# The existing fixture owns its process/window and uses the product Bevy capture.
# No embedded-child HWND or screenshot of the user's desktop is involved.
& cargo xtask test-mcp native-platform --desktop-input --server $executable --out (Join-Path $DiagnosticsDirectory 'native-platform')
if ($LASTEXITCODE -ne 0) { throw 'Packaged native input verification failed' }
