/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Bracketed paste
 * Purpose: Encodes clipboard text for terminal applications
 */

pub fn sandi_tempel(teks: &str, berbingkai: bool) -> Vec<u8> {
    let teks = teks.replace("\r\n", "\n").replace('\r', "\n");

    if !berbingkai {
        return teks.replace('\n', " ").into_bytes();
    }

    let teks = teks.replace('\x1b', "");

    format!("\x1b[200~{teks}\x1b[201~").into_bytes()
}

#[cfg(test)]
mod pengujian {
    use super::sandi_tempel;

    #[test]
    fn tempel_biasa_menggabungkan_baris() {
        let hasil = sandi_tempel("baris satu\nbaris dua", false);

        assert_eq!(hasil, b"baris satu baris dua");
    }

    #[test]
    fn tempel_berbingkai_mempertahankan_baris() {
        let hasil = sandi_tempel("baris satu\r\nbaris dua", true);

        assert_eq!(hasil, b"\x1b[200~baris satu\nbaris dua\x1b[201~");
    }

    #[test]
    fn tempel_berbingkai_menghapus_escape() {
        let hasil = sandi_tempel("awal\x1bakhir", true);

        assert_eq!(hasil, b"\x1b[200~awalakhir\x1b[201~");
    }
}
