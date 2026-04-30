$ErrorActionPreference = 'Stop';
$toolsDir = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
Remove-Item -Path "$toolsDir\lukuid-cli.exe" -Force -ErrorAction SilentlyContinue
