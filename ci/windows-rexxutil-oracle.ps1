$ErrorActionPreference = 'Stop'
$archive = Join-Path $env:RUNNER_TEMP 'oorexx-oracle.zip'
$bundle = Join-Path $env:RUNNER_TEMP 'oorexx-oracle'
$url = 'https://downloads.sourceforge.net/project/oorexx/oorexx/5.2.0/portable/oorexx-5.2.0-13156.windows.x86_64-portable-release.zip?use_mirror=netix'
curl.exe --fail --location --silent --show-error --max-time 675 --output $archive $url
if ($LASTEXITCODE -ne 0) { throw 'Could not download the ooRexx reference' }
$hash = (Get-FileHash -Algorithm SHA256 $archive).Hash
if ($hash -ne 'd7ea5b40f0f889d0b307bbc217fe9fb36bc3e3fd0688a829668ce681a350203e') {
    throw "Unexpected ooRexx archive hash $hash"
}
Expand-Archive -Path $archive -DestinationPath $bundle
$oracle = Get-ChildItem -LiteralPath $bundle -Recurse -Filter rexx.exe | Select-Object -First 1
if (-not $oracle) { throw 'The ooRexx archive did not contain rexx.exe' }
$env:PATH = "$($oracle.DirectoryName);$env:PATH"
$script = Join-Path $PSScriptRoot 'windows-rexxutil-oracle.rex'
$fixture = Join-Path $env:RUNNER_TEMP 'rexxutil-oracle-fixture'
New-Item -ItemType Directory -Path $fixture | Out-Null
try {
    $probe = Start-Process $oracle.FullName -NoNewWindow -PassThru -ArgumentList @(
        ('"' + $script + '"'), ('"' + $fixture + '"')
    )
    if (-not $probe.WaitForExit(30000)) {
        $probe.Kill()
        throw 'The ooRexx RexxUtil probe timed out'
    }
    if ($probe.ExitCode -ne 0) { throw "ooRexx RexxUtil probe exited $($probe.ExitCode)" }
} finally {
    Remove-Item -LiteralPath $fixture -Recurse -Force
}
