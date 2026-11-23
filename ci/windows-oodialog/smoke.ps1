# SPDX-License-Identifier: MIT OR Apache-2.0
Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class DialogWindow {
    [DllImport("user32.dll")]
    public static extern uint GetWindowThreadProcessId(IntPtr window, out uint processId);

    [DllImport("user32.dll", SetLastError = true)]
    public static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);
}
'@

$start = [Diagnostics.ProcessStartInfo]::new((Join-Path $PWD 'rexx.exe'))
$start.ArgumentList.Add('smoke.rex')
$start.WorkingDirectory = (Get-Location).Path
$start.UseShellExecute = $false
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$process = [Diagnostics.Process]::Start($start)
$handle = $process.StandardOutput.ReadLineAsync()
$stdout = $null
$stderr = $process.StandardError.ReadToEndAsync()
try {
    if (!$handle.Wait(20000) -or $handle.Result -notmatch '^HWND 0x([0-9a-fA-F]+)$') {
        throw 'The ooDialog smoke window did not report a handle'
    }
    $window = [IntPtr][Convert]::ToInt64($Matches[1], 16)
    [uint32]$owner = 0
    $null = [DialogWindow]::GetWindowThreadProcessId($window, [ref]$owner)
    if ($owner -ne $process.Id) {
        throw 'The reported window does not belong to the dialog process'
    }
    $stdout = $process.StandardOutput.ReadToEndAsync()
    # WM_COMMAND with IDOK sends a standard dialog command across threads.
    if (![DialogWindow]::PostMessage($window, 0x0111, [IntPtr]1, [IntPtr]::Zero)) {
        throw 'Could not post the dialog command'
    }
    if (!$process.WaitForExit(20000)) {
        throw 'The dialog command did not close the window'
    }
    if ($process.ExitCode -ne 0 -or $stdout.Result.Trim() -cne 'ooDialog command event passed') {
        throw "The dialog callback failed ($($process.ExitCode))"
    }
} finally {
    if (!$process.HasExited) { $process.Kill() }
    $process.WaitForExit()
    if ($handle.IsCompletedSuccessfully) { Write-Host $handle.Result }
    if ($null -ne $stdout) { Write-Host $stdout.Result }
    Write-Host $stderr.Result
    $process.Dispose()
}
