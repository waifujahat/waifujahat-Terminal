/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Shell process tracking
 * Purpose : Discovers descendant shell processes
 */

use std::collections::{HashMap, HashSet, VecDeque};

use anyhow::Result;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProsesTerminal {
    pub id: u32,
    pub id_induk: u32,
    pub nama_program: String,
}

impl ProsesTerminal {
    pub fn merupakan_shell(&self) -> bool {
        matches!(
            self.nama_program.to_ascii_lowercase().as_str(),
            "powershell.exe" | "pwsh.exe" | "cmd.exe" | "bash.exe" | "wsl.exe"
        )
    }
}

pub fn cari_shell_turunan(akar: u32, daftar: &[ProsesTerminal]) -> Vec<ProsesTerminal> {
    if akar == 0 || !daftar.iter().any(|proses| proses.id == akar) {
        return Vec::new();
    }

    let mut indeks = HashMap::<u32, Vec<&ProsesTerminal>>::new();

    for proses in daftar {
        indeks.entry(proses.id_induk).or_default().push(proses);
    }

    let mut antrean = VecDeque::from([akar]);

    let mut dikunjungi = HashSet::from([akar]);

    let mut hasil = Vec::new();

    while let Some(induk) = antrean.pop_front() {
        let Some(turunan) = indeks.get(&induk) else {
            continue;
        };

        for &proses in turunan {
            if !dikunjungi.insert(proses.id) {
                continue;
            }

            antrean.push_back(proses.id);

            if proses.merupakan_shell() {
                hasil.push(proses.clone());
            }
        }
    }

    hasil
}

#[cfg(windows)]
pub(super) fn potret_proses() -> Result<Vec<ProsesTerminal>> {
    use std::io;
    use std::mem::size_of;
    use std::os::windows::io::{FromRawHandle, OwnedHandle};

    use windows_sys::Win32::{
        Foundation::{ERROR_NO_MORE_FILES, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };

    // SAFETY : Membuat snapshot proses melalui Windows API.
    let handle = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };

    if handle == INVALID_HANDLE_VALUE {
        return Err(io::Error::last_os_error().into());
    }

    // SAFETY : Handle valid dan kepemilikannya dipindahkan ke OwnedHandle.
    let _penjaga = unsafe { OwnedHandle::from_raw_handle(handle) };

    let mut entri = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };

    let mut daftar = Vec::new();

    // SAFETY : Handle valid dan struktur memiliki ukuran yang sesuai.
    let mut berhasil = unsafe { Process32FirstW(handle, &mut entri) };

    while berhasil != 0 {
        let panjang = entri
            .szExeFile
            .iter()
            .position(|karakter| *karakter == 0)
            .unwrap_or(entri.szExeFile.len());

        daftar.push(ProsesTerminal {
            id: entri.th32ProcessID,
            id_induk: entri.th32ParentProcessID,
            nama_program: String::from_utf16_lossy(&entri.szExeFile[..panjang]),
        });

        // SAFETY : Handle dan struktur tetap valid selama iterasi.
        berhasil = unsafe { Process32NextW(handle, &mut entri) };
    }

    let kesalahan = io::Error::last_os_error();

    if kesalahan.raw_os_error() != Some(ERROR_NO_MORE_FILES as i32) {
        return Err(kesalahan.into());
    }

    Ok(daftar)
}

#[cfg(not(windows))]
pub(super) fn potret_proses() -> Result<Vec<ProsesTerminal>> {
    anyhow::bail!("Pelacakan proses belum tersedia pada platform ini")
}
