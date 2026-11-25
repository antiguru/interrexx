# SPDX-License-Identifier: MIT OR Apache-2.0
$ErrorActionPreference = 'Stop'

$suffix = [Guid]::NewGuid().ToString('N')
$first = 'interrexx-a-' + $suffix
$second = 'interrexx-b-' + $suffix
$spooler = Get-Service -Name Spooler
$wasRunning = $spooler.Status -eq 'Running'
$wasDisabled = $spooler.StartType -eq 'Disabled'
$previous = $null
try {
    if ($wasDisabled) {
        Set-Service -Name Spooler -StartupType Manual
    }
    if (-not $wasRunning) {
        Start-Service -Name Spooler
    }
    $previous = Get-CimInstance Win32_Printer -Filter 'Default = TRUE' | Select-Object -First 1
    $driver = Get-PrinterDriver | Select-Object -First 1
    if (-not $driver) {
        Add-PrinterDriver -Name 'Generic / Text Only'
        $driver = Get-PrinterDriver -Name 'Generic / Text Only'
    }

    # These are local queues with no physical printer and the test submits no jobs.
    Add-Printer -Name $first -DriverName $driver.Name -PortName 'FILE:'
    Add-Printer -Name $second -DriverName $driver.Name -PortName 'FILE:'
    & ./target/debug/rexx.exe crates/interrexx/tests/windows-rexxutil-printers.rex $first $second
    if ($LASTEXITCODE -ne 0) {
        throw "RexxUtil printer fixture failed at check $LASTEXITCODE"
    }
    Write-Host 'RexxUtil printer enumeration and both default setter forms passed.'
} finally {
    try {
        foreach ($name in $second, $first) {
            if (Get-Printer -Name $name -ErrorAction SilentlyContinue) {
                Remove-Printer -Name $name
            }
        }
        if ($previous) {
            $restored = Invoke-CimMethod -InputObject $previous -MethodName SetDefaultPrinter
            if ($restored.ReturnValue -ne 0) {
                throw "Restoring the previous default printer failed ($($restored.ReturnValue))"
            }
        }
    } finally {
        if (-not $wasRunning) {
            Stop-Service -Name Spooler
        }
        if ($wasDisabled) {
            Set-Service -Name Spooler -StartupType Disabled
        }
    }
}
