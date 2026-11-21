/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Adaptive text contrast
 * Purpose : Maintains readable terminal foreground colors
 */

use std::collections::HashMap;

pub const WARNA_LAPISAN: u32 = 0x090d18;

const RASIO_MINIMUM: f64 = 4.5;
const PENGUAT_KONTRAS: u32 = 16;

#[derive(Clone, Copy)]
pub struct WarnaKontras {
    pub warna_teks: u32,
    pub warna_latar: Option<u32>,
}

pub struct KontrasTeks {
    luminansi_minimum: f64,
    luminansi_maksimum: f64,
}

impl KontrasTeks {
    pub fn baru(alfa: u8) -> Self {
        let minimum = campur(0x000000, WARNA_LAPISAN, u32::from(alfa), 0);

        let maksimum = campur(0xffffff, WARNA_LAPISAN, u32::from(alfa), 254);

        Self {
            luminansi_minimum: luminansi(minimum),
            luminansi_maksimum: luminansi(maksimum),
        }
    }

    pub fn sesuaikan(&self, warna_teks: u32, warna_latar: Option<u32>) -> WarnaKontras {
        if let Some(latar) = warna_latar {
            let nilai = luminansi(latar);

            return WarnaKontras {
                warna_teks: warna_terdekat(warna_teks, nilai, nilai)
                    .expect("Warna kontras tidak tersedia"),
                warna_latar: Some(latar),
            };
        }

        if let Some(teks) =
            warna_terdekat(warna_teks, self.luminansi_minimum, self.luminansi_maksimum)
        {
            return WarnaKontras {
                warna_teks: teks,
                warna_latar: None,
            };
        }

        let nilai = luminansi(WARNA_LAPISAN);

        WarnaKontras {
            warna_teks: warna_terdekat(warna_teks, nilai, nilai)
                .expect("Warna kontras cadangan tidak tersedia"),
            warna_latar: Some(WARNA_LAPISAN),
        }
    }
}

fn luminansi(warna: u32) -> f64 {
    let kanal = |geser: u32| {
        let nilai = f64::from((warna >> geser) & 0xff) / 255.0;

        if nilai <= 0.04045 {
            nilai / 12.92
        } else {
            ((nilai + 0.055) / 1.055).powf(2.4)
        }
    };

    0.2126 * kanal(16) + 0.7152 * kanal(8) + 0.0722 * kanal(0)
}

fn campur(asal: u32, tujuan: u32, faktor: u32, pembulatan: u32) -> u32 {
    let kanal = |geser: u32| {
        let pertama = (asal >> geser) & 0xff;
        let kedua = (tujuan >> geser) & 0xff;

        (pertama * (255 - faktor) + kedua * faktor + pembulatan) / 255
    };

    (kanal(16) << 16) | (kanal(8) << 8) | kanal(0)
}

fn memenuhi_kontras(warna: u32, minimum: f64, maksimum: f64) -> bool {
    let nilai = luminansi(warna);

    if nilai > maksimum {
        (nilai + 0.05) / (maksimum + 0.05) >= RASIO_MINIMUM
    } else if nilai < minimum {
        (minimum + 0.05) / (nilai + 0.05) >= RASIO_MINIMUM
    } else {
        false
    }
}

fn warna_terdekat(warna: u32, minimum: f64, maksimum: f64) -> Option<u32> {
    let memenuhi = |warna| memenuhi_kontras(warna, minimum, maksimum);

    if memenuhi(warna) {
        return Some(warna);
    }

    [0x000000, 0xffffff]
        .into_iter()
        .filter_map(|tujuan| {
            if !memenuhi(tujuan) {
                return None;
            }

            let mut bawah = 0;
            let mut atas = 255;

            while bawah < atas {
                let tengah = (bawah + atas) / 2;

                let kandidat = campur(warna, tujuan, tengah, 127);

                if memenuhi(kandidat) {
                    atas = tengah;
                } else {
                    bawah = tengah + 1;
                }
            }

            let faktor = (bawah + PENGUAT_KONTRAS).min(255);

            Some((bawah, campur(warna, tujuan, faktor, 127)))
        })
        .min_by_key(|(faktor, _)| *faktor)
        .map(|(_, warna)| warna)
}

pub struct WarnaTampilan {
    pub warna_teks: Option<u32>,
    pub warna_latar: Option<u32>,
}

pub struct PaletKontras {
    kontras: KontrasTeks,
    cache: HashMap<(u32, Option<u32>), WarnaKontras>,
}

impl PaletKontras {
    pub fn baru(alfa: u8) -> Self {
        Self {
            kontras: KontrasTeks::baru(alfa),
            cache: HashMap::new(),
        }
    }

    pub fn gaya(
        &mut self,
        teks: &str,
        warna_teks: u32,
        warna_latar: Option<u32>,
        tersembunyi: bool,
    ) -> WarnaTampilan {
        if tersembunyi {
            return WarnaTampilan {
                warna_teks: None,
                warna_latar,
            };
        }

        if teks.trim().is_empty() {
            return WarnaTampilan {
                warna_teks: Some(warna_teks),
                warna_latar,
            };
        }

        let hasil = self
            .cache
            .entry((warna_teks, warna_latar))
            .or_insert_with(|| self.kontras.sesuaikan(warna_teks, warna_latar));

        WarnaTampilan {
            warna_teks: Some(hasil.warna_teks),
            warna_latar: hasil.warna_latar,
        }
    }
}
