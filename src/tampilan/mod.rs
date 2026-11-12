/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Window state
 * Purpose: Owns GPUI window state and application initialization.
 */

mod interaksi;
mod pemantauan;
mod pengukuran;
mod render;

use std::time::Duration;

use gpui::{Context, FocusHandle, ScrollHandle, Subscription, Task, Window, prelude::*};
use waifujahat_terminal::terminal::{
    LayarTerminal, MasukanTerminal, MotorTerminal, RiwayatPerintah, SeleksiTerminal, SesiTerminal,
};

pub(crate) struct JendelaTerminal {
    fokus: FocusHandle,
    gulir: ScrollHandle,
    masukan: MasukanTerminal,
    layar: LayarTerminal,
    motor: MotorTerminal,
    sesi: Option<SesiTerminal>,
    status: String,
    riwayat: RiwayatPerintah,
    seleksi: SeleksiTerminal,
    ukuran_pty: (u16, u16),
    _pengamat_ukuran: Subscription,
    _tugas_pantau: Task<()>,
}

impl JendelaTerminal {
    pub(crate) fn baru(jendela: &mut Window, konteks: &mut Context<Self>) -> Self {
        let fokus = konteks.focus_handle().tab_stop(true);
        jendela.focus(&fokus, konteks);

        let (sesi, status) = match SesiTerminal::baru() {
            Ok(sesi) => (Some(sesi), String::new()),
            Err(kesalahan) => (None, format!("Gagal membuka PowerShell: {kesalahan}")),
        };

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
            gulir: ScrollHandle::new(),
            masukan: MasukanTerminal::baru(),
            layar: LayarTerminal::baru(),
            motor: MotorTerminal::baru(80, 24),
            sesi,
            status,
            riwayat: RiwayatPerintah::baru(),
            seleksi: SeleksiTerminal::baru(),
            ukuran_pty: (80, 24),
            _pengamat_ukuran: pengamat_ukuran,
            _tugas_pantau: tugas_pantau,
        };

        terminal.sesuaikan_ukuran(jendela, konteks);
        terminal
    }
}
