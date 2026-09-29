$ErrorActionPreference = 'Stop'
# This process uses only managed fake window APIs. Never call user32 or operate
# the developer's desktop while testing hosted-runner preparation.
Add-Type -TypeDefinition @'
using System;
using System.Text;
public static class HostedArmAccountWindow {
    public delegate bool EnumProc(IntPtr window, IntPtr unused);
    public static long Foreground = 100;
    public static long[] Enumerated = new long[0];
    public static bool Visible = true;
    public static int CloseCount = 0;
    public static string Executable;
    public static bool EnumWindows(EnumProc callback, IntPtr unused) {
        foreach (long window in Enumerated) if (!callback(new IntPtr(window), unused)) return false;
        return true;
    }
    public static IntPtr GetForegroundWindow() { return new IntPtr(Foreground); }
    public static bool IsWindowVisible(IntPtr window) { return Visible && window.ToInt64() >= 100; }
    public static uint GetWindowThreadProcessId(IntPtr window, out uint pid) { pid = (uint)window.ToInt64() + 100; return 1; }
    public static int GetWindowText(IntPtr window, StringBuilder text, int count) { text.Append("Microsoft account"); return text.Length; }
    public static int GetClassName(IntPtr window, StringBuilder text, int count) { text.Append("Windows.UI.Core.CoreWindow"); return text.Length; }
    public static IntPtr SendMessageTimeout(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam, uint flags, uint timeout, out UIntPtr result) {
        if (message != 0x0010 || flags != 3 || timeout != 1500) throw new Exception("Unexpected window action");
        CloseCount++; Visible = false; result = UIntPtr.Zero; return new IntPtr(1);
    }
}
'@
# Shadow only after the managed fake is loaded. The actual script body runs,
# but its P/Invoke declarations can never be loaded into this test process.
function Add-Type { param($TypeDefinition) }
[HostedArmAccountWindow]::Executable = Join-Path $env:WINDIR 'System32\WWAHost.exe'
function Get-Process { param($Id, $ErrorAction) [pscustomobject]@{ ProcessName = 'WWAHost'; Path = [HostedArmAccountWindow]::Executable } }

$guardNames = @('GITHUB_ACTIONS', 'RUNNER_ENVIRONMENT', 'RUNNER_OS', 'RUNNER_ARCH', 'GITHUB_REPOSITORY', 'GITHUB_RUN_ID', 'RUNNER_TEMP')
$original = @{}
foreach ($name in $guardNames) { $original[$name] = [Environment]::GetEnvironmentVariable($name) }
$evidenceRoot = Join-Path ([IO.Path]::GetTempPath()) ('nbcad-preflight-fake-' + [Guid]::NewGuid())
$preflight = Join-Path $PSScriptRoot '../prepare-hosted-arm-desktop.ps1'
function Invoke-Case([string]$name, [string]$expected, [int]$closes) {
    $path = Join-Path $evidenceRoot ($name + '.json')
    $caught = $null
    try { & $preflight -EvidencePath $path } catch { $caught = $_ }
    $report = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    if ($report.status -ne $expected -or [HostedArmAccountWindow]::CloseCount -ne $closes) {
        throw "$name expected $expected/$closes, got $($report.status)/$([HostedArmAccountWindow]::CloseCount): $caught"
    }
    if (($expected -eq 'failed') -ne ($null -ne $caught)) { throw "$name exception did not match outcome" }
    if (-not $report.started_utc -or -not $report.finished_utc) { throw "$name omitted timing evidence" }
    return $report
}
try {
    $env:GITHUB_ACTIONS = 'true'
    $env:RUNNER_ENVIRONMENT = 'github-hosted'
    $env:RUNNER_OS = 'Windows'
    $env:RUNNER_ARCH = 'ARM64'
    $env:GITHUB_REPOSITORY = 'jackControls/noBS-CAD'
    $env:GITHUB_RUN_ID = '1234'
    $env:RUNNER_TEMP = $evidenceRoot

    $report = Invoke-Case 'foreground-omitted-by-enumeration' 'closed' 1
    if ($report.windows.Count -ne 1 -or $report.windows[0].observed_via -ne 'foreground') { throw 'Foreground provenance missing' }

    [HostedArmAccountWindow]::Visible = $true
    [HostedArmAccountWindow]::CloseCount = 0
    [HostedArmAccountWindow]::Enumerated = @(100)
    $null = Invoke-Case 'same-hwnd-not-ambiguous' 'closed' 1

    [HostedArmAccountWindow]::Visible = $true
    [HostedArmAccountWindow]::CloseCount = 0
    [HostedArmAccountWindow]::Enumerated = @(101)
    $null = Invoke-Case 'two-windows-refused' 'failed' 0

    [HostedArmAccountWindow]::Enumerated = @()
    [HostedArmAccountWindow]::Executable = Join-Path $evidenceRoot 'WWAHost.exe'
    $null = Invoke-Case 'wrong-executable-refused' 'failed' 0

    [HostedArmAccountWindow]::Foreground = 0
    $null = Invoke-Case 'no-account-window' 'not_present' 0

    $env:GITHUB_ACTIONS = 'false'
    $refusalPath = Join-Path $evidenceRoot 'outside-host-refused.json'
    $refused = $false
    try { & $preflight -EvidencePath $refusalPath } catch { $refused = $_.Exception.Message -like '*disposable GitHub-hosted ARM64*' }
    if (-not $refused -or (Test-Path -LiteralPath $refusalPath)) { throw 'Non-host guard did not refuse before writes' }
    Write-Output "6 managed preflight cases passed; no desktop APIs invoked. Evidence: $evidenceRoot"
} finally {
    foreach ($name in $guardNames) { [Environment]::SetEnvironmentVariable($name, $original[$name]) }
}
