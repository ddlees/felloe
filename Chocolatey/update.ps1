Import-Module AU

$releases = 'https://github.com/ddlees/felloe/releases'

function global:au_SearchReplace {
    @{
        ".\tools\chocolateyinstall.ps1" = @{
            "(^[$]url\s*=\s*)('.*')" = "`$1'$($Latest.URL32)'"
            "(?i)(^\s*checksum\s*=\s*)('.*')" = "`$1'$($Latest.Checksum32)'"
        }
        "felloe.nuspec" = @{
            "\d+\.\d+\.\d+" = "$($Latest.Version)"
        }
    }
}

function global:au_GetLatest {
    $download_page = Invoke-WebRequest -Uri $releases -UseBasicParsing
    $url_base = 'https://github.com'
    $urls =  $download_page.Links | Where-Object href -match '-windows-gnu' | ForEach-Object href | Select-Object -First 1
    $url32 = $url_base + $urls.Where({ $_ -match 64 })
    $version = $url32 -split '/' | Select-Object -Last 1 -Skip 1

    @{
        URL32 = $url32
        Version = $version
    }
}

Update-Package
