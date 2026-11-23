/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Session profile menu
 * Purpose : Provides local shell and SSH connection selection
 */

use gpui::{Context, CursorStyle, MouseButton, Pixels, Window, div, prelude::*, px, rgb};

use waifujahat_terminal::{
    konfigurasi::PenyimpananSsh,
    terminal::{ClientSsh, JenisSesiTerminal, ProfilShell, ProfilSsh},
};

use super::JendelaTerminal;

use crate::tema::{WARNA_AKSEN, WARNA_TEKS};

const WARNA_MENU: u32 = 0x202536;
const WARNA_HOVER: u32 = 0x493663;
const WARNA_BORDER: u32 = 0x393f52;
const WARNA_SEKUNDER: u32 = 0x8992ab;

pub(super) struct MenuProfil {
    posisi: Pixels,
    lokal: Vec<ProfilShell>,
    ssh: Vec<ProfilSsh>,
    ssh_tersedia: bool,
    kesalahan: Option<String>,
}

impl MenuProfil {
    fn baru(posisi: Pixels) -> Self {
        let lokal = ProfilShell::deteksi_tersedia()
            .into_iter()
            .map(|(profil, _)| profil)
            .collect();

        let ssh_tersedia = ClientSsh::deteksi().is_some();

        let (ssh, kesalahan) =
            match PenyimpananSsh::bawaan().and_then(|penyimpanan| penyimpanan.muat()) {
                Ok(Some(daftar)) => (daftar.profil, None),

                Ok(None) => (Vec::new(), None),

                Err(kesalahan) => (
                    Vec::new(),
                    Some(format!("Gagal memuat profil SSH : {kesalahan:#}")),
                ),
            };

        Self {
            posisi,
            lokal,
            ssh,
            ssh_tersedia,
            kesalahan,
        }
    }
}

impl JendelaTerminal {
    pub(super) fn ubah_menu_profil(&mut self, posisi: Pixels, konteks: &mut Context<Self>) {
        if self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        self.ruang_mut().keadaan.seleksi.selesai();
        self.ruang_mut().arah_gulir_seleksi = 0;

        self.menu_split = None;

        self.menu_profil = if self.menu_profil.is_some() {
            None
        } else {
            Some(MenuProfil::baru(posisi))
        };

        konteks.notify();
    }

    pub(super) fn buat_menu_profil(
        &self,
        menu: &MenuProfil,
        jendela: &Window,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        let ukuran = jendela.viewport_size();

        let lebar_jendela = f32::from(ukuran.width);
        let tinggi_jendela = f32::from(ukuran.height);

        let lebar_menu = (lebar_jendela - 16.0).clamp(1.0, 340.0);

        let posisi_x =
            f32::from(menu.posisi).clamp(0.0, (lebar_jendela - lebar_menu - 8.0).max(0.0));

        let jumlah = menu.lokal.len() + menu.ssh.len();

        let tinggi_menu = (100.0 + jumlah as f32 * 44.0).min((tinggi_jendela - 42.0).max(1.0));

        let mut daftar = div()
            .id("menu-profil")
            .absolute()
            .left(px(posisi_x))
            .top(px(34.0))
            .w(px(lebar_menu))
            .h(px(tinggi_menu))
            .flex()
            .flex_col()
            .overflow_y_scroll()
            .p_1()
            .gap_1()
            .bg(rgb(WARNA_MENU))
            .border_1()
            .border_color(rgb(WARNA_BORDER))
            .rounded_md()
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|_, _, _, konteks| {
                    konteks.stop_propagation();
                }),
            )
            .child(Self::judul_menu("LOCAL SHELLS"));

        for profil in menu.lokal.iter().copied() {
            daftar = daftar.child(self.buat_pilihan_profil(
                JenisSesiTerminal::Lokal(profil),
                true,
                konteks,
            ));
        }

        daftar = daftar
            .child(div().h(px(1.0)).flex_shrink_0().bg(rgb(WARNA_BORDER)))
            .child(Self::judul_menu("SSH CONNECTIONS"));

        if let Some(kesalahan) = &menu.kesalahan {
            daftar = daftar.child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(WARNA_SEKUNDER))
                    .child(kesalahan.clone()),
            );
        } else if menu.ssh.is_empty() {
            daftar = daftar.child(
                div()
                    .px_2()
                    .py_1()
                    .text_xs()
                    .text_color(rgb(WARNA_SEKUNDER))
                    .child("No saved SSH connections"),
            );
        }

        for profil in &menu.ssh {
            daftar = daftar.child(self.buat_pilihan_profil(
                JenisSesiTerminal::Ssh(profil.clone()),
                menu.ssh_tersedia,
                konteks,
            ));
        }

        div()
            .id("lapisan-menu-profil")
            .absolute()
            .inset_0()
            .occlude()
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, konteks| {
                    terminal.menu_profil = None;

                    konteks.stop_propagation();
                    konteks.notify();
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(|_, _, _, konteks| {
                    konteks.stop_propagation();
                }),
            )
            .child(daftar)
    }

    fn judul_menu(nama: &'static str) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .flex_shrink_0()
            .text_xs()
            .text_color(rgb(WARNA_SEKUNDER))
            .child(nama)
    }

    fn buat_pilihan_profil(
        &self,
        jenis: JenisSesiTerminal,
        tersedia: bool,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        let nama = jenis.nama().to_owned();

        let (id, keterangan) = match &jenis {
            JenisSesiTerminal::Lokal(profil) => (
                format!("pilihan-shell-{}", profil.nama()),
                "Local terminal".to_owned(),
            ),

            JenisSesiTerminal::Ssh(profil) => (
                format!("pilihan-ssh-{}", profil.id),
                if tersedia {
                    format!("{}@{}:{}", profil.pengguna, profil.host, profil.port)
                } else {
                    "OpenSSH Client unavailable".to_owned()
                },
            ),
        };

        let mut pilihan = div()
            .id(id)
            .w_full()
            .h(px(40.0))
            .flex_shrink_0()
            .px_2()
            .flex()
            .flex_col()
            .justify_center()
            .overflow_hidden()
            .rounded_sm()
            .text_color(rgb(if tersedia { WARNA_TEKS } else { WARNA_SEKUNDER }))
            .child(div().whitespace_nowrap().child(nama))
            .child(
                div()
                    .text_xs()
                    .whitespace_nowrap()
                    .text_color(rgb(WARNA_AKSEN))
                    .child(keterangan),
            );

        if tersedia {
            pilihan = pilihan
                .cursor(CursorStyle::PointingHand)
                .hover(|gaya| gaya.bg(rgb(WARNA_HOVER)))
                .on_mouse_down(
                    MouseButton::Left,
                    konteks.listener(move |terminal, _, jendela, konteks| {
                        terminal.menu_profil = None;

                        terminal.tambah_sesi_dengan_jenis(jenis.clone(), jendela, konteks);

                        konteks.stop_propagation();
                        konteks.notify();
                    }),
                );
        }

        pilihan
    }
}
