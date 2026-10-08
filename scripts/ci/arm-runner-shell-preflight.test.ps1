$ErrorActionPreference = 'Stop'


Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Text;
public sealed class FakeShellWindow {
    public string Title, ClassName = "Windows.UI.Core.CoreWindow", ProcessName, Executable;
    public bool Visible = true, ChangeOwner, RejectClose, StayVisible, RejectHide;
    public int IdentityReads;
}
public static class HostedArmAccountWindow {
    public delegate bool EnumProc(IntPtr window, IntPtr unused);
    public static Dictionary<long, FakeShellWindow> Windows = new Dictionary<long, FakeShellWindow>();
    public static List<long> Closed = new List<long>();
    public static long Foreground;
    public static bool EnumWindows(EnumProc callback, IntPtr unused) { return true; }
    public static IntPtr GetForegroundWindow() { return new IntPtr(Foreground); }
    public static bool IsWindowVisible(IntPtr window) {
        FakeShellWindow value;
        return Windows.TryGetValue(window.ToInt64(), out value) && value.Visible;
    }
    public static uint GetWindowThreadProcessId(IntPtr window, out uint pid) {
        FakeShellWindow value;
        long handle = window.ToInt64();
        pid = (uint)handle + 1000;
        if (Windows.TryGetValue(handle, out value) && value.ChangeOwner && ++value.IdentityReads > 1) pid += 10000;
        return 1;
    }
    public static int GetWindowText(IntPtr window, StringBuilder text, int count) {
        FakeShellWindow value;
        if (Windows.TryGetValue(window.ToInt64(), out value)) text.Append(value.Title);
        return text.Length;
    }
    public static int GetClassName(IntPtr window, StringBuilder text, int count) {
        FakeShellWindow value;
        if (Windows.TryGetValue(window.ToInt64(), out value)) text.Append(value.ClassName);
        return text.Length;
    }
    public static IntPtr SendMessageTimeout(IntPtr window, uint message, UIntPtr wParam, IntPtr lParam, uint flags, uint timeout, out UIntPtr result) {
        if (message != 0x0010 || flags != 3 || timeout != 1500 || wParam != UIntPtr.Zero || lParam != IntPtr.Zero)
            throw new Exception("Unexpected shell action");
        long handle = window.ToInt64();
        Closed.Add(handle);
        FakeShellWindow value = Windows[handle];
        result = UIntPtr.Zero;
        if (value.RejectClose) return IntPtr.Zero;
        if (!value.StayVisible) value.Visible = false;
        return new IntPtr(1);
    }
    public static bool ShowWindowAsync(IntPtr window, int command) {
        if (command != 0) throw new Exception("Unexpected shell hide action");
        FakeShellWindow value = Windows[window.ToInt64()];
        if (value.RejectHide) return false;
        value.Visible = false;
        return true;
    }
}
'@
function Add-Type { param($TypeDefinition) }
function Get-Process {
    param($Id, $ErrorAction)
    $value = [HostedArmAccountWindow]::Windows[[long]$Id - 1000]
    if ($null -eq $value) { throw 'Unknown fake process' }
    [pscustomobject]@{ ProcessName = $value.ProcessName; Path = $value.Executable }
}
function Reset-Windows {
    [HostedArmAccountWindow]::Windows.Clear()
    [HostedArmAccountWindow]::Closed.Clear()
    [HostedArmAccountWindow]::Foreground = 0
}
function Add-Shell([long]$handle, [string]$kind) {
    $value = [FakeShellWindow]::new()
    if ($kind -eq 'Start') {
        $value.Title = 'Start'
        $value.ProcessName = 'StartMenuExperienceHost'
        $value.Executable = Join-Path $env:WINDIR 'SystemApps\Microsoft.Windows.StartMenuExperienceHost_cw5n1h2txyewy\StartMenuExperienceHost.exe'
    } else {
        $value.Title = 'Search'
        $value.ProcessName = 'SearchHost'
        $value.Executable = Join-Path $env:WINDIR 'SystemApps\MicrosoftWindows.Client.CBS_cw5n1h2txyewy\SearchHost.exe'
    }
    [HostedArmAccountWindow]::Windows[$handle] = $value
    $value
}

$guardNames = @('GITHUB_ACTIONS', 'RUNNER_ENVIRONMENT', 'RUNNER_OS', 'RUNNER_ARCH', 'GITHUB_REPOSITORY_ID', 'GITHUB_RUN_ID', 'RUNNER_TEMP')
$original = @{}
foreach ($name in $guardNames) { $original[$name] = [Environment]::GetEnvironmentVariable($name) }
$evidenceRoot = Join-Path ([IO.Path]::GetTempPath()) ('limo-cad-shell-preflight-fake-' + [Guid]::NewGuid())
$preflight = Join-Path $PSScriptRoot '../prepare-hosted-arm-desktop.ps1'
function Invoke-Case([string]$name, [string]$expected, [int]$closes, [long]$window = 0, [switch]$identify) {
    $path = Join-Path $evidenceRoot ($name + '.json')
    $caught = $null
    try { & $preflight -EvidencePath $path -Window $window -IdentifyOnly:$identify } catch { $caught = $_ }
    $report = Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    if ($report.status -ne $expected -or [HostedArmAccountWindow]::Closed.Count -ne $closes -or
        (($expected -eq 'failed') -ne ($null -ne $caught))) {
        throw "$name expected $expected/$closes, got $($report.status)/$([HostedArmAccountWindow]::Closed.Count): $caught"
    }
    if (-not $report.started_utc -or -not $report.finished_utc) { throw "$name omitted timing evidence" }
    $report
}
try {
    $env:GITHUB_ACTIONS = 'true'
    $env:RUNNER_ENVIRONMENT = 'github-hosted'
    $env:RUNNER_OS = 'Windows'
    $env:RUNNER_ARCH = 'ARM64'
    $env:GITHUB_REPOSITORY_ID = '1313334315'
    $env:GITHUB_RUN_ID = '1234'
    $env:RUNNER_TEMP = $evidenceRoot

    foreach ($kind in @('Start', 'Search')) {
        Reset-Windows
        $null = Add-Shell 100 $kind
        [HostedArmAccountWindow]::Foreground = 100
        $report = Invoke-Case ($kind + '-foreground') 'closed' 1
        if ($report.shell_windows.Count -ne 1 -or $report.shell_windows[0].status -ne 'closed') { throw 'Shell closure evidence missing' }
    }
    Reset-Windows
    $null = Add-Shell 100 'Start'
    $null = Add-Shell 200 'Search'
    [HostedArmAccountWindow]::Foreground = 200
    $report = Invoke-Case 'distinct-occluder-and-foreground' 'closed' 2 100
    if ($report.shell_windows.Count -ne 2) { throw 'Both observed shell windows must be recorded' }

    Reset-Windows
    $null = Add-Shell 100 'Start'
    [HostedArmAccountWindow]::Foreground = 100
    $null = Invoke-Case 'same-hwnd-once' 'closed' 1 100

    Reset-Windows
    $null = Add-Shell 100 'Start'
    $unobserved = Add-Shell 200 'Search'
    $null = Invoke-Case 'only-observed-window' 'closed' 1 100
    if (-not $unobserved.Visible) { throw 'Unobserved shell window was touched' }

    foreach ($kind in @('Start', 'Search')) {
        Reset-Windows
        $value = Add-Shell 100 $kind
        $value.Executable = Join-Path $evidenceRoot ($value.ProcessName + '.exe')
        $null = Invoke-Case ($kind + '-forged-path-refused') 'failed' 0 100
    }
    Reset-Windows
    $value = Add-Shell 100 'Search'
    $value.ProcessName = 'UnrelatedProcess'
    $null = Invoke-Case 'wrong-process-refused' 'failed' 0 100

    Reset-Windows
    $value = Add-Shell 100 'Start'
    $value.ClassName = 'UnrelatedClass'
    $null = Invoke-Case 'wrong-class-untouched' 'not_present' 0 100

    Reset-Windows
    $value = Add-Shell 100 'Start'
    $value.Title = 'Unrelated title'
    $null = Invoke-Case 'wrong-title-untouched' 'not_present' 0 100

    Reset-Windows
    $value = Add-Shell 100 'Start'
    $value.ChangeOwner = $true
    $null = Invoke-Case 'owner-change-refused' 'failed' 0 100

    Reset-Windows
    $null = Add-Shell 100 'Start'
    $null = Add-Shell 200 'Search'
    [HostedArmAccountWindow]::Windows[200].Executable = Join-Path $evidenceRoot 'SearchHost.exe'
    [HostedArmAccountWindow]::Foreground = 200
    $null = Invoke-Case 'validate-all-before-closing-any' 'failed' 0 100

    Reset-Windows
    $value = Add-Shell 100 'Start'
    $value.RejectClose = $true
    $null = Invoke-Case 'unacknowledged-close-fails' 'failed' 1 100

    Reset-Windows
    $value = Add-Shell 100 'Search'
    $value.StayVisible = $true
    $report = Invoke-Case 'persistent-shell-hidden' 'closed' 1 100
    if ($report.shell_windows[0].fallback -ne 'SW_HIDE') { throw 'Shell hide fallback evidence missing' }

    Reset-Windows
    $value = Add-Shell 100 'Search'
    $value.StayVisible = $true
    $value.RejectHide = $true
    $null = Invoke-Case 'visible-after-close-fails' 'failed' 1 100

    Reset-Windows
    $null = Add-Shell 100 'Start'
    $report = Invoke-Case 'identify-only-never-closes' 'not_present' 0 100 -identify
    if ($report.occluder.title -ne 'Start') { throw 'Identity-only evidence missing' }

    foreach ($guard in $guardNames | Where-Object { $_ -ne 'RUNNER_TEMP' }) {
        $before = [Environment]::GetEnvironmentVariable($guard)
        [Environment]::SetEnvironmentVariable($guard, 'invalid')
        $path = Join-Path $evidenceRoot ($guard + '-refused.json')
        $refused = $false
        try { & $preflight -EvidencePath $path -Window 100 } catch { $refused = $true }
        [Environment]::SetEnvironmentVariable($guard, $before)
        if (-not $refused -or (Test-Path -LiteralPath $path) -or [HostedArmAccountWindow]::Closed.Count -ne 0) {
            throw "$guard did not refuse before window actions or evidence writes"
        }
    }
    $escaped = Join-Path ([IO.Path]::GetTempPath()) ('limo-cad-refused-' + [Guid]::NewGuid() + '.json')
    $refused = $false
    try { & $preflight -EvidencePath $escaped -Window 100 } catch { $refused = $true }
    if (-not $refused -or (Test-Path -LiteralPath $escaped) -or [HostedArmAccountWindow]::Closed.Count -ne 0) { throw 'Evidence path escape did not fail closed' }
    Write-Output 'PASS: 23 managed shell-preflight cases; no desktop APIs invoked'
} finally {
    foreach ($name in $guardNames) { [Environment]::SetEnvironmentVariable($name, $original[$name]) }
}
