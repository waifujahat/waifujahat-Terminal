/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Mouse reporting
 * Purpose: Encodes mouse events using the SGR protocol
 */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TombolMouse {
    Kiri,
    Tengah,
    Kanan,
}

impl TombolMouse {
    fn kode(self) -> u8 {
        match self {
            Self::Kiri => 0,
            Self::Tengah => 1,
            Self::Kanan => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AksiMouse {
    Tekan(TombolMouse),
    Lepas(TombolMouse),
    Gerak(Option<TombolMouse>),
    RodaAtas,
    RodaBawah,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PengubahMouse {
    pub shift: bool,
    pub alt: bool,
    pub kontrol: bool,
}

pub fn sandi_mouse_sgr(
    aksi: AksiMouse,
    kolom: usize,
    baris: usize,
    pengubah: PengubahMouse,
) -> Vec<u8> {
    let (kode_dasar, penutup) = match aksi {
        AksiMouse::Tekan(tombol) => (tombol.kode(), 'M'),
        AksiMouse::Lepas(tombol) => (tombol.kode(), 'm'),
        AksiMouse::Gerak(Some(tombol)) => (32 + tombol.kode(), 'M'),
        AksiMouse::Gerak(None) => (35, 'M'),
        AksiMouse::RodaAtas => (64, 'M'),
        AksiMouse::RodaBawah => (65, 'M'),
    };

    let mut kode = kode_dasar;

    if pengubah.shift {
        kode += 4;
    }

    if pengubah.alt {
        kode += 8;
    }

    if pengubah.kontrol {
        kode += 16;
    }

    let kolom = kolom.saturating_add(1);
    let baris = baris.saturating_add(1);

    format!("\x1b[<{kode};{kolom};{baris}{penutup}").into_bytes()
}
