//! Flat-colour placeholder tiles. Magenta edges and a "PH" stamp mark them
//! as stand-ins. Real art replaces the file; the id stays.

use std::fs;
use std::io;
use std::path::Path;

use crate::assets::{Asset, AssetFamily, ASSETS};
use crate::project::{Facing, SIT_X, SIT_Y};

struct Image {
    w: i32,
    h: i32,
    px: Vec<u8>,
}

impl Image {
    fn new(w: i32, h: i32) -> Self {
        Self {
            w,
            h,
            px: vec![0; (w * h * 4) as usize],
        }
    }

    fn set(&mut self, x: i32, y: i32, color: [u8; 4]) {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return;
        }
        let i = ((y * self.w + x) * 4) as usize;
        self.px[i..i + 4].copy_from_slice(&color);
    }

    #[cfg(test)]
    fn get(&self, x: i32, y: i32) -> [u8; 4] {
        let i = ((y * self.w + x) * 4) as usize;
        self.px[i..i + 4].try_into().unwrap()
    }

    fn write_png(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, encode_png(self))
    }
}

pub fn write_asset_files(assets_dir: &Path) -> io::Result<()> {
    let mut csv = String::from("id,file,canvas_w,canvas_h,sit_x,sit_y,datum_y,family,note\n");
    for asset in ASSETS {
        let image = render(asset);
        image.write_png(&assets_dir.join(asset.file))?;
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            asset.id,
            asset.file,
            asset.canvas_w,
            asset.canvas_h,
            asset.sit_x,
            asset.sit_y,
            asset.datum_y,
            asset.family.as_str(),
            asset.note
        ));
    }
    let catalog = assets_dir.join("catalog");
    fs::create_dir_all(&catalog)?;
    fs::write(catalog.join(".gdignore"), b"")?;
    fs::write(catalog.join("asset-list.csv"), csv)
}

fn render(asset: &Asset) -> Image {
    let mut image = Image::new(asset.canvas_w, asset.canvas_h);
    match asset.family {
        AssetFamily::Ship => draw_ship(&mut image, facing_of(asset.id)),
        AssetFamily::Port => {
            fill_diamond(&mut image, [232, 208, 150, 255], 36, 18);
            stamp(&mut image);
        }
        AssetFamily::Lane if asset.id.ends_with("blocked") => {
            fill_diamond(&mut image, family_color(asset.family, asset.id), 64, 32);
            draw_cross(&mut image);
            stamp(&mut image);
        }
        _ => {
            fill_diamond(&mut image, family_color(asset.family, asset.id), 64, 32);
            stamp(&mut image);
        }
    }
    image
}

fn family_color(family: AssetFamily, id: &str) -> [u8; 4] {
    match family {
        AssetFamily::Sea => [27, 79, 128, 255],
        AssetFamily::Water => [46, 120, 176, 255],
        AssetFamily::Quay => [168, 132, 86, 255],
        AssetFamily::Pier => [110, 86, 62, 255],
        AssetFamily::Port => [232, 208, 150, 255],
        AssetFamily::Ship => [236, 228, 210, 255],
        AssetFamily::Lane if id.ends_with("warning") => [214, 164, 48, 255],
        AssetFamily::Lane if id.ends_with("blocked") => [196, 72, 72, 255],
        AssetFamily::Lane if id.ends_with("underway") => [236, 224, 196, 255],
        AssetFamily::Lane => [36, 168, 150, 255],
    }
}

fn in_diamond(x: i32, y: i32, half_w: i32, half_h: i32) -> bool {
    let dx = (x - SIT_X).abs();
    let dy = (y - SIT_Y).abs();
    dx * half_h + dy * half_w <= half_w * half_h
}

fn fill_diamond(image: &mut Image, fill: [u8; 4], half_w: i32, half_h: i32) {
    let edge = [255, 0, 255, 255];
    for y in 0..image.h {
        for x in 0..image.w {
            if !in_diamond(x, y, half_w, half_h) {
                continue;
            }
            let border = !in_diamond(x - 1, y, half_w, half_h)
                || !in_diamond(x + 1, y, half_w, half_h)
                || !in_diamond(x, y - 1, half_w, half_h)
                || !in_diamond(x, y + 1, half_w, half_h);
            image.set(x, y, if border { edge } else { fill });
        }
    }
}

fn draw_cross(image: &mut Image) {
    let ink = [40, 16, 16, 255];
    for step in 0..48 {
        let x = SIT_X - 24 + step;
        let y0 = SIT_Y - 16 + step * 32 / 48;
        let y1 = SIT_Y + 16 - step * 32 / 48;
        image.set(x, y0, ink);
        image.set(x, y0 + 1, ink);
        image.set(x, y1, ink);
        image.set(x, y1 + 1, ink);
    }
}

fn draw_ship(image: &mut Image, facing: Facing) {
    let angle = facing.index() as f64 * std::f64::consts::FRAC_PI_4;
    let dx = angle.cos();
    let dy = angle.sin();
    let hull = [236, 228, 210, 255];
    let bow = [42, 48, 58, 255];
    let edge = [255, 0, 255, 255];
    for y in 0..image.h {
        for x in 0..image.w {
            let rx = f64::from(x - SIT_X);
            let ry = f64::from(y - SIT_Y);
            let along = rx * dx + ry * dy;
            let across = -rx * dy + ry * dx;
            let body = (-16.0..14.0).contains(&along) && across.abs() < 8.0;
            let nose = (12.0..28.0).contains(&along) && across.abs() < (28.0 - along) * 0.55;
            if nose {
                image.set(x, y, bow);
            } else if body {
                image.set(x, y, hull);
            }
        }
    }
    // Magenta rim so the glyph reads as a placeholder, not finished art.
    let snapshot = image.px.clone();
    for y in 1..image.h - 1 {
        for x in 1..image.w - 1 {
            let i = ((y * image.w + x) * 4) as usize;
            if snapshot[i + 3] == 0 {
                continue;
            }
            let neighbor_clear = |ox: i32, oy: i32| {
                let nx = x + ox;
                let ny = y + oy;
                let j = ((ny * image.w + nx) * 4) as usize;
                snapshot[j + 3] == 0
            };
            if neighbor_clear(-1, 0)
                || neighbor_clear(1, 0)
                || neighbor_clear(0, -1)
                || neighbor_clear(0, 1)
            {
                image.set(x, y, edge);
            }
        }
    }
    stamp(image);
}

fn facing_of(id: &str) -> Facing {
    match id.rsplit('.').next().unwrap_or("e") {
        "se" => Facing::Se,
        "s" => Facing::S,
        "sw" => Facing::Sw,
        "w" => Facing::W,
        "nw" => Facing::Nw,
        "n" => Facing::N,
        "ne" => Facing::Ne,
        _ => Facing::E,
    }
}

/// Block letters "PH", the placeholder stamp.
fn stamp(image: &mut Image) {
    const P: [u8; 5] = [0b1110, 0b1001, 0b1110, 0b1000, 0b1000];
    const H: [u8; 5] = [0b1001, 0b1001, 0b1111, 0b1001, 0b1001];
    let ink = [255, 0, 255, 255];
    blit_glyph(image, 8, 18, &P, ink);
    blit_glyph(image, 20, 18, &H, ink);
}

fn blit_glyph(image: &mut Image, ox: i32, oy: i32, rows: &[u8], color: [u8; 4]) {
    for (row, bits) in rows.iter().enumerate() {
        for col in 0..4 {
            if bits & (1 << (3 - col)) == 0 {
                continue;
            }
            let x = ox + col * 2;
            let y = oy + i32::try_from(row).unwrap_or(0) * 2;
            image.set(x, y, color);
            image.set(x + 1, y, color);
            image.set(x, y + 1, color);
            image.set(x + 1, y + 1, color);
        }
    }
}

fn encode_png(image: &Image) -> Vec<u8> {
    let mut raw = Vec::with_capacity((image.h * (1 + image.w * 4)) as usize);
    for y in 0..image.h {
        raw.push(0);
        let start = (y * image.w * 4) as usize;
        let end = start + (image.w * 4) as usize;
        raw.extend_from_slice(&image.px[start..end]);
    }
    let mut out = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(image.w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(image.h as u32).to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &zlib_store(&raw));
    chunk(&mut out, b"IEND", &[]);
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let mut signed = kind.to_vec();
    signed.extend_from_slice(data);
    out.extend_from_slice(&crc32(&signed).to_be_bytes());
}

fn zlib_store(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    let mut index = 0;
    loop {
        let end = (index + 65535).min(data.len());
        let last = end == data.len();
        out.push(if last { 1 } else { 0 });
        let len = (end - index) as u16;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&(!len).to_le_bytes());
        out.extend_from_slice(&data[index..end]);
        index = end;
        if last {
            break;
        }
    }
    out.extend_from_slice(&adler32(data).to_be_bytes());
    out
}

fn adler32(data: &[u8]) -> u32 {
    let mut s1: u32 = 1;
    let mut s2: u32 = 0;
    for &byte in data {
        s1 = (s1 + u32::from(byte)) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::asset;

    #[test]
    fn quay_sit_pixel_is_opaque_and_stamped() {
        let quay = asset("chart.tile.quay").unwrap();
        let image = render(quay);
        let center = image.get(SIT_X, SIT_Y);
        assert_eq!(center, [168, 132, 86, 255]);
        let stamp = image.get(8, 18);
        assert_eq!(stamp, [255, 0, 255, 255]);
        let png = encode_png(&image);
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
        let mut decoder = png::Decoder::new(std::io::Cursor::new(png));
        decoder.set_transformations(png::Transformations::IDENTITY);
        let mut reader = decoder.read_info().unwrap();
        assert_eq!(reader.info().width, quay.canvas_w as u32);
        assert_eq!(reader.info().height, quay.canvas_h as u32);
        let mut buf = vec![0; reader.output_buffer_size()];
        reader.next_frame(&mut buf).unwrap();
        let i = ((SIT_Y * quay.canvas_w + SIT_X) * 4) as usize;
        assert_eq!(&buf[i..i + 4], &[168, 132, 86, 255]);
    }

    #[test]
    fn committed_placeholders_match_the_generator() {
        let committed = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../godot/assets");
        let fresh =
            std::env::temp_dir().join(format!("portlight-placeholders-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&fresh);
        write_asset_files(&fresh).unwrap();
        let committed_csv = std::fs::read(committed.join("catalog/asset-list.csv")).unwrap();
        let fresh_csv = std::fs::read(fresh.join("catalog/asset-list.csv")).unwrap();
        assert_eq!(committed_csv, fresh_csv);
        for asset in ASSETS {
            let on_disk = std::fs::read(committed.join(asset.file)).unwrap();
            let generated = std::fs::read(fresh.join(asset.file)).unwrap();
            assert_eq!(on_disk, generated, "{}", asset.id);
        }
        let _ = std::fs::remove_dir_all(&fresh);
    }

    #[test]
    fn ship_glyph_has_eight_distinct_silhouettes() {
        let mut masks = std::collections::HashSet::new();
        for index in 0..8 {
            let facing = Facing::from_index(index);
            let asset = crate::assets::ship_asset(facing);
            let image = render(asset);
            assert_eq!(image.get(8, 18)[0], 255, "stamp {}", asset.id);
            let opaque: Vec<u8> = image.px.chunks(4).map(|px| u8::from(px[3] != 0)).collect();
            assert!(masks.insert(opaque), "duplicate silhouette {}", asset.id);
        }
        assert_eq!(masks.len(), 8);
    }
}
