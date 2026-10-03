[CmdletBinding()]
param([string]$BundleDirectory = 'app/src-tauri/target/release/bundle/nsis')
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Run installer smoke only in a disposable Windows CI user/VM.' }
$installers = @(Get-ChildItem -Path $BundleDirectory -Filter '*-setup.exe' -File)
if ($installers.Count -eq 0) { throw 'No packaged NSIS installer found.' }
foreach ($installer in $installers) {
  $installRoot = Join-Path $env:TEMP ('telegram-drive-installer-smoke-' + [guid]::NewGuid().ToString('N'))
  try {
    # NSIS requires /D last and without surrounding quotes. This generated TEMP
    # path is not accepted from untrusted caller input.
    $installation = Start-Process -FilePath $installer.FullName -ArgumentList @('/S', "/D=$installRoot") -PassThru
    if (-not $installation.WaitForExit(120000)) { Stop-Process -Id $installation.Id -Force; throw 'Installer smoke timed out.' }
    if ($installation.ExitCode -ne 0) { throw "Installer failed: $($installation.ExitCode)" }
    $applications = @(Get-ChildItem -Path $installRoot -Filter '*.exe' -File | Where-Object { $_.Name -notmatch 'uninstall|vc_redist' })
    if ($applications.Count -ne 1) { throw "Expected one installed application, found $($applications.Count)" }
    & node (Join-Path $PSScriptRoot 'packaged-startup-smoke.cjs') --disposable-user --executable $applications[0].FullName
    if ($LASTEXITCODE -ne 0) { throw 'Packaged Windows startup readiness failed.' }
  } finally { if (Test-Path $installRoot) { Remove-Item -Recurse -Force $installRoot } }
}
# Registry/shortcuts/keyring effects are confined to the disposable CI VM, not
# considered isolated by /D or environment-variable overrides alone.
