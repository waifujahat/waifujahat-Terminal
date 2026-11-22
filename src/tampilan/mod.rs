/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Window state
 * Purpose: Owns GPUI window state and managed terminal sessions
 */

mod divider_panel;
mod edit_nama_tab;
mod input_win32;
mod interaksi;
mod interaksi_mouse;
mod keyboard_kitty;
#[cfg(windows)]
mod keyboard_win32;
mod manajemen_sesi;
mod menu_profil;
mod menu_split;
mod pemantauan;
mod pencarian_terminal;
mod pengukuran;
mod render;
mod render_grid;
mod render_panel;
mod ruang_sesi;
mod tab_sesi;
mod workspace;

use std::{cell::RefCell, sync::Arc, time::Duration};

use gpui::{
    Context, FocusHandle, KeyDownEvent, KeyUpEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Subscription, Task, Window,
};

use waifujahat_terminal::terminal::{
    IdSesi, KepemilikanTombol, ManajerTab, PemantauProsesShell, RekamanTombolWin32,
    StatusKepemilikanTombol,
};

use divider_panel::TarikanPanel;
use ruang_sesi::RuangSesi;

use input_win32::identitas_tombol_win32;

#[cfg(windows)]
use keyboard_win32::{PengamatKeyboardWin32, kode_virtual_keyboard};

use edit_nama_tab::EditNamaTab;

use waifujahat_terminal::konfigurasi::{PengaturanTerminal, PenyimpanWorkspace};

use workspace::AutosaveWorkspace;

pub(crate) struct PengaturanAplikasi(pub(crate) PengaturanTerminal);

impl gpui::Global for PengaturanAplikasi {}

pub(crate) struct WorkspaceAplikasi(pub(crate) RefCell<Option<PenyimpanWorkspace>>);

impl gpui::Global for WorkspaceAplikasi {}

pub(crate) struct JendelaTerminal {
    fokus: FocusHandle,
    manajer: ManajerTab<RuangSesi>,
    kepemilikan_tombol: KepemilikanTombol<IdSesi>,
    lebar_karakter: f32,
    _pengamat_ukuran: Subscription,
    _pengamat_aktivasi: Subscription,
    _tugas_pantau: Task<()>,
    menu_split: Option<gpui::Pixels>,
    menu_profil: Option<menu_profil::MenuProfil>,
    tarikan_panel: Option<TarikanPanel>,
    #[cfg(windows)]
    pengamat_keyboard_win32: Option<PengamatKeyboardWin32>,
    pemantau_shell: Option<PemantauProsesShell>,
    edit_nama_tab: Option<EditNamaTab>,
    pengaturan: PengaturanTerminal,
    penyimpan_workspace: Option<Arc<AutosaveWorkspace>>,
    _tugas_workspace: Option<Task<()>>,
}

impl JendelaTerminal {
    pub(crate) fn baru(jendela: &mut Window, konteks: &mut Context<Self>) -> Self {
        let fokus = konteks.focus_handle().tab_stop(true);

        jendela.focus(&fokus, konteks);

        let pengaturan = konteks
            .try_global::<PengaturanAplikasi>()
            .map(|global| global.0.clone());

        let penyimpan = konteks
            .try_global::<WorkspaceAplikasi>()
            .and_then(|global| global.0.borrow_mut().take());

        let (mut manajer, aman_disimpan) = Self::siapkan_manajer(
            pengaturan.as_ref(),
            penyimpan.as_ref().and_then(PenyimpanWorkspace::terakhir),
        );

        let penyimpan_workspace = if aman_disimpan {
            penyimpan.map(AutosaveWorkspace::baru).map(Arc::new)
        } else {
            None
        };

        #[cfg(windows)]
        let pengamat_keyboard_win32 = match PengamatKeyboardWin32::pasang(jendela) {
            Ok(pengamat) => Some(pengamat),

            Err(kesalahan) => {
                if let Some(ruang) = manajer.aktif_mut()
                    && ruang.status.is_empty()
                {
                    ruang.status = format!("Pengamat keyboard Win32 tidak tersedia: {kesalahan}");
                }

                None
            }
        };

        let pengamat_ukuran =
            konteks.observe_window_bounds(jendela, |terminal, jendela, konteks| {
                terminal.sesuaikan_ukuran(jendela, konteks);
            });

        let pengamat_aktivasi =
            konteks.observe_window_activation(jendela, |terminal, jendela, konteks| {
                if !jendela.is_window_active() {
                    terminal.batalkan_semua_tombol();
                    konteks.notify();
                }
            });

        let tugas_pantau = konteks.spawn(async move |terminal, konteks| {
            loop {
                konteks
                    .background_executor()
                    .timer(Duration::from_millis(40))
                    .await;

                match terminal.update(konteks, |terminal, konteks| {
                    terminal.periksa_keluaran(konteks)
                }) {
                    Ok(true) => {}
                    Ok(false) | Err(_) => break,
                }
            }
        });

        let tugas_workspace = penyimpan_workspace.as_ref().map(|penyimpan| {
            let penyimpan = Arc::clone(penyimpan);

            konteks.spawn(async move |terminal, konteks| {
                loop {
                    konteks
                        .background_executor()
                        .timer(Duration::from_secs(2))
                        .await;

                    let snapshot = match terminal.update(konteks, |terminal, _| {
                        Self::snapshot_workspace(&terminal.manajer)
                    }) {
                        Ok(snapshot) => snapshot,

                        Err(_) => break,
                    };

                    let penyimpan = Arc::clone(&penyimpan);

                    konteks
                        .background_executor()
                        .spawn(async move {
                            penyimpan.simpan(snapshot, false);
                        })
                        .await;
                }
            })
        });

        if penyimpan_workspace.is_some() {
            let terminal = konteks.weak_entity();

            jendela.on_window_should_close(konteks, move |_, aplikasi| {
                let _ = terminal.update(aplikasi, |terminal, _| {
                    terminal.simpan_workspace_akhir();
                });

                true
            });
        }

        let pemantau_shell = match PemantauProsesShell::baru() {
            Ok(pemantau) => Some(pemantau),

            Err(kesalahan) => {
                if std::env::var_os("WAIFUJAHAT_DEBUG_SHELL").is_some() {
                    eprintln!("Gagal membuka pemantau shell : {Error}", Error = kesalahan);
                }

                None
            }
        };

        let mut terminal = Self {
            fokus,
            manajer,
            kepemilikan_tombol: KepemilikanTombol::baru(),
            lebar_karakter: 1.0,
            _pengamat_ukuran: pengamat_ukuran,
            _pengamat_aktivasi: pengamat_aktivasi,
            _tugas_pantau: tugas_pantau,
            menu_split: None,
            menu_profil: None,
            tarikan_panel: None,
            #[cfg(windows)]
            pengamat_keyboard_win32,
            pemantau_shell,
            edit_nama_tab: None,
            pengaturan: pengaturan.unwrap_or_default(),
            penyimpan_workspace,
            _tugas_workspace: tugas_workspace,
        };

        terminal.sesuaikan_ukuran(jendela, konteks);

        terminal
    }

    fn ruang(&self) -> &RuangSesi {
        self.manajer.aktif().expect("Sesi aktif tidak tersedia")
    }

    fn ruang_mut(&mut self) -> &mut RuangSesi {
        self.manajer.aktif_mut().expect("Sesi aktif tidak tersedia")
    }

    fn kode_virtual_win32(nama: &str) -> Option<u16> {
        #[cfg(windows)]
        {
            kode_virtual_keyboard(nama)
        }

        #[cfg(not(windows))]
        {
            let _ = nama;

            None
        }
    }

    fn ambil_rekaman_win32(
        &self,
        kode: Option<u16>,
        ditekan: bool,
        diulang: Option<bool>,
    ) -> Option<RekamanTombolWin32> {
        #[cfg(windows)]
        {
            self.pengamat_keyboard_win32
                .as_ref()?
                .ambil(kode?, ditekan, diulang)
        }

        #[cfg(not(windows))]
        {
            let _ = (kode, ditekan, diulang);

            None
        }
    }

    fn identitas_kepemilikan(
        &self,
        nama: &str,
        kode: Option<u16>,
        rekaman: Option<RekamanTombolWin32>,
    ) -> String {
        let identitas = match rekaman {
            Some(rekaman) => Some(identitas_tombol_win32(rekaman)),

            None => kode.and_then(|kode| {
                self.kepemilikan_tombol
                    .cari_awalan_unik(&format!("win32:{kode}:"))
            }),
        };

        if let Some(identitas) = identitas
            && self.kepemilikan_tombol.ambil(&identitas).is_some()
        {
            return identitas;
        }

        nama.to_owned()
    }

    fn batalkan_semua_tombol(&mut self) {
        for (_, ruang) in self.manajer.iter_sesi_mut() {
            ruang.lepas_semua_tombol_kitty();
            ruang.lepas_semua_tombol_win32();
        }

        self.kepemilikan_tombol.batalkan_semua();
    }

    pub(super) fn proses_tombol_lepas(
        &mut self,
        kejadian: &KeyUpEvent,
        _jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        let nama = kejadian.keystroke.key.to_ascii_lowercase();

        let kode = Self::kode_virtual_win32(&kejadian.keystroke.key);

        let rekaman_win32 = self.ambil_rekaman_win32(kode, false, None);

        let identitas = self.identitas_kepemilikan(&nama, kode, rekaman_win32);

        let Some(status) = self.kepemilikan_tombol.lepas(&identitas) else {
            return;
        };

        if let StatusKepemilikanTombol::Dimiliki(id) = status
            && let Some(ruang) = self.manajer.ambil_mut(id)
            && !ruang.lepas_tombol_win32(&identitas, rekaman_win32)
            && identitas == nama
        {
            ruang.lepas_tombol_kitty(&kejadian.keystroke);
        }

        konteks.stop_propagation();
        konteks.notify();
    }

    pub(super) fn sesuaikan_ukuran(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        self.tarikan_panel = None;

        self.lebar_karakter = self.ruang().lebar_karakter_grid(jendela);

        konteks.notify();
    }

    pub(super) fn mouse_ditekan_panel(
        &mut self,
        id: IdSesi,
        kejadian: &MouseDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.tarikan_panel.is_some() {
            return;
        }

        if self.manajer.id_panel_aktif() != Some(id) {
            if self.ruang().mouse_tui_ditekan.is_some() {
                return;
            }

            self.ruang_mut().keadaan.seleksi.selesai();
            self.ruang_mut().arah_gulir_seleksi = 0;

            if !self.manajer.pilih_panel(id) {
                return;
            }

            konteks.notify();
        }

        self.mouse_ditekan(kejadian, jendela, konteks);
    }

    pub(super) fn proses_tombol(
        &mut self,
        kejadian: &KeyDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        let nama = kejadian.keystroke.key.to_ascii_lowercase();

        let kode = Self::kode_virtual_win32(&kejadian.keystroke.key);

        let rekaman_win32 = self.ambil_rekaman_win32(kode, true, Some(kejadian.is_held));

        let identitas = self.identitas_kepemilikan(&nama, kode, rekaman_win32);

        if kejadian.is_held {
            if let Some(status) = self.kepemilikan_tombol.ambil(&identitas) {
                if status == StatusKepemilikanTombol::EditorNamaTab {
                    self.tangani_tombol_edit_nama_tab(kejadian, konteks);
                } else if let StatusKepemilikanTombol::Dimiliki(id) = status
                    && let Some(ruang) = self.manajer.ambil_mut(id)
                    && !ruang.ulangi_tombol_win32(&identitas, rekaman_win32)
                    && identitas == nama
                {
                    ruang.ulangi_tombol_kitty(&kejadian.keystroke);
                }

                konteks.stop_propagation();
                konteks.notify();

                return;
            }
        } else if let Some(StatusKepemilikanTombol::Dimiliki(id)) =
            self.kepemilikan_tombol.lepas(&identitas)
            && let Some(ruang) = self.manajer.ambil_mut(id)
        {
            if identitas == nama {
                ruang.tombol_kitty_ditekan.remove(&nama);
            } else {
                ruang.lepas_tombol_win32(&identitas, None);
            }
        }

        if self.tarikan_panel.is_some() {
            if kejadian.keystroke.key.eq_ignore_ascii_case("escape") {
                self.akhiri_tarikan_panel(konteks);
            }

            konteks.stop_propagation();

            return;
        }

        if self.menu_split.is_some() || self.menu_profil.is_some() {
            if kejadian.keystroke.key.eq_ignore_ascii_case("escape") {
                self.menu_split = None;
                self.menu_profil = None;

                konteks.notify();
            }

            konteks.stop_propagation();

            return;
        }

        if self.tangani_tombol_edit_nama_tab(kejadian, konteks) {
            self.kepemilikan_tombol.tandai_editor_nama_tab(&identitas);

            konteks.stop_propagation();

            return;
        }

        if self.tangani_tombol_sesi(kejadian, jendela, konteks) {
            konteks.stop_propagation();

            return;
        }

        if self.tangani_tombol_pencarian(kejadian, konteks) {
            konteks.stop_propagation();

            return;
        }

        self.ruang_mut()
            .proses_tombol(kejadian, rekaman_win32, jendela, konteks);

        if let Some(id) = self.manajer.id_panel_aktif() {
            if let Some(rekaman) = rekaman_win32
                && self
                    .ruang()
                    .tombol_win32_ditekan
                    .contains_key(&identitas_tombol_win32(rekaman))
            {
                self.kepemilikan_tombol
                    .catat(&identitas_tombol_win32(rekaman), id);
            } else if self.ruang().tombol_kitty_ditekan.contains_key(&nama) {
                self.kepemilikan_tombol.catat(&nama, id);
            }
        }
    }

    pub(super) fn mouse_ditekan(
        &mut self,
        kejadian: &MouseDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.tarikan_panel.is_some() {
            return;
        }

        jendela.focus(&self.fokus, konteks);

        self.ruang_mut().mouse_ditekan(kejadian, jendela, konteks);
    }

    pub(super) fn mouse_digerakkan(
        &mut self,
        kejadian: &MouseMoveEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.tarikan_panel.is_some() {
            return;
        }

        self.ruang_mut()
            .mouse_digerakkan(kejadian, jendela, konteks);
    }

    pub(super) fn tangani_lepas_mouse_tui(
        &mut self,
        kejadian: &MouseUpEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.tarikan_panel.is_some() {
            return;
        }

        self.ruang_mut()
            .tangani_lepas_mouse_tui(kejadian, jendela, konteks);
    }
}
