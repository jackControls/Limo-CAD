//! Outline font for characters a Latin-1 viewer font does not draw.
//! The DXF STYLE record still names the sheet family. These glyphs are also
//! written as closed polylines and the font bytes are embedded, because a
//! family name does not carry the outlines.
use super::{dxf_text, Primitive, P};

const EM: f64 = 1000.;

pub(super) fn needs_embed(items: &[Primitive]) -> bool {
    items.iter().any(|item| match item {
        Primitive::Text { value, .. } => value.chars().any(|ch| contours(ch).is_some()),
        _ => false,
    })
}

pub(super) fn write_section(out: &mut String, bytes: &[u8]) {
    use std::fmt::Write;
    let _ = write!(
        out,
        "0\nSECTION\n2\nNOBS_EMBEDDED_FONT\n1\nnobs-drawing.ttf\n90\n{}\n",
        bytes.len()
    );
    for chunk in bytes.chunks(100) {
        out.push_str("310\n");
        for byte in chunk {
            let _ = write!(out, "{byte:02X}");
        }
        out.push('\n');
    }
    out.push_str("0\nENDSEC\n");
}

pub(super) fn font_bytes() -> Vec<u8> {
    let mut glyphs = vec![(0u32, 0u16, 500u16, Vec::new())];
    let mut defined: Vec<char> = CONTOURS.iter().map(|(ch, _, _)| *ch).collect();
    defined.sort_unstable();
    defined.dedup();
    for (index, ch) in defined.into_iter().enumerate() {
        let (advance, contours) = contours(ch).unwrap();
        glyphs.push((ch as u32, (index + 1) as u16, advance, contours));
    }
    assemble(&glyphs)
}

pub(super) fn write_text(
    out: &mut String,
    sheet_height: f64,
    point: P,
    value: &str,
    height: f64,
    centered: bool,
    rotation_deg: f64,
    fitted_width: Option<f64>,
    layer: &str,
) {
    use std::fmt::Write;
    let _ = writeln!(
        out,
        "0\nTEXT\n8\n{layer}\n7\nSTANDARD\n10\n{:.5}\n20\n{:.5}\n40\n{height}\n1\n{}",
        point[0],
        sheet_height - point[1],
        dxf_text(value)
    );
    if let Some(width) = fitted_width {
        let _ = writeln!(
            out,
            "72\n5\n73\n0\n11\n{:.5}\n21\n{:.5}",
            point[0] + width,
            sheet_height - point[1]
        );
    } else if centered {
        let _ = writeln!(
            out,
            "72\n1\n11\n{:.5}\n21\n{:.5}",
            point[0],
            sheet_height - point[1]
        );
    }
    if rotation_deg != 0. {
        let _ = writeln!(out, "50\n{:.5}", -rotation_deg);
    }
    if hide_source(value) {
        out.push_str("60\n1\n");
    }
    write_outlines(
        out,
        sheet_height,
        point,
        value,
        height,
        centered,
        rotation_deg,
        fitted_width,
    );
}

fn hide_source(value: &str) -> bool {
    let mut drew = false;
    for ch in value.chars() {
        if ch.is_whitespace() || ch == '\u{fe0e}' {
            continue;
        }
        if (ch as u32) <= 0x00FF || contours(ch).is_none() {
            return false;
        }
        drew = true;
    }
    drew
}

fn write_outlines(
    out: &mut String,
    sheet_height: f64,
    point: P,
    value: &str,
    height: f64,
    centered: bool,
    rotation_deg: f64,
    fitted_width: Option<f64>,
) {
    use std::fmt::Write;
    let scale = height / EM;
    if !scale.is_finite() || scale <= 0. {
        return;
    }
    let natural: f64 = value
        .chars()
        .map(|ch| advance_mm(ch, height, scale))
        .sum();
    let x_scale = match fitted_width {
        Some(width) if natural > 1e-6 => width / natural,
        _ => 1.,
    };
    let mut pen = if centered { -natural * x_scale * 0.5 } else { 0. };
    let radians = rotation_deg.to_radians();
    let (sin, cos) = (radians.sin(), radians.cos());
    for ch in value.chars() {
        let advance = advance_mm(ch, height, scale) * x_scale;
        if let Some((_, contours)) = contours(ch) {
            for contour in contours {
                if contour.len() < 3 {
                    continue;
                }
                let _ = writeln!(out, "0\nLWPOLYLINE\n8\nGLYPH\n90\n{}\n70\n1", contour.len());
                for (x, y) in contour {
                    let dx = pen + f64::from(x) * scale * x_scale;
                    let dy = -f64::from(y) * scale;
                    let paper_x = point[0] + dx * cos - dy * sin;
                    let paper_y = point[1] + dx * sin + dy * cos;
                    let _ = writeln!(
                        out,
                        "10\n{:.5}\n20\n{:.5}",
                        paper_x,
                        sheet_height - paper_y
                    );
                }
            }
        }
        pen += advance;
    }
}

fn advance_mm(ch: char, height: f64, scale: f64) -> f64 {
    if ch.is_whitespace() {
        return height * 0.33;
    }
    if ch == '\u{fe0e}' {
        return 0.;
    }
    contours(ch).map_or_else(
        || crate::drawing_presentation::text::width(&ch.to_string(), height),
        |(advance, _)| f64::from(advance) * scale,
    )
}

fn contours(ch: char) -> Option<(u16, Vec<Vec<(i16, i16)>>)> {
    CONTOURS
        .iter()
        .find(|(defined, _, _)| *defined == ch)
        .map(|(_, advance, parts)| (*advance, parts.iter().copied().map(part_contours).flatten().collect()))
}

fn part_contours(part: Part) -> Vec<Vec<(i16, i16)>> {
    match part {
        Part::Stroke(x0, y0, x1, y1, width) => vec![thick(x0, y0, x1, y1, width)],
        Part::Ring(cx, cy, radius, width) => ring(cx, cy, radius, width),
    }
}

#[derive(Clone, Copy)]
enum Part {
    Stroke(i16, i16, i16, i16, i16),
    Ring(i16, i16, i16, i16),
}

fn thick(x0: i16, y0: i16, x1: i16, y1: i16, width: i16) -> Vec<(i16, i16)> {
    let dx = f64::from(x1 - x0);
    let dy = f64::from(y1 - y0);
    let len = dx.hypot(dy).max(1.);
    let half = f64::from(width.max(1)) * 0.5;
    let px = (-dy / len * half).round() as i16;
    let py = (dx / len * half).round() as i16;
    vec![
        (x0 + px, y0 + py),
        (x1 + px, y1 + py),
        (x1 - px, y1 - py),
        (x0 - px, y0 - py),
    ]
}

fn ring(cx: i16, cy: i16, radius: i16, width: i16) -> Vec<Vec<(i16, i16)>> {
    let outer = radius + width / 2;
    let inner = (radius - width / 2).max(1);
    vec![polygon(cx, cy, outer, false), polygon(cx, cy, inner, true)]
}

fn polygon(cx: i16, cy: i16, radius: i16, reverse: bool) -> Vec<(i16, i16)> {
    let mut points = Vec::new();
    for step in 0..16 {
        let angle = std::f64::consts::TAU * f64::from(step) / 16.;
        points.push((
            cx + (f64::from(radius) * angle.cos()).round() as i16,
            cy + (f64::from(radius) * angle.sin()).round() as i16,
        ));
    }
    if reverse {
        points.reverse();
    }
    points
}

fn assemble(glyphs: &[(u32, u16, u16, Vec<Vec<(i16, i16)>>)]) -> Vec<u8> {
    let glyf_built: Vec<Vec<u8>> = glyphs
        .iter()
        .map(|(_, _, _, contours)| glyf(contours))
        .collect();
    let mut glyf_table = Vec::new();
    let mut loca = Vec::new();
    for glyph in &glyf_built {
        loca.extend(u16((glyf_table.len() / 2) as u16));
        glyf_table.extend(glyph);
    }
    loca.extend(u16((glyf_table.len() / 2) as u16));
    let advances: Vec<u16> = glyphs.iter().map(|(_, _, advance, _)| *advance).collect();
    let bounds = glyf_built
        .iter()
        .filter(|glyph| glyph.len() >= 10)
        .map(|glyph| {
            (
                i16::from_be_bytes([glyph[2], glyph[3]]),
                i16::from_be_bytes([glyph[4], glyph[5]]),
                i16::from_be_bytes([glyph[6], glyph[7]]),
                i16::from_be_bytes([glyph[8], glyph[9]]),
            )
        })
        .fold((0i16, 0i16, 0i16, 0i16), |acc, (l, t, r, b)| {
            (acc.0.min(l), acc.1.min(t), acc.2.max(r), acc.3.max(b))
        });
    let max_points = glyphs
        .iter()
        .map(|(_, _, _, contours)| contours.iter().map(Vec::len).sum::<usize>())
        .max()
        .unwrap_or(0) as u16;
    let max_contours = glyphs
        .iter()
        .map(|(_, _, _, contours)| contours.len())
        .max()
        .unwrap_or(0) as u16;
    let num = glyphs.len() as u16;
    let mut tables = vec![
        (b"cmap", cmap(glyphs)),
        (b"glyf", glyf_table),
        (
            b"head",
            head(bounds.0, bounds.1, bounds.2, bounds.3),
        ),
        (
            b"hhea",
            hhea(*advances.iter().max().unwrap_or(&500), bounds.2, num),
        ),
        (b"hmtx", hmtx(&advances, &glyf_built)),
        (b"loca", loca),
        (b"maxp", maxp(num, max_points, max_contours)),
        (b"name", name_table()),
        (b"post", post()),
    ];
    tables.sort_by_key(|table| table.0);
    let count = tables.len() as u16;
    let mut power = 1u16;
    while power * 2 <= count {
        power *= 2;
    }
    let search = power * 16;
    let mut font = Vec::new();
    font.extend(u32(0x0001_0000));
    font.extend(u16(count));
    font.extend(u16(search));
    font.extend(u16(power.trailing_zeros() as u16));
    font.extend(u16(count * 16 - search));
    let directory = font.len();
    font.resize(directory + tables.len() * 16, 0);
    let mut head_at = 0usize;
    for (index, (tag, data)) in tables.iter().enumerate() {
        let offset = font.len();
        if *tag == b"head" {
            head_at = offset;
        }
        let record = directory + index * 16;
        font[record..record + 4].copy_from_slice(*tag);
        put_u32(&mut font, record + 4, checksum(data));
        put_u32(&mut font, record + 8, offset as u32);
        put_u32(&mut font, record + 12, data.len() as u32);
        font.extend(data);
        while font.len() % 4 != 0 {
            font.push(0);
        }
    }
    let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&font));
    put_u32(&mut font, head_at + 8, adjustment);
    let head_record = font[directory..]
        .chunks(16)
        .position(|record| &record[..4] == b"head")
        .unwrap();
    let checksum_at = directory + head_record * 16 + 4;
    let prior = u32::from_be_bytes(font[checksum_at..checksum_at + 4].try_into().unwrap());
    put_u32(&mut font, checksum_at, prior.wrapping_add(adjustment));
    font
}

fn glyf(contours: &[Vec<(i16, i16)>]) -> Vec<u8> {
    if contours.is_empty() {
        return Vec::new();
    }
    let points: Vec<(i16, i16)> = contours.iter().flatten().copied().collect();
    if points.is_empty() {
        return Vec::new();
    }
    let xmin = points.iter().map(|point| point.0).min().unwrap();
    let ymin = points.iter().map(|point| point.1).min().unwrap();
    let xmax = points.iter().map(|point| point.0).max().unwrap();
    let ymax = points.iter().map(|point| point.1).max().unwrap();
    let mut out = Vec::new();
    out.extend(i16(contours.len() as i16));
    out.extend(i16(xmin));
    out.extend(i16(ymin));
    out.extend(i16(xmax));
    out.extend(i16(ymax));
    let mut end = -1i32;
    for contour in contours {
        end += contour.len() as i32;
        out.extend(u16(end as u16));
    }
    out.extend(u16(0));
    let mut cursor = (0i16, 0i16);
    let mut flags = Vec::new();
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for point in points {
        flags.push(0x01);
        xs.extend(i16(point.0.wrapping_sub(cursor.0)));
        ys.extend(i16(point.1.wrapping_sub(cursor.1)));
        cursor = point;
    }
    out.extend(flags);
    out.extend(xs);
    out.extend(ys);
    if out.len() % 2 == 1 {
        out.push(0);
    }
    out
}

fn cmap(glyphs: &[(u32, u16, u16, Vec<Vec<(i16, i16)>>)]) -> Vec<u8> {
    let mapped: Vec<(u16, u16)> = glyphs
        .iter()
        .filter(|(code, _, _, _)| *code > 0 && *code <= 0xFFFF)
        .map(|(code, id, _, _)| (*code as u16, *id))
        .collect();
    let segments = mapped.len() + 1;
    let mut power = 1usize;
    while power * 2 <= segments {
        power *= 2;
    }
    let search = (power * 2) as u16;
    let mut out = Vec::new();
    out.extend(u16(0));
    out.extend(u16(1));
    out.extend(u16(3));
    out.extend(u16(1));
    out.extend(u32(12));
    out.extend(u16(4));
    let length = 16 + segments * 8;
    out.extend(u16(length as u16));
    out.extend(u16(0));
    out.extend(u16((segments * 2) as u16));
    out.extend(u16(search));
    out.extend(u16(power.trailing_zeros() as u16));
    out.extend(u16((segments * 2) as u16 - search));
    for (code, _) in &mapped {
        out.extend(u16(*code));
    }
    out.extend(u16(0xFFFF));
    out.extend(u16(0));
    for (code, _) in &mapped {
        out.extend(u16(*code));
    }
    out.extend(u16(0xFFFF));
    for (code, id) in &mapped {
        let delta = (i32::from(*id) - i32::from(*code)).rem_euclid(65536) as u16;
        out.extend(u16(delta));
    }
    out.extend(u16(1));
    for _ in 0..segments {
        out.extend(u16(0));
    }
    out
}

fn head(xmin: i16, ymin: i16, xmax: i16, ymax: i16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(u32(0x0001_0000));
    out.extend(u32(0x0001_0000));
    out.extend(u32(0));
    out.extend(u32(0x5F0F_3CF5));
    out.extend(u16(0x000B));
    out.extend(u16(1000));
    out.extend([0u8; 16]);
    out.extend(i16(xmin));
    out.extend(i16(ymin));
    out.extend(i16(xmax));
    out.extend(i16(ymax));
    out.extend(u16(0));
    out.extend(u16(8));
    out.extend(i16(2));
    out.extend(i16(0));
    out.extend(i16(0));
    out
}

fn hhea(advance_max: u16, xmax: i16, glyphs: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(u32(0x0001_0000));
    out.extend(i16(800));
    out.extend(i16(-200));
    out.extend(i16(0));
    out.extend(u16(advance_max));
    out.extend(i16(0));
    out.extend(i16(0));
    out.extend(i16(xmax));
    out.extend(i16(1));
    out.extend(i16(0));
    out.extend(i16(0));
    out.extend([0u8; 8]);
    out.extend(i16(0));
    out.extend(u16(glyphs));
    out
}

fn hmtx(advances: &[u16], glyf: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for (advance, glyph) in advances.iter().zip(glyf) {
        let lsb = if glyph.len() >= 4 {
            i16::from_be_bytes([glyph[2], glyph[3]])
        } else {
            0
        };
        out.extend(u16(*advance));
        out.extend(i16(lsb));
    }
    out
}

fn maxp(glyphs: u16, max_points: u16, max_contours: u16) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(u32(0x0001_0000));
    out.extend(u16(glyphs));
    out.extend(u16(max_points));
    out.extend(u16(max_contours));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(2));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(0));
    out.extend(u16(0));
    out
}

fn name_table() -> Vec<u8> {
    let text: Vec<u16> = "nobs-drawing".encode_utf16().collect();
    let mut out = Vec::new();
    out.extend(u16(0));
    out.extend(u16(1));
    out.extend(u16(18));
    out.extend(u16(3));
    out.extend(u16(1));
    out.extend(u16(0x0409));
    out.extend(u16(1));
    out.extend(u16((text.len() * 2) as u16));
    out.extend(u16(0));
    for unit in text {
        out.extend(u16(unit));
    }
    out
}

fn post() -> Vec<u8> {
    let mut out = vec![0u8; 32];
    out[..4].copy_from_slice(&0x0003_0000u32.to_be_bytes());
    out
}

fn checksum(data: &[u8]) -> u32 {
    let mut sum = 0u32;
    let mut index = 0;
    while index < data.len() {
        let mut word = [0u8; 4];
        let end = (index + 4).min(data.len());
        word[..end - index].copy_from_slice(&data[index..end]);
        sum = sum.wrapping_add(u32::from_be_bytes(word));
        index += 4;
    }
    sum
}

fn put_u32(buf: &mut [u8], offset: usize, value: u32) {
    buf[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn u16(value: u16) -> [u8; 2] {
    value.to_be_bytes()
}
fn i16(value: i16) -> [u8; 2] {
    value.to_be_bytes()
}
fn u32(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

const CONTOURS: &[(char, u16, &[Part])] = &[
    (
        '⌀',
        800,
        &[
            Part::Ring(400, 360, 250, 70),
            Part::Stroke(170, 150, 630, 570, 64),
        ],
    ),
    (
        '⊥',
        800,
        &[
            Part::Stroke(140, 180, 660, 180, 70),
            Part::Stroke(400, 180, 400, 720, 70),
        ],
    ),
    (
        '⌖',
        800,
        &[
            Part::Ring(400, 400, 220, 60),
            Part::Stroke(400, 80, 400, 720, 50),
            Part::Stroke(80, 400, 720, 400, 50),
        ],
    ),
    (
        '↧',
        800,
        &[
            Part::Stroke(400, 720, 400, 180, 60),
            Part::Stroke(220, 340, 400, 140, 60),
            Part::Stroke(580, 340, 400, 140, 60),
        ],
    ),
    (
        '⌴',
        800,
        &[
            Part::Stroke(160, 200, 160, 640, 60),
            Part::Stroke(160, 640, 640, 640, 60),
            Part::Stroke(640, 640, 640, 200, 60),
        ],
    ),
    (
        '⌵',
        800,
        &[
            Part::Stroke(160, 640, 400, 180, 60),
            Part::Stroke(640, 640, 400, 180, 60),
        ],
    ),
    (
        '∥',
        800,
        &[
            Part::Stroke(280, 140, 280, 720, 70),
            Part::Stroke(520, 140, 520, 720, 70),
        ],
    ),
    (
        '∠',
        800,
        &[
            Part::Stroke(180, 180, 180, 680, 60),
            Part::Stroke(180, 180, 680, 180, 60),
        ],
    ),
    ('○', 800, &[Part::Ring(400, 400, 250, 70)]),
    (
        '◎',
        800,
        &[
            Part::Ring(400, 400, 280, 50),
            Part::Ring(400, 400, 120, 50),
        ],
    ),
    (
        '⌭',
        800,
        &[
            Part::Ring(400, 400, 220, 50),
            Part::Stroke(250, 160, 250, 640, 40),
            Part::Stroke(550, 160, 550, 640, 40),
        ],
    ),
    (
        '▱',
        800,
        &[
            Part::Stroke(220, 180, 620, 180, 50),
            Part::Stroke(620, 180, 760, 680, 50),
            Part::Stroke(760, 680, 360, 680, 50),
            Part::Stroke(360, 680, 220, 180, 50),
        ],
    ),
    (
        '⌒',
        800,
        &[
            Part::Stroke(140, 260, 280, 560, 50),
            Part::Stroke(280, 560, 520, 560, 50),
            Part::Stroke(520, 560, 660, 260, 50),
        ],
    ),
    (
        '⌓',
        800,
        &[
            Part::Stroke(160, 220, 300, 560, 50),
            Part::Stroke(300, 560, 500, 560, 50),
            Part::Stroke(500, 560, 640, 220, 50),
            Part::Stroke(200, 300, 600, 300, 40),
        ],
    ),
    (
        '⌯',
        800,
        &[
            Part::Stroke(160, 260, 640, 260, 50),
            Part::Stroke(160, 540, 640, 540, 50),
            Part::Stroke(400, 260, 400, 540, 50),
        ],
    ),
    (
        '↗',
        800,
        &[
            Part::Stroke(180, 180, 640, 640, 60),
            Part::Stroke(360, 640, 660, 640, 60),
            Part::Stroke(640, 360, 640, 660, 60),
        ],
    ),
    (
        '—',
        800,
        &[Part::Stroke(80, 400, 720, 400, 70)],
    ),
    (
        'Ⓜ',
        900,
        &[
            Part::Ring(400, 400, 280, 60),
            Part::Stroke(250, 220, 250, 560, 50),
            Part::Stroke(400, 220, 400, 560, 50),
            Part::Stroke(250, 560, 400, 380, 50),
        ],
    ),
    (
        'Ⓛ',
        900,
        &[
            Part::Ring(400, 400, 280, 60),
            Part::Stroke(300, 220, 300, 580, 55),
            Part::Stroke(300, 220, 520, 220, 55),
        ],
    ),
    (
        'Ⓢ',
        900,
        &[
            Part::Ring(400, 400, 280, 60),
            Part::Stroke(250, 520, 550, 520, 50),
            Part::Stroke(250, 400, 550, 400, 50),
            Part::Stroke(250, 280, 550, 280, 50),
        ],
    ),
    (
        'ω',
        800,
        &[
            Part::Stroke(140, 560, 220, 200, 50),
            Part::Stroke(220, 200, 400, 480, 50),
            Part::Stroke(400, 480, 580, 200, 50),
            Part::Stroke(580, 200, 680, 560, 50),
        ],
    ),
    (
        '件',
        1000,
        &[
            Part::Stroke(150, 80, 150, 760, 70),
            Part::Stroke(70, 620, 230, 500, 55),
            Part::Stroke(60, 360, 250, 360, 55),
            Part::Stroke(520, 60, 520, 800, 70),
            Part::Stroke(300, 640, 840, 640, 55),
            Part::Stroke(280, 420, 860, 420, 55),
            Part::Stroke(360, 420, 360, 120, 55),
        ],
    ),
    (
        '零',
        1000,
        &[
            Part::Stroke(140, 820, 860, 820, 50),
            Part::Stroke(240, 760, 240, 680, 40),
            Part::Stroke(500, 780, 500, 680, 40),
            Part::Stroke(760, 760, 760, 680, 40),
            Part::Stroke(180, 600, 820, 600, 50),
            Part::Stroke(180, 220, 180, 600, 50),
            Part::Stroke(820, 220, 820, 600, 50),
            Part::Stroke(180, 220, 820, 220, 50),
            Part::Stroke(320, 500, 680, 500, 40),
            Part::Stroke(420, 500, 420, 300, 40),
            Part::Stroke(520, 400, 700, 300, 40),
        ],
    ),
    (
        '加',
        1000,
        &[
            Part::Stroke(140, 160, 140, 700, 60),
            Part::Stroke(140, 420, 340, 200, 55),
            Part::Stroke(480, 180, 480, 700, 55),
            Part::Stroke(480, 700, 840, 700, 55),
            Part::Stroke(840, 180, 840, 700, 55),
            Part::Stroke(480, 180, 840, 180, 55),
        ],
    ),
    (
        '工',
        1000,
        &[
            Part::Stroke(160, 760, 840, 760, 70),
            Part::Stroke(500, 200, 500, 760, 70),
            Part::Stroke(140, 180, 860, 180, 70),
        ],
    ),
];
