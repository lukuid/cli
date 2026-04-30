$ErrorActionPreference = 'Stop';
$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url        = '__URL__'
$checksum   = '__CHECKSUM__'

Install-ChocolateyZipPackage -PackageName 'lukuid-cli' `
                             -Url $url `
                             -UnzipLocation $toolsDir `
                             -Checksum $checksum `
                             -ChecksumType 'sha256'
