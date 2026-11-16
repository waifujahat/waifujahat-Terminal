<#
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / PowerShell integration
 * Purpose: Reports command completion through OSC 133
#>

$global:WaifujahatRiwayatTerakhir = -1
$global:WaifujahatKesalahanTerakhir = $null
$global:WaifujahatPromptAsli = $function:prompt

function global:prompt {
    $berhasil = $?
    $kodeNative = $global:LASTEXITCODE

    $kesalahanTerbaru = $null

    if ($Error.Count -gt 0) {
        $kesalahanTerbaru = $Error[0]
    }

    $kesalahanBaru = (
        $null -ne $kesalahanTerbaru -and
        -not [object]::ReferenceEquals(
            $kesalahanTerbaru,
            $global:WaifujahatKesalahanTerakhir
        )
    )

    $riwayat = Get-History -Count 1
    $indeks = 0

    if ($null -ne $riwayat) {
        $indeks = $riwayat.Id
    }

    $escape = [char]27
    $bell = [char]7

    if ($global:WaifujahatRiwayatTerakhir -ne -1) {
        if ($indeks -eq $global:WaifujahatRiwayatTerakhir) {
            [Console]::Write(
                ('{0}]133;D{1}' -f $escape, $bell)
            )
        }
        else {
            $kode = 0

            if (-not $berhasil) {
                $kesalahanPowerShell = (
                    $kesalahanBaru -and
                    $kesalahanTerbaru.FullyQualifiedErrorId -notlike
                        'NativeCommandError*'
                )

                if ($kesalahanPowerShell) {
                    $kode = 1
                }
                elseif ($null -ne $kodeNative -and $kodeNative -ne 0) {
                    $kode = $kodeNative
                }
                else {
                    $kode = 1
                }
            }

            [Console]::Write(
                ('{0}]133;D;{1}{2}' -f $escape, $kode, $bell)
            )
        }
    }

    $global:WaifujahatRiwayatTerakhir = $indeks
    $global:WaifujahatKesalahanTerakhir = $kesalahanTerbaru

    & $global:WaifujahatPromptAsli
}
