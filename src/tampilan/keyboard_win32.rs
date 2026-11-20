/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Interface / Win32 keyboard
 * Purpose: Correlates native Windows keyboard metadata with interface events
 */

use std::cell::Cell;
use std::ptr::null_mut;

use anyhow::{Result, anyhow, ensure};
use gpui::Window;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::{
        Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardState, ToUnicodeEx, VkKeyScanExW},
        WindowsAndMessaging::{
            CallNextHookEx, GetWindowThreadProcessId, HC_ACTION, HHOOK, MSG, PM_REMOVE,
            SetWindowsHookExW, UnhookWindowsHookEx, WH_GETMESSAGE, WM_KEYDOWN, WM_KEYUP,
            WM_SYSKEYDOWN, WM_SYSKEYUP,
        },
    },
};

use waifujahat_terminal::terminal::{MetadataTombolWin32, RekamanTombolWin32};

struct CuplikanTombolWin32 {
    metadata: MetadataTombolWin32,
    keadaan: [u8; 256],
}

thread_local! {
    static JENDELA_NATIVE: Cell<HWND> = const {
        Cell::new(null_mut())
    };

    static PESAN_TERAKHIR: Cell<Option<CuplikanTombolWin32>> = const {
        Cell::new(None)
    };
}

pub(super) struct PengamatKeyboardWin32 {
    hook: HHOOK,
}

pub(super) fn kode_virtual_navigasi(nama: &str) -> Option<u16> {
    match nama {
        "left" | "arrowleft" => Some(0x25),
        "up" | "arrowup" => Some(0x26),
        "right" | "arrowright" => Some(0x27),
        "down" | "arrowdown" => Some(0x28),
        "home" => Some(0x24),
        "end" => Some(0x23),
        "insert" => Some(0x2D),
        "delete" => Some(0x2E),
        "pageup" | "page-up" | "page_up" => Some(0x21),
        "pagedown" | "page-down" | "page_down" => Some(0x22),

        _ => {
            let nomor = nama.strip_prefix('f')?.parse::<u16>().ok()?;

            (1..=12).contains(&nomor).then(|| 0x70 + nomor - 1)
        }
    }
}

pub(super) fn kode_virtual_keyboard(nama: &str) -> Option<u16> {
    let nama_normal = nama.to_ascii_lowercase();

    if let Some(kode) = kode_virtual_navigasi(&nama_normal) {
        return Some(kode);
    }

    match nama_normal.as_str() {
        "enter" | "return" => return Some(0x0D),
        "tab" | "backtab" => return Some(0x09),
        "backspace" => return Some(0x08),
        "escape" | "esc" => return Some(0x1B),
        "space" => return Some(0x20),

        _ => {}
    }

    let mut karakter = nama.chars();

    let nilai = karakter.next()?;

    if karakter.next().is_some() || !nilai.is_ascii() || nilai.is_ascii_control() {
        return None;
    }

    let hasil = unsafe { VkKeyScanExW(nilai as u16, GetKeyboardLayout(0)) };

    if hasil == -1 {
        return None;
    }

    Some((hasil as u16) & 0x00FF)
}

impl PengamatKeyboardWin32 {
    pub(super) fn pasang(jendela: &Window) -> Result<Self> {
        let handle = HasWindowHandle::window_handle(jendela)
            .map_err(|kesalahan| anyhow!("Gagal memperoleh handle jendela: {kesalahan:?}"))?;

        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            return Err(anyhow!("Handle jendela bukan Win32"));
        };

        let hwnd = handle.hwnd.get() as HWND;

        ensure!(
            JENDELA_NATIVE.with(|nilai| nilai.get().is_null()),
            "Pengamat keyboard sudah terpasang"
        );

        let thread = unsafe { GetCurrentThreadId() };

        ensure!(
            unsafe { GetWindowThreadProcessId(hwnd, null_mut()) } == thread,
            "Jendela tidak dimiliki oleh thread saat ini"
        );

        let hook =
            unsafe { SetWindowsHookExW(WH_GETMESSAGE, Some(amati_pesan), null_mut(), thread) };

        if hook.is_null() {
            return Err(std::io::Error::last_os_error().into());
        }

        JENDELA_NATIVE.with(|nilai| nilai.set(hwnd));

        Ok(Self { hook })
    }

    pub(super) fn ambil(
        &self,
        kode_virtual: u16,
        ditekan: bool,
        diulang: Option<bool>,
    ) -> Option<RekamanTombolWin32> {
        let cuplikan = PESAN_TERAKHIR.try_with(Cell::take).ok().flatten()?;

        let metadata = cuplikan.metadata;

        if metadata.kode_virtual != kode_virtual
            || metadata.ditekan != ditekan
            || diulang.is_some_and(|ulang| metadata.ditekan_sebelumnya != ulang)
        {
            return None;
        }

        let mut buffer = [0u16; 16];

        let kode_pindai =
            u32::from(metadata.kode_pindai) | if metadata.ditekan { 0 } else { 0x8000 };

        let jumlah = unsafe {
            ToUnicodeEx(
                u32::from(metadata.kode_virtual),
                kode_pindai,
                cuplikan.keadaan.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as i32,
                0x06,
                GetKeyboardLayout(0),
            )
        };

        let unicode = match jumlah {
            0 => 0,
            1 => buffer[0],
            _ => return None,
        };

        Some(metadata.ke_rekaman(unicode, &cuplikan.keadaan))
    }
}

impl Drop for PengamatKeyboardWin32 {
    fn drop(&mut self) {
        let _ = JENDELA_NATIVE.try_with(|nilai| nilai.set(null_mut()));
        let _ = PESAN_TERAKHIR.try_with(|nilai| nilai.set(None));

        unsafe {
            UnhookWindowsHookEx(self.hook);
        }
    }
}

unsafe extern "system" fn amati_pesan(kode: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if kode == HC_ACTION as i32 && wparam == PM_REMOVE as usize {
        let _ = PESAN_TERAKHIR.try_with(|slot| {
            slot.set(None);

            if lparam == 0 {
                return;
            }

            let pesan = unsafe { &*(lparam as *const MSG) };

            let hwnd = JENDELA_NATIVE.try_with(Cell::get).unwrap_or(null_mut());

            if hwnd.is_null()
                || pesan.hwnd != hwnd
                || !matches!(
                    pesan.message,
                    WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP
                )
            {
                return;
            }

            let Some(metadata) =
                MetadataTombolWin32::dari_pesan(pesan.wParam as u16, pesan.lParam as u32)
            else {
                return;
            };

            let ditekan = matches!(pesan.message, WM_KEYDOWN | WM_SYSKEYDOWN);

            if metadata.ditekan != ditekan {
                return;
            }

            let mut keadaan = [0u8; 256];

            if unsafe { GetKeyboardState(keadaan.as_mut_ptr()) } == 0 {
                return;
            }

            slot.set(Some(CuplikanTombolWin32 { metadata, keadaan }));
        });
    }

    unsafe { CallNextHookEx(null_mut(), kode, wparam, lparam) }
}
