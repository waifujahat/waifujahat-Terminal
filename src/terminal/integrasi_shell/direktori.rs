/*
 * Waifujahat Terminal
 * Author : waifujahat
 * Module : Terminal / Working directory
 * Purpose : Decodes and validates local Windows working directory URIs
 */

use std::path::PathBuf;

const BATAS_URI_DIREKTORI: usize = 32 * 1024;

pub(super) fn uraikan_direktori(data: &[u8]) -> Option<PathBuf> {
    if data.len() > BATAS_URI_DIREKTORI {
        return None;
    }

    let uri = std::str::from_utf8(data).ok()?;

    let (skema, sisa) = uri.split_once("://")?;

    if !skema.eq_ignore_ascii_case("file") {
        return None;
    }

    let (host, jalur) = sisa.split_once('/')?;

    if !host.is_empty() && !host.eq_ignore_ascii_case("localhost") {
        return None;
    }

    let awalan = jalur.as_bytes().get(..3)?;

    if !awalan[0].is_ascii_alphabetic() || awalan[1] != b':' || awalan[2] != b'/' {
        return None;
    }

    let mut direktori = format!("{}:\\", char::from(awalan[0]));

    for bagian in jalur[3..].split_terminator('/') {
        let bagian = dekode_komponen(bagian)?;

        if !komponen_valid(&bagian) {
            return None;
        }

        if !direktori.ends_with('\\') {
            direktori.push('\\');
        }

        direktori.push_str(&bagian);
    }

    Some(PathBuf::from(direktori))
}

fn dekode_komponen(teks: &str) -> Option<String> {
    let data = teks.as_bytes();
    let mut hasil = Vec::with_capacity(data.len());

    let mut indeks = 0;

    while indeks < data.len() {
        let byte = data[indeks];

        if byte == b'%' {
            let atas = nilai_heksadesimal(*data.get(indeks + 1)?)?;
            let bawah = nilai_heksadesimal(*data.get(indeks + 2)?)?;

            hasil.push((atas << 4) | bawah);

            indeks += 3;

            continue;
        }

        if !byte.is_ascii_alphanumeric() && !b"-._~!$&'()*+,=@".contains(&byte) {
            return None;
        }

        hasil.push(byte);

        indeks += 1;
    }

    String::from_utf8(hasil).ok()
}

fn nilai_heksadesimal(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn komponen_valid(komponen: &str) -> bool {
    if komponen.is_empty() || komponen == "." || komponen == ".." || komponen.ends_with(['.', ' '])
    {
        return false;
    }

    !komponen.chars().any(|karakter| {
        karakter.is_control()
            || matches!(
                karakter,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            )
    })
}
