/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Persistent history archive
 * Purpose: Stores terminal history with bounded disk
 */

use std::{
    collections::VecDeque,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, ErrorKind, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, ensure};

const BATAS_BERKAS_DEFAULT: u64 = 4 * 1024 * 1024;
const JUMLAH_BERKAS_DEFAULT: usize = 4;

#[derive(Clone)]
pub struct ArsipRiwayat {
    direktori: PathBuf,
    batas_berkas: u64,
    jumlah_berkas: usize,
}

impl ArsipRiwayat {
    pub fn baru(direktori: impl Into<PathBuf>) -> Result<Self> {
        Self::dengan_batas(direktori, BATAS_BERKAS_DEFAULT, JUMLAH_BERKAS_DEFAULT)
    }

    fn dengan_batas(
        direktori: impl Into<PathBuf>,
        batas_berkas: u64,
        jumlah_berkas: usize,
    ) -> Result<Self> {
        ensure!(batas_berkas > 0, "Batas arsip harus lebih dari nol");

        ensure!(
            (1..=32).contains(&jumlah_berkas),
            "Jumlah berkas arsip tidak valid"
        );

        let direktori = direktori.into();

        ensure!(
            direktori.is_absolute(),
            "Direktori arsip harus menggunakan jalur absolut"
        );

        fs::create_dir_all(&direktori).context("Gagal membuat direktori arsip riwayat")?;

        Ok(Self {
            direktori,
            batas_berkas,
            jumlah_berkas,
        })
    }

    pub fn direktori(&self) -> &Path {
        &self.direktori
    }

    fn jalur_berkas(&self, indeks: usize) -> PathBuf {
        if indeks == 0 {
            self.direktori.join("scrollback.log")
        } else {
            self.direktori.join(format!("scrollback.{indeks}.log"))
        }
    }

    fn ukuran_baris_tervalidasi(&self, teks: &str) -> Result<u64> {
        ensure!(
            !teks
                .chars()
                .any(|karakter| karakter.is_control() && karakter != '\t'),
            "Baris arsip mengandung karakter kontrol"
        );

        let ukuran = u64::try_from(teks.len())?
            .checked_add(1)
            .context("Ukuran baris arsip tidak valid")?;

        ensure!(
            ukuran <= self.batas_berkas,
            "Baris arsip melebihi batas ukuran berkas"
        );

        Ok(ukuran)
    }

    pub(crate) fn validasi_batch(&self, baris: &[String]) -> Result<()> {
        ensure!(!baris.is_empty(), "Kumpulan baris arsip kosong");

        for teks in baris {
            self.ukuran_baris_tervalidasi(teks)?;
        }

        Ok(())
    }

    pub fn simpan_baris(&self, teks: &str) -> Result<()> {
        let ukuran_baris = self.ukuran_baris_tervalidasi(teks)?;

        let jalur = self.jalur_berkas(0);

        let ukuran_sekarang = match fs::metadata(&jalur) {
            Ok(metadata) => metadata.len(),

            Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => 0,

            Err(kesalahan) => {
                return Err(kesalahan).context("Gagal membaca ukuran arsip");
            }
        };

        if ukuran_sekarang.saturating_add(ukuran_baris) > self.batas_berkas {
            self.rotasi()?;
        }

        let mut berkas = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&jalur)
            .context("Gagal membuka arsip riwayat")?;

        berkas
            .write_all(teks.as_bytes())
            .context("Gagal menulis teks arsip")?;

        berkas
            .write_all(b"\n")
            .context("Gagal mengakhiri baris arsip")?;

        Ok(())
    }

    pub fn baca_baris_terakhir(&self, batas: usize) -> Result<Vec<String>> {
        const BATAS_PEMBACAAN: usize = 10_000;

        ensure!(
            batas <= BATAS_PEMBACAAN,
            "Jumlah baris pembacaan melebihi batas"
        );

        if batas == 0 {
            return Ok(Vec::new());
        }

        let mut hasil = VecDeque::new();

        // Arsip bernomor terbesar merupakan arsip paling lama.
        for indeks in (0..self.jumlah_berkas).rev() {
            let jalur = self.jalur_berkas(indeks);

            let berkas = match File::open(&jalur) {
                Ok(berkas) => berkas,

                Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => {
                    continue;
                }

                Err(kesalahan) => {
                    return Err(kesalahan)
                        .with_context(|| format!("Gagal membuka arsip {}", jalur.display()));
                }
            };

            ensure!(
                berkas.metadata()?.len() <= self.batas_berkas,
                "Ukuran berkas arsip melebihi batas"
            );

            for baris in BufReader::new(berkas).lines() {
                let baris =
                    baris.with_context(|| format!("Gagal membaca arsip {}", jalur.display()))?;

                if hasil.len() == batas {
                    hasil.pop_front();
                }

                hasil.push_back(baris);
            }
        }

        Ok(hasil.into_iter().collect())
    }

    pub fn sinkronkan(&self) -> Result<()> {
        let jalur = self.jalur_berkas(0);

        let berkas = OpenOptions::new()
            .write(true)
            .open(&jalur)
            .context("Gagal membuka arsip untuk sinkronisasi")?;

        berkas
            .sync_all()
            .context("Gagal menyinkronkan arsip ke disk")
    }

    fn rotasi(&self) -> Result<()> {
        self.sinkronkan()?;

        self.hapus_jika_ada(self.jumlah_berkas - 1)?;

        for indeks in (1..self.jumlah_berkas).rev() {
            let sumber = self.jalur_berkas(indeks - 1);
            let tujuan = self.jalur_berkas(indeks);

            match fs::rename(&sumber, &tujuan) {
                Ok(()) => {}

                Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => {}

                Err(kesalahan) => {
                    return Err(kesalahan)
                        .with_context(|| format!("Gagal merotasi arsip {indeks}"));
                }
            }
        }

        Ok(())
    }

    pub fn hapus_semua_berkas(&self) -> Result<()> {
        for indeks in 0..self.jumlah_berkas {
            self.hapus_jika_ada(indeks)?;
        }

        Ok(())
    }

    fn hapus_jika_ada(&self, indeks: usize) -> Result<()> {
        match fs::remove_file(self.jalur_berkas(indeks)) {
            Ok(()) => Ok(()),

            Err(kesalahan) if kesalahan.kind() == ErrorKind::NotFound => Ok(()),

            Err(kesalahan) => Err(kesalahan).context("Gagal menghapus arsip terlama"),
        }
    }
}
