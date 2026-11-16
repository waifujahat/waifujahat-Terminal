/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Command lifecycle
 * Purpose: Manages command state, confirmation and execution duration
 */

use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusPerintah {
    Siap,
    MenungguKonfirmasi,
    Berjalan {
        indeks_baris_awal: usize,
        mulai: Instant,
    },
    SesiBerakhir,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AksiPengajuan {
    Konfirmasi,
    Jalankan,
    Abaikan,
}

pub struct SiklusPerintah {
    status: StatusPerintah,
    konfirmasi_aktif: bool,
    durasi_terakhir: Option<Duration>,
    kode_keluar_terakhir: Option<i32>,
}

impl SiklusPerintah {
    pub fn baru() -> Self {
        Self {
            status: StatusPerintah::Siap,
            konfirmasi_aktif: true,
            durasi_terakhir: None,
            kode_keluar_terakhir: None,
        }
    }

    pub fn kode_keluar_terakhir(&self) -> Option<i32> {
        self.kode_keluar_terakhir
    }

    pub fn status(&self) -> StatusPerintah {
        self.status
    }

    pub fn siap(&self) -> bool {
        self.status == StatusPerintah::Siap
    }

    pub fn berjalan(&self) -> bool {
        matches!(self.status, StatusPerintah::Berjalan { .. })
    }

    pub fn menunggu_konfirmasi(&self) -> bool {
        self.status == StatusPerintah::MenungguKonfirmasi
    }

    pub fn konfirmasi_aktif(&self) -> bool {
        self.konfirmasi_aktif
    }

    pub fn durasi_terakhir(&self) -> Option<Duration> {
        self.durasi_terakhir
    }

    pub fn indeks_baris_awal(&self) -> Option<usize> {
        match self.status {
            StatusPerintah::Berjalan {
                indeks_baris_awal, ..
            } => Some(indeks_baris_awal),

            _ => None,
        }
    }

    pub fn ubah_konfirmasi(&mut self) -> bool {
        if self.berjalan() || self.status == StatusPerintah::SesiBerakhir {
            return false;
        }

        self.konfirmasi_aktif = !self.konfirmasi_aktif;
        self.status = StatusPerintah::Siap;

        true
    }

    pub fn ajukan(&mut self) -> AksiPengajuan {
        if !self.siap() {
            return AksiPengajuan::Abaikan;
        }

        if self.konfirmasi_aktif {
            self.status = StatusPerintah::MenungguKonfirmasi;

            AksiPengajuan::Konfirmasi
        } else {
            AksiPengajuan::Jalankan
        }
    }

    pub fn selesaikan_konfirmasi(&mut self) -> bool {
        if !self.menunggu_konfirmasi() {
            return false;
        }

        self.status = StatusPerintah::Siap;

        true
    }

    pub fn mulai(&mut self, indeks_baris_awal: usize, waktu: Instant) -> bool {
        if !self.siap() {
            return false;
        }

        self.status = StatusPerintah::Berjalan {
            indeks_baris_awal,
            mulai: waktu,
        };

        true
    }

    pub fn selesai(&mut self, waktu: Instant) -> bool {
        self.selesai_dengan_kode(waktu, None)
    }

    pub fn selesai_dengan_kode(&mut self, waktu: Instant, kode_keluar: Option<i32>) -> bool {
        let StatusPerintah::Berjalan { mulai, .. } = self.status else {
            return false;
        };

        self.durasi_terakhir = Some(waktu.saturating_duration_since(mulai));

        self.kode_keluar_terakhir = kode_keluar;
        self.status = StatusPerintah::Siap;

        true
    }

    pub fn akhiri_sesi(&mut self) {
        self.status = StatusPerintah::SesiBerakhir;
    }
}
