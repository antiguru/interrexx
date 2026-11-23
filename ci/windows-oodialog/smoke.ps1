# SPDX-License-Identifier: MIT OR Apache-2.0
Add-Type @'
using System;
using System.Runtime.InteropServices;

public static class DialogWindow {
    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    public static extern IntPtr FindWindow(string className, string title);

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
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
try {
    $window = [IntPtr]::Zero
    $deadline = [DateTime]::UtcNow.AddSeconds(20)
    while (!$process.HasExited -and [DateTime]::UtcNow -lt $deadline) {
        $candidate = [DialogWindow]::FindWindow($null, 'InterRexx ooDialog smoke')
        if ($candidate -ne [IntPtr]::Zero) {
            [uint32]$owner = 0
            $null = [DialogWindow]::GetWindowThreadProcessId($candidate, [ref]$owner)
            if ($owner -eq $process.Id) {
                $window = $candidate
                break
            }
        }
        Start-Sleep -Milliseconds 100
    }
    if ($window -eq [IntPtr]::Zero) {
        throw 'The ooDialog smoke window did not open'
    }
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
    Write-Host $stdout.Result
    Write-Host $stderr.Result
    $process.Dispose()
}
