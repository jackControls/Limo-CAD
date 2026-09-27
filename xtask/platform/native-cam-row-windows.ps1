# Dot-sourced only after native-input-windows.ps1 has resolved/focused one
# owned Winit window and loaded its SendInput declarations.
param([int]$CamOwnedPid, [IntPtr]$CamWindow)
$camRequest = [Console]::In.ReadToEnd() | ConvertFrom-Json
$camClientRect = [NativePlatformInput+RECT]::new()
if (-not [NativePlatformInput]::GetClientRect($CamWindow, [ref]$camClientRect)) { throw 'Cannot locate owned CAM client' }
function Assert-CamFinite($value) {
    if ($null -eq $value -or $value -is [bool] -or -not ($value -is [ValueType]) -or [double]::IsNaN([double]$value) -or [double]::IsInfinity([double]$value)) { throw 'CAM gesture coordinates must be finite numbers' }
}
foreach ($value in @($camRequest.client.x,$camRequest.client.y,$camRequest.client.width,$camRequest.client.height)) { Assert-CamFinite $value }
if ($camRequest.client.width -le 0 -or $camRequest.client.height -le 0) { throw 'CAM client dimensions must be positive' }
$camScaleX = $camClientRect.right / [double]$camRequest.client.width
$camScaleY = $camClientRect.bottom / [double]$camRequest.client.height
if ($camScaleX -lt .5 -or $camScaleX -gt 4 -or $camScaleY -lt .5 -or $camScaleY -gt 4 -or [Math]::Abs($camScaleX-$camScaleY) -gt .01) { throw 'Owned CAM client scale differs from published bounds' }
if ($camRequest.cancel -isnot [bool]) { throw 'CAM cancel must be a boolean' }
$camWaypoints = @($camRequest.points)
if ($camWaypoints.Count -lt 1 -or $camWaypoints.Count -gt 8) { throw 'CAM drag requires 1..8 bounded waypoints' }
$camTotalHold = 0
foreach ($waypoint in $camWaypoints) {
    Assert-CamFinite $waypoint.hold_ms
    if ($waypoint.hold_ms -lt 0 -or $waypoint.hold_ms -gt 800 -or [Math]::Floor($waypoint.hold_ms) -ne $waypoint.hold_ms) { throw 'CAM waypoint hold must be 0..800 integral milliseconds' }
    $camTotalHold += $waypoint.hold_ms
}
if ($camTotalHold -gt 1600) { throw 'CAM total dwell exceeds 1600 milliseconds' }
function Resolve-CamPoint($x, $y) {
    Assert-CamFinite $x
    Assert-CamFinite $y
    $localX = [double]$x - $camRequest.client.x
    $localY = [double]$y - $camRequest.client.y
    if ($localX -lt 0 -or $localY -lt 0 -or $localX -ge $camRequest.client.width -or $localY -ge $camRequest.client.height) { throw 'CAM gesture point lies outside the owned client' }
    $point = [NativePlatformInput+POINT]::new()
    $point.x = [int][Math]::Round($localX * $camScaleX)
    $point.y = [int][Math]::Round($localY * $camScaleY)
    if ($point.x -lt 0 -or $point.y -lt 0 -or $point.x -ge $camClientRect.right -or $point.y -ge $camClientRect.bottom) { throw 'Rounded CAM gesture point lies outside the owned client' }
    $logical = @(($camRequest.client.x + $point.x / $camScaleX), ($camRequest.client.y + $point.y / $camScaleY))
    if (-not [NativePlatformInput]::ClientToScreen($CamWindow, [ref]$point)) { throw 'Cannot map owned CAM client coordinate' }
    return [pscustomobject]@{ point=$point; logical=$logical }
}
function Assert-CamRecipient($point) {
    $currentRect = [NativePlatformInput+RECT]::new()
    if (-not [NativePlatformInput]::GetClientRect($CamWindow, [ref]$currentRect) -or $currentRect.right -ne $camClientRect.right -or $currentRect.bottom -ne $camClientRect.bottom) { throw 'Owned CAM client changed size during gesture' }
    [uint32]$owner = 0
    $recipient = [NativePlatformInput]::WindowFromPoint($point)
    [void][NativePlatformInput]::GetWindowThreadProcessId($recipient, [ref]$owner)
    if ($owner -ne $CamOwnedPid -or $recipient -ne $CamWindow -or [NativePlatformInput]::GetForegroundWindow() -ne $CamWindow) { throw 'Owned CAM target is occluded or lost focus; no further input sent' }
}
function Move-CamPoint($x, $y) {
    $resolved = Resolve-CamPoint $x $y
    Assert-CamRecipient $resolved.point
    if (-not [NativePlatformInput]::SetCursorPos($resolved.point.x, $resolved.point.y)) { throw 'Cannot move owned CAM pointer' }
    $actual = [NativePlatformInput+POINT]::new()
    if (-not [NativePlatformInput]::GetCursorPos([ref]$actual) -or $actual.x -ne $resolved.point.x -or $actual.y -ne $resolved.point.y) { throw 'Owned CAM pointer did not reach its physical target' }
    Start-Sleep -Milliseconds 35
}
# Validate all coordinates and recipients before pressing the button. Guard
# them again during each step/dwell in case another window takes ownership.
$camStart = Resolve-CamPoint $camRequest.x $camRequest.y
Assert-CamRecipient $camStart.point
foreach ($waypoint in $camWaypoints) { $resolved = Resolve-CamPoint $waypoint.x $waypoint.y; Assert-CamRecipient $resolved.point }
Move-CamPoint $camRequest.x $camRequest.y
[NativePlatformInput]::Mouse(2)
try {
    $fromX = [double]$camRequest.x
    $fromY = [double]$camRequest.y
    foreach ($waypoint in $camWaypoints) {
        for ($step=1; $step -le 6; $step++) {
            Move-CamPoint ($fromX + ($waypoint.x-$fromX)*$step/6) ($fromY + ($waypoint.y-$fromY)*$step/6)
        }
        $deadline = [DateTime]::UtcNow.AddMilliseconds($waypoint.hold_ms)
        do {
            $resolved = Resolve-CamPoint $waypoint.x $waypoint.y
            Assert-CamRecipient $resolved.point
            if ([DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 20 }
        } while ([DateTime]::UtcNow -lt $deadline)
        $fromX = $waypoint.x
        $fromY = $waypoint.y
    }
    if ($camRequest.cancel) {
        Assert-CamRecipient (Resolve-CamPoint $fromX $fromY).point
        [NativePlatformInput]::Key(0x1B, $false)
        [NativePlatformInput]::Key(0x1B, $true)
        Start-Sleep -Milliseconds 100
    }
} finally { [NativePlatformInput]::Mouse(4) }
Start-Sleep -Milliseconds 150
$camEnd = Resolve-CamPoint $fromX $fromY
[pscustomobject]@{source='Windows SendInput';pid=$CamOwnedPid;window=$CamWindow.ToInt64();operation='cam-row-drag';cancel=$camRequest.cancel;
    client=$camRequest.client;scale=@($camScaleX,$camScaleY);points=$camWaypoints;logical_start=$camStart.logical;logical_end=$camEnd.logical;
    physical_start=@($camStart.point.x,$camStart.point.y);physical_end=@($camEnd.point.x,$camEnd.point.y)} | ConvertTo-Json -Depth 8 -Compress
