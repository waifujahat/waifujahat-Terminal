/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / PTY session
 * Purpose: Owns the PowerShell process, PTY, resizing, and worker lifetime.
 */

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

mod io;

use super::{
    profil_shell::ProfilShell,
    proses_shell::{ProsesTerminal, cari_shell_turunan, potret_proses},
    ssh::{ClientSsh, ProfilSsh},
    tombol::TombolTerminal,
};

use io::{jalankan_pembaca, jalankan_penulis};

use anyhow::{Context, Result, anyhow};
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};

#[derive(Debug)]
pub enum PesanTerminal {
    Keluaran(Vec<u8>),
    Kesalahan(String),
    Berakhir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusProsesTerminal {
    Berjalan,
    Berakhir { kode_keluar: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JenisSesiTerminal {
    Lokal(ProfilShell),
    Ssh(ProfilSsh),
}

impl JenisSesiTerminal {
    pub fn nama(&self) -> &str {
        match self {
            Self::Lokal(profil) => profil.nama(),
            Self::Ssh(profil) => &profil.nama,
        }
    }
}

pub struct SesiTerminal {
    jenis: JenisSesiTerminal,
    direktori_awal: Option<PathBuf>,
    master: Option<Box<dyn MasterPty + Send>>,
    proses: Box<dyn Child + Send + Sync>,
    kode_keluar: Option<u32>,
    pengirim_masukan: Sender<Vec<u8>>,
    penerima_peristiwa: Receiver<PesanTerminal>,
}

impl SesiTerminal {
    pub fn baru() -> Result<Self> {
        let profil = ProfilShell::default_tersedia()
            .ok_or_else(|| anyhow!("Tidak ada shell yang tersedia"))?;

        Self::baru_dengan_profil(profil)
    }

    pub fn baru_dengan_profil(profil: ProfilShell) -> Result<Self> {
        Self::baru_dengan_direktori(profil, None)
    }

    pub fn baru_dengan_direktori(profil: ProfilShell, direktori: Option<&Path>) -> Result<Self> {
        let program = profil
            .lokasi_program()
            .ok_or_else(|| anyhow!("Shell {} tidak tersedia", profil.nama()))?;

        let mut perintah = CommandBuilder::new(program);

        let direktori_awal = direktori
            .filter(|jalur| Self::direktori_awal_valid(jalur))
            .map(Path::to_path_buf);

        if let Some(jalur) = direktori_awal.as_deref() {
            perintah.cwd(jalur);

            if profil == ProfilShell::GitBash {
                perintah.env("CHERE_INVOKING", "1");
            }
        }

        match profil {
            ProfilShell::WindowsPowerShell | ProfilShell::PowerShell7 => {
                perintah.arg("-NoLogo");
                perintah.arg("-NoProfile");
                perintah.arg("-NoExit");
                perintah.arg("-Command");
                perintah.arg(include_str!("integrasi_powershell.ps1"));
            }

            ProfilShell::Cmd => {
                perintah.arg("/D");
            }

            ProfilShell::GitBash => {
                perintah.arg("--login");
                perintah.arg("-i");
            }
        }

        Self::jalankan(JenisSesiTerminal::Lokal(profil), perintah, direktori_awal)
    }

    pub fn baru_dengan_ssh(profil: &ProfilSsh) -> Result<Self> {
        Self::baru_dengan_opsi_ssh(profil, None)
    }

    pub fn baru_dengan_ssh_dan_known_hosts(profil: &ProfilSsh, known_hosts: &Path) -> Result<Self> {
        Self::baru_dengan_opsi_ssh(profil, Some(known_hosts))
    }

    fn baru_dengan_opsi_ssh(profil: &ProfilSsh, known_hosts: Option<&Path>) -> Result<Self> {
        profil.validasi()?;

        let client = ClientSsh::deteksi().context("OpenSSH Client tidak tersedia")?;

        let perintah = match known_hosts {
            Some(jalur) => client.perintah_dengan_known_hosts(profil, jalur)?,

            None => client.perintah(profil)?,
        };

        Self::jalankan(JenisSesiTerminal::Ssh(profil.clone()), perintah, None)
    }

    fn jalankan(
        jenis: JenisSesiTerminal,
        perintah: CommandBuilder,
        direktori_awal: Option<PathBuf>,
    ) -> Result<Self> {
        let sistem_pty = native_pty_system();

        let pasangan = sistem_pty
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Gagal membuat pseudoterminal")?;

        let mut proses = pasangan
            .slave
            .spawn_command(perintah)
            .with_context(|| format!("Gagal menjalankan {}", jenis.nama()))?;

        drop(pasangan.slave);

        let hasil_io = Self::siapkan_io(pasangan.master.as_ref());

        let (pengirim_masukan, penerima_peristiwa) = match hasil_io {
            Ok(hasil) => hasil,

            Err(kesalahan) => {
                let _ = proses.kill();

                return Err(kesalahan);
            }
        };

        Ok(Self {
            jenis,
            direktori_awal,
            master: Some(pasangan.master),
            proses,
            kode_keluar: None,
            pengirim_masukan,
            penerima_peristiwa,
        })
    }

    fn siapkan_io(master: &dyn MasterPty) -> Result<(Sender<Vec<u8>>, Receiver<PesanTerminal>)> {
        let pembaca = master
            .try_clone_reader()
            .context("Gagal membuka pembaca terminal")?;

        let penulis = master
            .take_writer()
            .context("Gagal membuka penulis terminal")?;

        let (pengirim_masukan, penerima_masukan) = mpsc::channel();

        let (pengirim_peristiwa, penerima_peristiwa) = mpsc::channel();

        let pengirim_kesalahan = pengirim_peristiwa.clone();

        thread::Builder::new()
            .name("waifujahat-pty-writer".into())
            .spawn(move || {
                jalankan_penulis(penulis, penerima_masukan, pengirim_kesalahan);
            })
            .context("Gagal membuat thread penulis")?;

        thread::Builder::new()
            .name("waifujahat-pty-reader".into())
            .spawn(move || {
                jalankan_pembaca(pembaca, pengirim_peristiwa);
            })
            .context("Gagal membuat thread pembaca")?;

        Ok((pengirim_masukan, penerima_peristiwa))
    }

    fn direktori_awal_valid(direktori: &Path) -> bool {
        if !direktori.is_absolute() {
            return false;
        }

        #[cfg(windows)]
        {
            use std::path::{Component, Prefix};

            let drive_valid = matches!(
                direktori.components().next(),
                Some(Component::Prefix(awalan))
                    if matches!(awalan.kind(), Prefix::Disk(_))
            );

            if !drive_valid {
                return false;
            }
        }

        direktori.is_dir()
    }

    pub fn direktori_awal(&self) -> Option<&Path> {
        self.direktori_awal.as_deref()
    }

    pub fn jenis(&self) -> &JenisSesiTerminal {
        &self.jenis
    }

    pub fn profil(&self) -> Option<ProfilShell> {
        match &self.jenis {
            JenisSesiTerminal::Lokal(profil) => Some(*profil),
            JenisSesiTerminal::Ssh(_) => None,
        }
    }

    pub fn profil_ssh(&self) -> Option<&ProfilSsh> {
        match &self.jenis {
            JenisSesiTerminal::Lokal(_) => None,
            JenisSesiTerminal::Ssh(profil) => Some(profil),
        }
    }

    pub fn id_proses(&self) -> Option<u32> {
        self.proses.process_id()
    }

    pub fn status_proses(&mut self) -> Result<StatusProsesTerminal> {
        if let Some(kode_keluar) = self.kode_keluar {
            return Ok(StatusProsesTerminal::Berakhir { kode_keluar });
        }

        let status = self
            .proses
            .try_wait()
            .context("Gagal memeriksa status proses terminal")?;

        match status {
            Some(status) => {
                let kode_keluar = status.exit_code();

                self.kode_keluar = Some(kode_keluar);

                Ok(StatusProsesTerminal::Berakhir { kode_keluar })
            }

            None => Ok(StatusProsesTerminal::Berjalan),
        }
    }

    pub fn tutup_pty_setelah_proses_berakhir(&mut self) -> Result<()> {
        if self.kode_keluar.is_none() {
            return Err(anyhow!("Proses terminal masih berjalan"));
        }

        if let Some(master) = self.master.take() {
            thread::Builder::new()
                .name("waifujahat-pty-close".into())
                .spawn(move || {
                    drop(master);
                })
                .context("Gagal membuat thread penutupan terminal")?;
        }

        Ok(())
    }

    pub fn deteksi_shell_turunan(&self) -> Result<Vec<ProsesTerminal>> {
        if self.profil().is_none() {
            return Ok(Vec::new());
        }

        let Some(akar) = self.id_proses() else {
            return Ok(Vec::new());
        };

        let daftar = potret_proses()?;

        Ok(cari_shell_turunan(akar, &daftar))
    }

    pub fn kirim(&self, masukan: &str) -> Result<()> {
        self.kirim_bytes(masukan.as_bytes())
    }

    pub fn kirim_bytes(&self, masukan: &[u8]) -> Result<()> {
        if self.kode_keluar.is_some() {
            return Err(anyhow!("Proses terminal telah berakhir"));
        }

        self.pengirim_masukan
            .send(masukan.to_vec())
            .map_err(|_| anyhow!("Koneksi input terminal terputus"))
    }

    pub fn kirim_tombol(&self, tombol: TombolTerminal<'_>) -> Result<()> {
        if let Some(bytes) = tombol.ke_bytes() {
            self.kirim_bytes(&bytes)?;
        }

        Ok(())
    }

    pub fn ubah_ukuran(&self, kolom: u16, baris: u16) -> Result<()> {
        let Some(master) = &self.master else {
            return Ok(());
        };

        master
            .resize(PtySize {
                rows: baris.max(1),
                cols: kolom.max(1),
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Gagal mengubah ukuran ConPTY")
    }

    pub fn try_ambil(&self) -> Option<PesanTerminal> {
        self.penerima_peristiwa.try_recv().ok()
    }
}

impl Drop for SesiTerminal {
    fn drop(&mut self) {
        if self.kode_keluar.is_none() {
            let _ = self.proses.kill();
        }
    }
}
