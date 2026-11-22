<#
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / PowerShell integration
 * Purpose: Reports prompt boundaries and command completion through OSC 133
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

try {
    $lokasi = $ExecutionContext.SessionState.Path.CurrentLocation

    if ($lokasi.Provider.Name -eq 'FileSystem') {
        $jalur = $lokasi.ProviderPath

        if ($jalur -match '^[A-Za-z]:\\') {
            $drive = $jalur.Substring(0, 2)

            $bagianUri = foreach ($bagian in $jalur.Substring(3).Split([char]'\')) {
                [Uri]::EscapeDataString($bagian)
            }

            $uri = 'file:///{0}/{1}' -f $drive, ($bagianUri -join '/')

            [Console]::Write(
                ('{0}]7;{1}{2}' -f $escape, $uri, $bell)
            )
        }
    }
}
    catch {
        # Directory reporting must not interrupt the shell prompt.
        $global:WaifujahatKesalahanTerakhir = $Error[0]
    }

    [Console]::Write(
        ('{0}]133;A{1}' -f $escape, $bell)
    )

    & $global:WaifujahatPromptAsli
}

[Console]::WriteLine()
[Console]::WriteLine('Waifujahat Terminal 0.1.0')
[Console]::WriteLine('Copyright (C) 2026 Nurwidhi Maulana Nugraha (waifujahat)')
[Console]::WriteLine()
