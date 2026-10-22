# Run from PowerShell on Windows x64; also used by GitHub Actions.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if ($env:OS -ne 'Windows_NT') {
    throw 'Build this installer on Windows with the x64 MSVC Rust toolchain.'
}

Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $toolchain = & rustc -vV
    if ($LASTEXITCODE -ne 0 -or $toolchain -notcontains 'host: x86_64-pc-windows-msvc') {
        throw 'Install Rust stable x86_64-pc-windows-msvc and Visual Studio C++ Build Tools.'
    }
    & npm.cmd ci
    if ($LASTEXITCODE -ne 0) { throw 'npm ci failed.' }
    & npm.cmd run bundle:windows -- --ci -- --locked
    if ($LASTEXITCODE -ne 0) { throw 'Windows installer build failed.' }

    $config = Get-Content src-tauri/tauri.conf.json -Raw | ConvertFrom-Json
    $version = [Version]$config.version
    $binary = Resolve-Path src-tauri/target/release/stock-asset-metadata.exe
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($binary)
    if ($info.FileMajorPart -ne $version.Major -or $info.FileMinorPart -ne $version.Minor -or $info.FileBuildPart -ne $version.Build) {
        throw 'Executable version does not match the application version.'
    }
    $bytes = [IO.File]::ReadAllBytes($binary)
    $peOffset = [BitConverter]::ToInt32($bytes, 0x3c)
    if ([BitConverter]::ToUInt16($bytes, $peOffset + 4) -ne 0x8664) {
        throw 'Expected a Windows x64 executable.'
    }
    $installers = @(Get-ChildItem "src-tauri/target/release/bundle/nsis/*_$($version.ToString())_x64-setup.exe" -File)
    if ($installers.Count -ne 1) { throw 'Expected exactly one NSIS installer.' }
    $installer = $installers[0]
    $hash = (Get-FileHash $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
    $checksum = "$($installer.FullName).sha256"
    [IO.File]::WriteAllText($checksum, "$hash  $($installer.Name)`n", [Text.UTF8Encoding]::new($false))
    $expected = (Get-Content $checksum -Raw).Split(' ')[0]
    if ((Get-FileHash $installer.FullName -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
        throw 'Installer checksum verification failed.'
    }
    Write-Host "Installer: $($installer.FullName)"
    Write-Host "Checksum: $checksum"
} finally {
    Pop-Location
}
