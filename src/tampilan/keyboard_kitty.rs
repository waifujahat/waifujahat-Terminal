/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Kitty keyboard
 * Purpose: Routes keyboard events to their owning terminal session
 */

use gpui::{KeyDownEvent, Keystroke};

use waifujahat_terminal::terminal::{
    JenisAksiTombol, PengubahTombol, TombolKhusus, TombolTerminal,
};

use super::{interaksi::petakan_tombol_khusus, ruang_sesi::RuangSesi};

#[derive(Clone, Copy, Debug)]
pub(super) enum IdentitasTombolKitty {
    Unicode(char),
    Khusus(TombolKhusus),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TombolKittyDitekan {
    identitas: IdentitasTombolKitty,
    pengubah: PengubahTombol,
}

impl IdentitasTombolKitty {
    fn dari_keystroke(tombol: &Keystroke) -> Option<Self> {
        let nama = tombol.key.to_ascii_lowercase();

        if let Some(khusus) = petakan_tombol_khusus(&nama) {
            return Some(Self::Khusus(khusus));
        }

        if matches!(nama.as_str(), "escape" | "esc") {
            return Some(Self::Unicode('\x1b'));
        }

        if !tombol.modifiers.control && !tombol.modifiers.alt {
            return None;
        }

        if nama == "space" {
            return Some(Self::Unicode(' '));
        }

        let mut karakter = nama.chars();
        let pertama = karakter.next()?;

        karakter.next().is_none().then_some(Self::Unicode(pertama))
    }
}

fn pengubah_keystroke(tombol: &Keystroke) -> PengubahTombol {
    PengubahTombol {
        shift: tombol.modifiers.shift,
        alt: tombol.modifiers.alt,
        kontrol: tombol.modifiers.control,
    }
}

impl RuangSesi {
    fn kirim_aksi_kitty(
        &mut self,
        identitas: IdentitasTombolKitty,
        pengubah: PengubahTombol,
        aksi: JenisAksiTombol,
    ) -> bool {
        if !self.keadaan.motor.kitty_aktif() {
            return false;
        }

        let data = match identitas {
            IdentitasTombolKitty::Unicode(karakter) => self
                .keadaan
                .motor
                .sandi_unicode_kitty(karakter, pengubah, aksi),

            IdentitasTombolKitty::Khusus(khusus) => self
                .keadaan
                .motor
                .sandi_tombol_dengan_aksi(TombolTerminal::Khusus(khusus, pengubah), aksi),
        };

        let Some(data) = data else {
            return false;
        };

        let Some(sesi) = &self.sesi else {
            return false;
        };

        if let Err(kesalahan) = sesi.kirim_bytes(&data) {
            self.status = format!("Gagal mengirim input Kitty: {kesalahan}");

            return false;
        }

        true
    }

    pub(super) fn kirim_tombol_kitty(&mut self, kejadian: &KeyDownEvent) -> bool {
        if !self.keadaan.motor.kitty_aktif() || kejadian.keystroke.modifiers.platform {
            return false;
        }

        let Some(identitas) = IdentitasTombolKitty::dari_keystroke(&kejadian.keystroke) else {
            return false;
        };

        if kejadian.is_held {
            return true;
        }

        let pengubah = pengubah_keystroke(&kejadian.keystroke);

        if self.kirim_aksi_kitty(identitas, pengubah, JenisAksiTombol::Tekan) {
            self.tombol_kitty_ditekan.insert(
                kejadian.keystroke.key.to_ascii_lowercase(),
                TombolKittyDitekan {
                    identitas,
                    pengubah,
                },
            );
        }

        true
    }

    pub(super) fn ulangi_tombol_kitty(&mut self, tombol: &Keystroke) -> bool {
        let nama = tombol.key.to_ascii_lowercase();

        let Some(ditekan) = self.tombol_kitty_ditekan.get(&nama).copied() else {
            return false;
        };

        let pengubah = pengubah_keystroke(tombol);

        if self.kirim_aksi_kitty(ditekan.identitas, pengubah, JenisAksiTombol::Ulang) {
            self.tombol_kitty_ditekan.insert(
                nama,
                TombolKittyDitekan {
                    pengubah,
                    ..ditekan
                },
            );
        }

        true
    }

    pub(super) fn lepas_tombol_kitty(&mut self, tombol: &Keystroke) -> bool {
        let nama = tombol.key.to_ascii_lowercase();

        let Some(ditekan) = self.tombol_kitty_ditekan.remove(&nama) else {
            return false;
        };

        self.kirim_aksi_kitty(
            ditekan.identitas,
            pengubah_keystroke(tombol),
            JenisAksiTombol::Lepas,
        );

        true
    }

    pub(super) fn lepas_semua_tombol_kitty(&mut self) {
        let daftar = std::mem::take(&mut self.tombol_kitty_ditekan);

        if !self.keadaan.motor.kitty_pelaporan_aksi_aktif() {
            return;
        }

        for tombol in daftar.into_values() {
            self.kirim_aksi_kitty(tombol.identitas, tombol.pengubah, JenisAksiTombol::Lepas);
        }
    }
}
