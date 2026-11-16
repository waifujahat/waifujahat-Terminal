/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Window state
 * Purpose: Owns GPUI window state and managed terminal sessions
 */

mod interaksi;
mod interaksi_mouse;
mod manajemen_sesi;
mod pemantauan;
mod pengukuran;
mod render;
mod render_grid;
mod ruang_sesi;
mod tab_sesi;

use std::time::Duration;

use gpui::{
    Context, FocusHandle, KeyDownEvent, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Subscription,
    Task, Window,
};

use waifujahat_terminal::terminal::ManajerTab;

use ruang_sesi::RuangSesi;

pub(crate) struct JendelaTerminal {
    fokus: FocusHandle,
    manajer: ManajerTab<RuangSesi>,
    lebar_karakter: f32,
    _pengamat_ukuran: Subscription,
    _tugas_pantau: Task<()>,
}

impl JendelaTerminal {
    pub(crate) fn baru(jendela: &mut Window, konteks: &mut Context<Self>) -> Self {
        let fokus = konteks.focus_handle().tab_stop(true);

        jendela.focus(&fokus, konteks);

        let mut manajer = ManajerTab::baru();

        manajer.tambah_tab(RuangSesi::baru());

        let pengamat_ukuran =
            konteks.observe_window_bounds(jendela, |terminal, jendela, konteks| {
                terminal.sesuaikan_ukuran(jendela, konteks);
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

        let mut terminal = Self {
            fokus,
            manajer,
            lebar_karakter: 1.0,
            _pengamat_ukuran: pengamat_ukuran,
            _tugas_pantau: tugas_pantau,
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

    pub(super) fn sesuaikan_ukuran(&mut self, jendela: &mut Window, konteks: &mut Context<Self>) {
        self.lebar_karakter = RuangSesi::lebar_karakter_grid(jendela);

        konteks.notify();
    }

    pub(super) fn proses_tombol(
        &mut self,
        kejadian: &KeyDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.tangani_tombol_sesi(kejadian, jendela, konteks) {
            konteks.stop_propagation();
            return;
        }

        self.ruang_mut().proses_tombol(kejadian, jendela, konteks);
    }

    pub(super) fn mouse_ditekan(
        &mut self,
        kejadian: &MouseDownEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        jendela.focus(&self.fokus, konteks);

        self.ruang_mut().mouse_ditekan(kejadian, jendela, konteks);
    }

    pub(super) fn mouse_digerakkan(
        &mut self,
        kejadian: &MouseMoveEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        self.ruang_mut()
            .mouse_digerakkan(kejadian, jendela, konteks);
    }

    pub(super) fn tangani_lepas_mouse_tui(
        &mut self,
        kejadian: &MouseUpEvent,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        self.ruang_mut()
            .tangani_lepas_mouse_tui(kejadian, jendela, konteks);
    }
}
