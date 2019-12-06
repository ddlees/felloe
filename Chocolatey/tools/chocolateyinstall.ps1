$ErrorActionPreference = 'Stop';

$version     = $env:chocolateyPackageVersion
$packageName = $env:chocolateyPackageName
$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$url        = 'https://github.com/ddlees/felloe/releases/download/v0.1.0/felloe-x86_64-windows-gnu.zip'

$packageArgs = @{
  packageName   = $packageName
  unzipLocation = $toolsDir
  fileType      = 'exe'
  url           = $url
  checksum      = 'b5eb700bbcd55dc56702919e5db766028b47ef5dde081920d5cd900eb6ee037b'
  checksumType  = 'sha256'

  validExitCodes= @(0)
}

Install-ChocolateyZipPackage @packageArgs
