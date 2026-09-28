//! Flat-colour placeholder tiles. Magenta edges and a "PH" stamp mark them
//! as stand-ins. Real art replaces the file; the id stays.

use std::fs;
use std::io;
use std::path::Path;

use crate::assets::{Asset, AssetFamily, ASSETS, SLOOP_ANCHOR, SLOOP_HULL_PX};
use crate::project::{uv_to_screen_delta, Facing};

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
    let mut csv = String::from("id,file,canvas_w,canvas_h,anchor_x,anchor_y,family,note\n");
    for asset in ASSETS {
        // Approved landing plates are committed byte-identical. The generator
        // must not rewrite them.
        if asset.note == crate::assets::PLACEHOLDER {
            let image = render(asset);
            image.write_png(&assets_dir.join(asset.file))?;
        }
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            asset.id,
            asset.file,
            asset.canvas_w,
            asset.canvas_h,
            asset.anchor_x,
            asset.anchor_y,
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
        AssetFamily::ChartWater => {
            let fill = match asset.id {
                id if id.ends_with("_b") => [28, 78, 108, 255],
                id if id.ends_with("_c") => [18, 58, 88, 255],
                _ => [22, 68, 96, 255],
            };
            fill_diamond(&mut image, (64, 32), 64, 32, fill);
            stamp(&mut image, 4, 4);
        }
        AssetFamily::Port => {
            fill_diamond(&mut image, (64, 95), 64, 32, [168, 156, 140, 255]);
            draw_tower(&mut image);
            stamp(&mut image, 8, 8);
        }
        AssetFamily::Ship => {
            draw_ship(&mut image, facing_of(asset.id));
            stamp(&mut image, 2, 2);
        }
        AssetFamily::Wake => {
            draw_wake(&mut image);
            stamp(&mut image, 2, 2);
        }
    }
    image
}

fn in_diamond(x: i32, y: i32, center: (i32, i32), half_w: i32, half_h: i32) -> bool {
    let dx = (x - center.0).abs();
    let dy = (y - center.1).abs();
    dx * half_h + dy * half_w <= half_w * half_h
}

fn fill_diamond(image: &mut Image, center: (i32, i32), half_w: i32, half_h: i32, fill: [u8; 4]) {
    let edge = [255, 0, 255, 255];
    for y in 0..image.h {
        for x in 0..image.w {
            if !in_diamond(x, y, center, half_w, half_h) {
                continue;
            }
            let border = !in_diamond(x - 1, y, center, half_w, half_h)
                || !in_diamond(x + 1, y, center, half_w, half_h)
                || !in_diamond(x, y - 1, center, half_w, half_h)
                || !in_diamond(x, y + 1, center, half_w, half_h);
            image.set(x, y, if border { edge } else { fill });
        }
    }
}

fn draw_tower(image: &mut Image) {
    let stone = [150, 140, 126, 255];
    for y in 40..90 {
        for x in 56..72 {
            image.set(x, y, stone);
        }
    }
}

fn draw_wake(image: &mut Image) {
    let (cx, cy) = SLOOP_ANCHOR;
    let rx = f64::from(SLOOP_HULL_PX) * 0.9 / 2.0;
    let ry = rx / 2.0;
    let ink = [180, 210, 220, 180];
    for y in 0..image.h {
        for x in 0..image.w {
            let nx = f64::from(x - cx) / rx;
            let ny = f64::from(y - cy) / ry;
            let r = nx * nx + ny * ny;
            if (0.55..=1.0).contains(&r) {
                image.set(x, y, ink);
            }
        }
    }
}

fn draw_ship(image: &mut Image, facing: Facing) {
    let angle = facing.index() as f64 * std::f64::consts::FRAC_PI_4;
    let (sx, sy) = uv_to_screen_delta(angle.cos(), angle.sin());
    let len = (sx * sx + sy * sy).sqrt();
    let dx = sx / len;
    let dy = sy / len;
    let hull = [236, 228, 210, 255];
    let bow = [42, 48, 58, 255];
    let (ax, ay) = SLOOP_ANCHOR;
    let half = f64::from(SLOOP_HULL_PX) / 2.0;
    for y in 0..image.h {
        for x in 0..image.w {
            let rx = f64::from(x - ax);
            let ry = f64::from(y - ay);
            let along = rx * dx + ry * dy;
            let across = -rx * dy + ry * dx;
            let body = (-half..half * 0.45).contains(&along) && across.abs() < 5.0;
            let nose = (half * 0.35..half).contains(&along) && across.abs() < (half - along) * 0.45;
            if nose {
                image.set(x, y, bow);
            } else if body {
                image.set(x, y, hull);
            }
        }
    }
    rim(image);
}

fn rim(image: &mut Image) {
    let edge = [255, 0, 255, 255];
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
}

fn facing_of(id: &str) -> Facing {
    match id.rsplit('_').next().unwrap_or("f1") {
        "f0" => Facing::F0,
        "f2" => Facing::F2,
        "f3" => Facing::F3,
        "f4" => Facing::F4,
        "f5" => Facing::F5,
        "f6" => Facing::F6,
        "f7" => Facing::F7,
        _ => Facing::F1,
    }
}

/// Block letters "PH", the placeholder stamp.
fn stamp(image: &mut Image, ox: i32, oy: i32) {
    const P: [u8; 5] = [0b1110, 0b1001, 0b1110, 0b1000, 0b1000];
    const H: [u8; 5] = [0b1001, 0b1001, 0b1111, 0b1001, 0b1001];
    let ink = [255, 0, 255, 255];
    blit_glyph(image, ox, oy, &P, ink);
    blit_glyph(image, ox + 12, oy, &H, ink);
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
    fn chart_water_diamond_is_two_to_one_with_clear_corners() {
        let water = asset("chart_water_a").unwrap();
        let image = render(water);
        assert_eq!((image.w, image.h), (128, 64));
        for (x, y) in [(0, 0), (7, 7), (120, 0), (0, 56), (120, 56)] {
            assert_eq!(image.get(x, y)[3], 0, "corner {x},{y}");
        }
        assert_eq!(image.get(64, 0)[3], 255);
        assert_eq!(image.get(64, 63)[3], 255);
        assert_eq!(image.get(0, 32)[3], 255);
        assert_eq!(image.get(127, 32)[3], 255);
        let stamp = image.get(4, 4);
        assert_eq!(stamp, [255, 0, 255, 255]);
        // Upper-right edge: from the top vertex toward the right vertex, rise/run = 1/2.
        let mut samples = Vec::new();
        for y in 1..31 {
            let mut right = 0;
            for x in 0..image.w {
                if image.get(x, y)[3] > 0 {
                    right = x;
                }
            }
            samples.push((right, y));
        }
        let (x0, y0) = samples[0];
        let (x1, y1) = samples[samples.len() - 1];
        let slope = f64::from(y1 - y0) / f64::from(x1 - x0);
        let degrees = slope.atan().to_degrees();
        assert!(
            (degrees - 26.565).abs() < 0.3,
            "edge {degrees}° from ({x0},{y0}) to ({x1},{y1})"
        );
        let png = encode_png(&image);
        assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
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
            if asset.note != crate::assets::PLACEHOLDER {
                assert!(
                    !fresh.join(asset.file).exists(),
                    "generator wrote approved plate {}",
                    asset.id
                );
                continue;
            }
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
            assert_eq!(image.get(2, 2)[0], 255, "stamp {}", asset.id);
            assert_eq!(
                image.get(SLOOP_ANCHOR.0, SLOOP_ANCHOR.1)[3],
                255,
                "{}",
                asset.id
            );
            let opaque: Vec<u8> = image.px.chunks(4).map(|px| u8::from(px[3] != 0)).collect();
            assert!(masks.insert(opaque), "duplicate silhouette {}", asset.id);
        }
        assert_eq!(masks.len(), 8);
    }
}
