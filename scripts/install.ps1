$ErrorActionPreference = 'Stop'

$Repo = 'https://github.com/paternosterrack/pater.git'
$CargoBin = Join-Path $HOME '.cargo\bin'

function Has-Cmd($name) {
  return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

Write-Host '[pater] starting install...'

if (-not (Has-Cmd cargo)) {
  Write-Host '[pater] cargo not found. installing rustup toolchain...'
  irm https://win.rustup.rs/x86_64 | iex
  $env:Path = "$CargoBin;$env:Path"
}

Write-Host '[pater] installing latest pater from git...'
& cargo install --locked --git $Repo pater --force

if (-not ($env:Path -split ';' | Where-Object { $_ -eq $CargoBin })) {
  Write-Host "[pater] add to PATH if needed: $CargoBin"
}

& "$CargoBin\pater.exe" --version
