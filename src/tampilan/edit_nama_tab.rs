/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Tab name editor
 * Purpose : Provides interactive tab name editing and text selection
 */

use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    ClipboardItem, Context, CursorStyle, Font, KeyDownEvent, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, Pixels, ShapedLine, TextRun, Window, div, prelude::*, px, rgb,
    rgba,
};

use waifujahat_terminal::terminal::{BATAS_NAMA_TAB, IdTab, MasukanTerminal};

use super::JendelaTerminal;
use crate::tema::{WARNA_AKSEN, WARNA_TEKS};

const WARNA_EDITOR: u32 = 0x202536;
const WARNA_INPUT: u32 = 0x191e30;
const WARNA_SELEKSI: u32 = 0x493663;

pub(super) struct EditNamaTab {
    id: IdTab,
    masukan: MasukanTerminal,
    jangkar_seleksi: Option<usize>,
    menarik_seleksi: bool,
}

impl EditNamaTab {
    fn rentang_seleksi(&self) -> Option<Range<usize>> {
        let jangkar = self.jangkar_seleksi?;
        let posisi = self.masukan.posisi();

        if jangkar == posisi {
            return None;
        }

        Some(jangkar.min(posisi)..jangkar.max(posisi))
    }

    fn pilih_semua(&mut self) {
        self.jangkar_seleksi = Some(0);
        self.masukan.pindah_akhir();
        self.menarik_seleksi = false;
    }

    fn pindah_kursor(&mut self, posisi: usize, seleksi: bool) {
        if seleksi {
            self.jangkar_seleksi.get_or_insert(self.masukan.posisi());
        } else {
            self.jangkar_seleksi = None;
        }

        self.masukan.pindah_ke(posisi);
    }

    fn pindah_kiri(&mut self, seleksi: bool) {
        let posisi = match self.rentang_seleksi() {
            Some(rentang) if !seleksi => rentang.start,

            _ => self.masukan.posisi().saturating_sub(1),
        };

        self.pindah_kursor(posisi, seleksi);
    }

    fn pindah_kanan(&mut self, seleksi: bool) {
        let posisi = match self.rentang_seleksi() {
            Some(rentang) if !seleksi => rentang.end,

            _ => self.masukan.posisi().saturating_add(1),
        };

        self.pindah_kursor(posisi, seleksi);
    }

    fn hapus_seleksi(&mut self) -> bool {
        let rentang = self.rentang_seleksi();

        self.jangkar_seleksi = None;
        self.menarik_seleksi = false;

        let Some(rentang) = rentang else {
            return false;
        };

        self.masukan.hapus_rentang(rentang);

        true
    }

    fn sisipkan(&mut self, teks: &str) {
        let jumlah_terseleksi = self.rentang_seleksi().map_or(0, |rentang| rentang.len());

        let panjang = self.masukan.teks().chars().count();

        let sisa = BATAS_NAMA_TAB.saturating_sub(panjang.saturating_sub(jumlah_terseleksi));

        let teks: String = teks
            .chars()
            .filter(|karakter| !karakter.is_control())
            .take(sisa)
            .collect();

        if teks.is_empty() {
            return;
        }

        self.hapus_seleksi();
        self.masukan.sisipkan(&teks);
    }
}

fn bentuk_teks_nama_tab(teks: &str, jendela: &Window) -> ShapedLine {
    let run = TextRun {
        len: teks.len(),
        font: Font {
            family: "Consolas".into(),
            ..Default::default()
        },
        color: rgb(WARNA_TEKS).into(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };

    jendela
        .text_system()
        .shape_line(teks.to_owned().into(), px(14.0), &[run], None)
}

fn posisi_dari_mouse(
    teks: &str,
    posisi_mouse: Pixels,
    awal_teks: Pixels,
    jendela: &Window,
) -> usize {
    let posisi_x = posisi_mouse - awal_teks;

    if teks.is_empty() || posisi_x <= px(0.0) {
        return 0;
    }

    let bentuk = bentuk_teks_nama_tab(teks, jendela);

    let indeks_byte = bentuk.closest_index_for_x(posisi_x).min(teks.len());

    teks.char_indices()
        .take_while(|(indeks, _)| *indeks < indeks_byte)
        .count()
}

impl JendelaTerminal {
    pub(super) fn buka_edit_nama_tab(
        &mut self,
        id: IdTab,
        jendela: &mut Window,
        konteks: &mut Context<Self>,
    ) {
        if self.tarikan_panel.is_some() || self.ruang().mouse_tui_ditekan.is_some() {
            return;
        }

        let Some(nama) = self.manajer.nama_tab(id).map(str::to_owned) else {
            return;
        };

        self.batalkan_semua_tombol();

        self.menu_split = None;

        let mut masukan = MasukanTerminal::baru();
        masukan.ganti(nama);

        self.edit_nama_tab = Some(EditNamaTab {
            id,
            masukan,
            jangkar_seleksi: None,
            menarik_seleksi: false,
        });

        jendela.focus(&self.fokus, konteks);

        konteks.notify();
    }

    pub(super) fn tangani_tombol_edit_nama_tab(
        &mut self,
        kejadian: &KeyDownEvent,
        konteks: &mut Context<Self>,
    ) -> bool {
        let Some(edit) = self.edit_nama_tab.as_mut() else {
            return false;
        };

        let tombol = &kejadian.keystroke;

        let nama = tombol.key.to_ascii_lowercase();

        edit.menarik_seleksi = false;

        if matches!(nama.as_str(), "escape" | "esc") {
            self.edit_nama_tab = None;

            konteks.notify();

            return true;
        }

        if matches!(nama.as_str(), "enter" | "return") {
            if self.manajer.ubah_nama_tab(edit.id, edit.masukan.teks()) {
                self.edit_nama_tab = None;
            }

            konteks.notify();

            return true;
        }

        if tombol.modifiers.control {
            match nama.as_str() {
                "a" => {
                    edit.pilih_semua();
                }

                "c" => {
                    if let Some(rentang) = edit.rentang_seleksi() {
                        let teks = edit.masukan.teks_rentang(rentang).to_owned();

                        konteks.write_to_clipboard(ClipboardItem::new_string(teks));
                    }
                }

                "x" => {
                    if let Some(rentang) = edit.rentang_seleksi() {
                        let teks = edit.masukan.teks_rentang(rentang).to_owned();

                        konteks.write_to_clipboard(ClipboardItem::new_string(teks));

                        edit.hapus_seleksi();
                    }
                }

                "v" => {
                    if let Some(teks) = konteks.read_from_clipboard().and_then(|isi| isi.text()) {
                        edit.sisipkan(&teks);
                    }
                }

                _ => {}
            }

            konteks.notify();

            return true;
        }

        let seleksi = tombol.modifiers.shift;

        match nama.as_str() {
            "backspace" => {
                if !edit.hapus_seleksi() {
                    edit.masukan.hapus_sebelum();
                }
            }

            "delete" => {
                if !edit.hapus_seleksi() {
                    edit.masukan.hapus_sesudah();
                }
            }

            "left" | "arrowleft" => {
                edit.pindah_kiri(seleksi);
            }

            "right" | "arrowright" => {
                edit.pindah_kanan(seleksi);
            }

            "home" => {
                edit.pindah_kursor(0, seleksi);
            }

            "end" => {
                let akhir = edit.masukan.teks().chars().count();

                edit.pindah_kursor(akhir, seleksi);
            }

            _ => {
                if !tombol.modifiers.alt
                    && !tombol.modifiers.platform
                    && let Some(teks) = tombol.key_char.as_deref()
                {
                    edit.sisipkan(teks);
                }
            }
        }

        konteks.notify();

        true
    }

    pub(super) fn buat_edit_nama_tab(
        &self,
        jendela: &Window,
        konteks: &mut Context<Self>,
    ) -> impl IntoElement {
        let edit = self
            .edit_nama_tab
            .as_ref()
            .expect("Editor nama tab tidak tersedia");

        let lebar = (f32::from(jendela.viewport_size().width) - 16.0).clamp(1.0, 320.0);

        let teks = edit.masukan.teks().to_owned();

        let bentuk = (!teks.is_empty()).then(|| bentuk_teks_nama_tab(&teks, jendela));

        let posisi_kursor = bentuk.as_ref().map_or(px(0.0), |bentuk| {
            bentuk.x_for_index(edit.masukan.bagian_kursor().0.len())
        });

        let rentang_seleksi = edit.rentang_seleksi();

        let awal_teks = Rc::new(Cell::new(None::<Pixels>));

        let awal_teks_klik = Rc::clone(&awal_teks);
        let awal_teks_gerak = Rc::clone(&awal_teks);
        let awal_teks_lepas = Rc::clone(&awal_teks);

        let mut isi_input = div()
            .debug_selector(|| "rename-tab-text".to_owned())
            .relative()
            .flex_none()
            .h(px(18.0))
            .whitespace_nowrap();

        if let Some(rentang) = rentang_seleksi.as_ref()
            && let Some(bentuk) = bentuk.as_ref()
        {
            let awal_byte = edit.masukan.teks_rentang(0..rentang.start).len();

            let akhir_byte = edit.masukan.teks_rentang(0..rentang.end).len();

            let awal = bentuk.x_for_index(awal_byte);
            let akhir = bentuk.x_for_index(akhir_byte);

            isi_input = isi_input.child(
                div()
                    .absolute()
                    .left(awal)
                    .top(px(0.0))
                    .w(akhir - awal)
                    .h(px(18.0))
                    .bg(rgb(WARNA_SELEKSI)),
            );
        }

        isi_input = isi_input.child(teks);

        if rentang_seleksi.is_none() {
            isi_input = isi_input.child(
                div()
                    .absolute()
                    .left(posisi_kursor)
                    .top(px(0.0))
                    .w(px(1.0))
                    .h(px(18.0))
                    .bg(rgb(WARNA_AKSEN)),
            );
        }

        div()
            .id("lapisan-edit-nama-tab")
            .absolute()
            .inset_0()
            .occlude()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba(0x00000066))
            .on_mouse_down(
                MouseButton::Left,
                konteks.listener(|terminal, _, _, konteks| {
                    terminal.edit_nama_tab = None;

                    konteks.stop_propagation();
                    konteks.notify();
                }),
            )
            .on_mouse_move(konteks.listener(
                move |terminal, kejadian: &MouseMoveEvent, jendela, konteks| {
                    konteks.stop_propagation();

                    let Some(edit) = terminal.edit_nama_tab.as_mut() else {
                        return;
                    };

                    if !edit.menarik_seleksi {
                        return;
                    }

                    if kejadian.pressed_button != Some(MouseButton::Left) {
                        edit.menarik_seleksi = false;

                        return;
                    }

                    let Some(awal) = awal_teks_gerak.get() else {
                        return;
                    };

                    let posisi =
                        posisi_dari_mouse(edit.masukan.teks(), kejadian.position.x, awal, jendela);

                    edit.pindah_kursor(posisi, true);

                    konteks.notify();
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                konteks.listener(move |terminal, kejadian: &MouseUpEvent, jendela, konteks| {
                    konteks.stop_propagation();

                    let Some(edit) = terminal.edit_nama_tab.as_mut() else {
                        return;
                    };

                    if edit.menarik_seleksi {
                        if let Some(awal) = awal_teks_lepas.get() {
                            let posisi = posisi_dari_mouse(
                                edit.masukan.teks(),
                                kejadian.position.x,
                                awal,
                                jendela,
                            );

                            edit.pindah_kursor(posisi, true);
                        }

                        edit.menarik_seleksi = false;

                        konteks.notify();
                    }
                }),
            )
            .child(
                div()
                    .w(px(lebar))
                    .flex()
                    .flex_col()
                    .p_3()
                    .gap_2()
                    .rounded_md()
                    .bg(rgba((WARNA_EDITOR << 8) | 0xcc))
                    .on_mouse_down(
                        MouseButton::Left,
                        konteks.listener(|_, _, _, konteks| {
                            konteks.stop_propagation();
                        }),
                    )
                    .child(div().text_color(rgb(WARNA_TEKS)).child("Rename"))
                    .child(
                        div()
                            .on_children_prepainted(move |batas, _, _| {
                                awal_teks.set(batas.first().map(|batas| batas.left()));
                            })
                            .id("input-edit-nama-tab")
                            .debug_selector(|| "rename-tab-input".to_owned())
                            .w_full()
                            .h(px(36.0))
                            .px(px(10.0))
                            .flex()
                            .items_center()
                            .overflow_hidden()
                            .rounded_sm()
                            .cursor(CursorStyle::IBeam)
                            .font_family("Consolas")
                            .text_size(px(14.0))
                            .line_height(px(18.0))
                            .bg(rgb(WARNA_INPUT))
                            .text_color(rgb(WARNA_TEKS))
                            .on_mouse_down(
                                MouseButton::Left,
                                konteks.listener(
                                    move |terminal, kejadian: &MouseDownEvent, jendela, konteks| {
                                        konteks.stop_propagation();

                                        let Some(edit) = terminal.edit_nama_tab.as_mut() else {
                                            return;
                                        };

                                        if kejadian.click_count >= 2 {
                                            edit.pilih_semua();
                                        } else {
                                            let posisi = posisi_dari_mouse(
                                                edit.masukan.teks(),
                                                kejadian.position.x,
                                                awal_teks_klik.get().unwrap_or(kejadian.position.x),
                                                jendela,
                                            );

                                            edit.pindah_kursor(posisi, kejadian.modifiers.shift);

                                            edit.jangkar_seleksi.get_or_insert(posisi);

                                            edit.menarik_seleksi = true;
                                        }

                                        jendela.focus(&terminal.fokus, konteks);

                                        konteks.notify();
                                    },
                                ),
                            )
                            .child(isi_input),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(WARNA_AKSEN))
                            .child("Enter : Save | Esc : Cancel"),
                    ),
            )
    }
}

#[cfg(test)]
#[path = "../../tests/unit/regresi_editor_nama_tab.rs"]
mod regresi_editor_nama_tab;

#[cfg(test)]
#[path = "../../tests/unit/regresi_interaksi_nama_tab.rs"]
mod regresi_interaksi_nama_tab;
