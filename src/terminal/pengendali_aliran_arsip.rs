/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Archive flow controller
 * Purpose: Coordinates incremental output and nonblocking archival
 */

use std::{
    io,
    sync::mpsc,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, ensure};

use super::{BufferKeluaran, HasilArsipSesi, KeadaanSesi, PenandaShell, PengarsipSesi};

const BATAS_POTONGAN: usize = 64;
const JUMLAH_BATCH: usize = 2;
const CADANGAN_RIWAYAT: usize = 2;

#[derive(Debug, PartialEq, Eq)]
pub enum HasilLangkahAliran {
    TidakAda,
    MenungguArsip,
    ArsipTersimpan(usize),
    OutputDiproses {
        jumlah_byte: usize,
        penanda_shell: Vec<PenandaShell>,
    },
    Dihentikan(String),
}

pub struct PengendaliAliranArsip {
    pengarsip: PengarsipSesi,
    buffer: Option<BufferKeluaran>,
    kapasitas_riwayat: usize,
    kesalahan: Option<String>,
    mode_langsung: bool,
}

impl PengendaliAliranArsip {
    pub fn baru(pengarsip: PengarsipSesi, kapasitas_riwayat: usize) -> Result<Self> {
        ensure!(
            kapasitas_riwayat >= 4,
            "Kapasitas riwayat minimal empat baris"
        );

        Ok(Self {
            pengarsip,
            buffer: None,
            kapasitas_riwayat,
            kesalahan: None,
            mode_langsung: false,
        })
    }

    pub fn ada_pekerjaan_tertunda(&self) -> bool {
        self.ada_buffer() || (!self.mode_langsung && self.pengarsip.ada_batch_tertunda())
    }

    pub fn ada_buffer(&self) -> bool {
        self.buffer.is_some()
    }

    pub fn ambil_buffer_untuk_pemulihan(&mut self) -> Option<BufferKeluaran> {
        self.kesalahan.as_ref()?;

        self.buffer.take()
    }

    pub fn sisa_buffer(&self) -> usize {
        self.buffer.as_ref().map_or(0, BufferKeluaran::sisa_byte)
    }

    pub fn kesalahan(&self) -> Option<&str> {
        self.kesalahan.as_deref()
    }

    pub fn aktifkan_mode_langsung(&mut self) -> Result<()> {
        ensure!(
            self.kesalahan.is_some(),
            "Mode langsung hanya dapat diaktifkan setelah pengarsipan gagal"
        );

        self.mode_langsung = true;

        Ok(())
    }

    pub fn terima_buffer(&mut self, data: Vec<u8>) -> Result<bool> {
        ensure!(
            self.kesalahan.is_none() || self.mode_langsung,
            "Tidak dapat menerima output: pengarsipan telah dihentikan"
        );

        if self.buffer.is_some() {
            return Ok(false);
        }

        self.buffer = Some(BufferKeluaran::baru(data)?);

        Ok(true)
    }

    pub fn langkah(&mut self, keadaan: &mut KeadaanSesi) -> Result<HasilLangkahAliran> {
        if self.mode_langsung {
            return self.langkah_langsung(keadaan);
        }

        if let Some(pesan) = &self.kesalahan {
            return Ok(HasilLangkahAliran::Dihentikan(pesan.clone()));
        }

        let kapasitas_motor = keadaan.motor.batas_riwayat();

        if self.kapasitas_riwayat != kapasitas_motor {
            return Ok(self.hentikan(format!(
                "Kapasitas riwayat tidak sesuai: pengendali={}, motor={}",
                self.kapasitas_riwayat, kapasitas_motor
            )));
        }

        if self.pengarsip.ada_batch_tertunda() {
            return Ok(match self.pengarsip.periksa(&mut keadaan.motor) {
                Some(HasilArsipSesi::Tersimpan(jumlah)) => {
                    HasilLangkahAliran::ArsipTersimpan(jumlah)
                }

                Some(HasilArsipSesi::Dihentikan(pesan)) => self.hentikan(pesan),

                None => HasilLangkahAliran::MenungguArsip,
            });
        }

        let ambang = self.kapasitas_riwayat.saturating_sub(CADANGAN_RIWAYAT);

        if !keadaan.motor.layar_alternatif_aktif() && keadaan.motor.jumlah_baris_riwayat() >= ambang
        {
            let hasil = self
                .pengarsip
                .kirim_dari_motor(&keadaan.motor, JUMLAH_BATCH);

            return Ok(match hasil {
                Ok(true) => HasilLangkahAliran::MenungguArsip,

                Ok(false) => self.hentikan("Riwayat hampir penuh tetapi batch tidak dapat dikirim"),

                Err(kesalahan) => self.hentikan(format!("{kesalahan:#}")),
            });
        }

        if self.buffer.is_none() {
            return Ok(HasilLangkahAliran::TidakAda);
        }

        let jumlah_reservasi = self.sisa_buffer().min(BATAS_POTONGAN);

        if !keadaan
            .motor
            .cadangkan_riwayat_sebelum_output(jumlah_reservasi)
        {
            return Ok(
                self.hentikan("Gagal mencadangkan kapasitas riwayat sebelum memproses output")
            );
        }

        let (potongan, selesai) = {
            let Some(buffer) = self.buffer.as_mut() else {
                return Ok(HasilLangkahAliran::TidakAda);
            };

            let Some(potongan) = buffer.ambil_potongan(BATAS_POTONGAN)? else {
                self.buffer = None;
                return Ok(HasilLangkahAliran::TidakAda);
            };

            let selesai = buffer.selesai();

            (potongan, selesai)
        };

        let jumlah_byte = potongan.len();
        let penanda_shell = keadaan.terima(&potongan);

        if selesai {
            self.buffer = None;
        }

        Ok(HasilLangkahAliran::OutputDiproses {
            jumlah_byte,
            penanda_shell,
        })
    }

    fn langkah_langsung(&mut self, keadaan: &mut KeadaanSesi) -> Result<HasilLangkahAliran> {
        let (potongan, selesai) = {
            let Some(buffer) = self.buffer.as_mut() else {
                return Ok(HasilLangkahAliran::TidakAda);
            };

            let Some(potongan) = buffer.ambil_potongan(BATAS_POTONGAN)? else {
                self.buffer = None;
                return Ok(HasilLangkahAliran::TidakAda);
            };

            (potongan, buffer.selesai())
        };

        let jumlah_byte = potongan.len();
        let penanda_shell = keadaan.terima(&potongan);

        if selesai {
            self.buffer = None;
        }

        Ok(HasilLangkahAliran::OutputDiproses {
            jumlah_byte,
            penanda_shell,
        })
    }

    fn hentikan(&mut self, pesan: impl Into<String>) -> HasilLangkahAliran {
        let pesan = pesan.into();

        self.kesalahan = Some(pesan.clone());

        HasilLangkahAliran::Dihentikan(pesan)
    }

    pub fn mulai_penyelesaian(self) -> Result<TugasPenyelesaianArsip> {
        self.mulai_penyelesaian_dengan_snapshot(None)
    }

    pub fn mulai_penyelesaian_dengan_snapshot(
        self,
        snapshot: Option<Vec<String>>,
    ) -> Result<TugasPenyelesaianArsip> {
        let mut tugas = TugasPenyelesaianArsip {
            pekerja: None,
            pekerjaan_tertunda: Some((self, snapshot)),
            kesalahan_penjadwalan: None,
            jadwal_coba_lagi: None,
            waktu_mulai: None,
        };

        tugas.coba_mulai();

        Ok(tugas)
    }

    fn jalankan_finalisasi(self, snapshot: Option<Vec<String>>) -> Result<()> {
        match snapshot {
            Some(snapshot) if self.kesalahan.is_none() && !self.mode_langsung => {
                self.pengarsip.selesaikan_dengan_snapshot(snapshot)
            }

            Some(_) => {
                let pesan = self
                    .kesalahan
                    .clone()
                    .unwrap_or_else(|| "mode langsung aktif".to_owned());

                self.pengarsip.selesaikan()?;

                Err(anyhow!("Snapshot akhir dilewati: {pesan}"))
            }

            None => self.pengarsip.selesaikan(),
        }
    }

    pub fn selesaikan(self) -> Result<()> {
        self.pengarsip.selesaikan()
    }
}

type PekerjaanArsip = (PengendaliAliranArsip, Option<Vec<String>>);

pub struct TugasPenyelesaianArsip {
    pekerja: Option<JoinHandle<Result<()>>>,
    pekerjaan_tertunda: Option<PekerjaanArsip>,
    kesalahan_penjadwalan: Option<String>,
    jadwal_coba_lagi: Option<Instant>,
    waktu_mulai: Option<Instant>,
}

impl TugasPenyelesaianArsip {
    pub fn kesalahan_penjadwalan(&self) -> Option<&str> {
        self.kesalahan_penjadwalan.as_deref()
    }

    fn coba_mulai(&mut self) {
        self.coba_mulai_dengan(|penerima| {
            thread::Builder::new()
                .name("waifujahat-archive-finish".into())
                .spawn(move || {
                    let (pengendali, snapshot) = penerima
                        .recv()
                        .context("Pekerjaan finalisasi arsip tidak diterima")?;

                    pengendali.jalankan_finalisasi(snapshot)
                })
        });
    }

    fn coba_mulai_dengan<F>(&mut self, jalankan: F)
    where
        F: FnOnce(mpsc::Receiver<PekerjaanArsip>) -> io::Result<JoinHandle<Result<()>>>,
    {
        let Some(pekerjaan) = self.pekerjaan_tertunda.take() else {
            return;
        };

        let (pengirim, penerima) = mpsc::channel();

        match jalankan(penerima) {
            Ok(pekerja) => match pengirim.send(pekerjaan) {
                Ok(()) => {
                    self.pekerja = Some(pekerja);
                    self.waktu_mulai = Some(Instant::now());
                    self.kesalahan_penjadwalan = None;
                    self.jadwal_coba_lagi = None;
                }

                Err(kesalahan) => {
                    self.pekerjaan_tertunda = Some(kesalahan.0);
                    self.kesalahan_penjadwalan =
                        Some("Kanal penjadwalan arsip terputus".to_owned());
                    self.jadwal_coba_lagi = Some(Instant::now() + Duration::from_secs(1));

                    drop(pekerja);
                }
            },

            Err(kesalahan) => {
                self.pekerjaan_tertunda = Some(pekerjaan);
                self.kesalahan_penjadwalan = Some(format!(
                    "Gagal membuat thread penyelesaian arsip: {kesalahan}"
                ));
                self.jadwal_coba_lagi = Some(Instant::now() + Duration::from_secs(1));
            }
        }
    }

    pub fn peringatan(&self) -> Option<String> {
        if let Some(pesan) = &self.kesalahan_penjadwalan {
            return Some(format!("Pengarsipan menunggu retry: {pesan}"));
        }

        if self.pekerja.is_some()
            && self
                .waktu_mulai
                .is_some_and(|mulai| mulai.elapsed() >= Duration::from_secs(10))
        {
            return Some("Finalisasi arsip belum selesai setelah 10 detik".to_owned());
        }

        None
    }

    pub fn periksa(&mut self) -> Option<Result<()>> {
        if self.pekerja.is_none()
            && self
                .jadwal_coba_lagi
                .is_none_or(|batas| Instant::now() >= batas)
        {
            self.coba_mulai();
        }

        if !self.pekerja.as_ref()?.is_finished() {
            return None;
        }

        let pekerja = self.pekerja.take()?;

        Some(Self::gabungkan(pekerja))
    }

    pub fn tunggu(mut self) -> Result<()> {
        if self.pekerja.is_none() {
            self.coba_mulai();
        }

        let pekerja = self.pekerja.take().ok_or_else(|| {
            anyhow!(
                "Pekerja penyelesaian arsip tidak tersedia: {}",
                self.kesalahan_penjadwalan
                    .as_deref()
                    .unwrap_or("belum dijadwalkan")
            )
        })?;

        Self::gabungkan(pekerja)
    }

    fn gabungkan(pekerja: JoinHandle<Result<()>>) -> Result<()> {
        pekerja
            .join()
            .map_err(|_| anyhow!("Thread penyelesaian arsip mengalami panic"))?
    }
}
