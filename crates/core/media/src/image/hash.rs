use image::Rgba;
use imfield_models::types::media::{PerceptualHash, ThumbHash};

use crate::image::math::{dct2_ortho, dct2_thumbhash};

/// Computes a perceptual hash using the DCT-based pHash algorithm
pub(crate) fn p_hash(b: [[u8; 32]; 32]) -> PerceptualHash {
    // Apply DCT to input
    let dct = dct2_ortho(
        &b.as_flattened()
            .iter()
            .map(|b| *b as f32 / 255.0)
            .collect::<Vec<_>>(),
        32,
        32,
        8,
        8,
    );

    // Calculate the median value
    // Takes all elements except DC for the calculation
    let mut median = dct.iter().copied().skip(1).collect::<Vec<_>>();
    median.sort_by(|a, b| {
        a.partial_cmp(b)
            .expect("pHash matrix values must be sorted")
    });
    let median = median[median.len() / 2];

    // Calculate the hash
    let mut hash = 0;
    for r in 0..8 {
        for c in 0..8 {
            // Skip DC
            if r == 0 && c == 0 {
                continue;
            }

            hash <<= 1;
            if dct[c + r * 8] > median {
                hash |= 1;
            }
        }
    }

    PerceptualHash::new(hash)
}

/// Computes an average hash using the aHash algorithm
pub(crate) fn a_hash(b: [[u8; 8]; 8]) -> PerceptualHash {
    // Find the average value
    let pixels = b.as_flattened();
    let avg = pixels.iter().map(|p| *p as u32).sum::<u32>() / pixels.len() as u32;

    // Calculate the hash
    let mut hash = 0;
    for p in pixels {
        hash <<= 1;

        if *p as u32 > avg {
            hash |= 1;
        }
    }

    PerceptualHash::new(hash)
}

/// Generates a ThumbHash representation of an RGBA image
pub(crate) fn thumbhash(rgba: &[Rgba<u8>], w: usize, h: usize) -> ThumbHash {
    // Determine the average color
    let mut avg_r = 0.0;
    let mut avg_g = 0.0;
    let mut avg_b = 0.0;
    let mut avg_a = 0.0;

    for [r, g, b, a] in rgba.iter().map(|r| r.0) {
        let alpha = a as f32 / 255.0;
        let alpha_n = alpha / 255.0;
        avg_r += alpha_n * r as f32;
        avg_g += alpha_n * g as f32;
        avg_b += alpha_n * b as f32;
        avg_a += alpha;
    }

    if avg_a > 0.0 {
        avg_r /= avg_a;
        avg_g /= avg_a;
        avg_b /= avg_a;
    }

    // Convert the image from RGBA to LPQA (composite atop the average color)
    let has_alpha = avg_a < (w * h) as f32;
    let l_limit = if has_alpha { 5 } else { 7 };

    let mut l = Vec::with_capacity(w * h);
    let mut p = Vec::with_capacity(w * h);
    let mut q = Vec::with_capacity(w * h);
    let mut a = Vec::with_capacity(w * h);

    for [pr, pg, pb, pa] in rgba.iter().map(|r| r.0) {
        let alpha = pa as f32 / 255.0;

        let alpha_d = 1.0 - alpha;
        let alpha_n = alpha / 255.0;

        let r = avg_r * alpha_d + alpha_n * pr as f32;
        let g = avg_g * alpha_d + alpha_n * pg as f32;
        let b = avg_b * alpha_d + alpha_n * pb as f32;

        l.push((r + g + b) / 3.0);
        p.push((r + g) / 2.0 - b);
        q.push(r - g);
        a.push(alpha);
    }

    // Encode using the DCT
    let lx = (((l_limit * w) as f32 / w.max(h) as f32).round() as usize).max(1);
    let ly = (((l_limit * h) as f32 / w.max(h) as f32).round() as usize).max(1);

    let (l_dct, l_scale) = dct2_thumbhash(&l, w, h, lx.max(3), ly.max(3));
    let (p_dct, p_scale) = dct2_thumbhash(&p, w, h, 3, 3);
    let (q_dct, q_scale) = dct2_thumbhash(&q, w, h, 3, 3);
    let (a_dct, a_scale) = if has_alpha {
        dct2_thumbhash(&a, w, h, 5, 5)
    } else {
        (vec![1.0], 1.0)
    };

    let (l_dc, l_ac) = (l_dct[0], &l_dct[1..]);
    let (p_dc, p_ac) = (p_dct[0], &p_dct[1..]);
    let (q_dc, q_ac) = (q_dct[0], &q_dct[1..]);
    let (a_dc, a_ac) = (a_dct[0], &a_dct[1..]);

    // Write the constants
    let mut hash = Vec::with_capacity(25);
    let is_landscape = w > h;

    let h24 = (63.0 * l_dc).round() as u32
        | (((31.5 + 31.5 * p_dc).round() as u32) << 6)
        | (((31.5 + 31.5 * q_dc).round() as u32) << 12)
        | (((31.0 * l_scale).round() as u32) << 18)
        | if has_alpha { 1 << 23 } else { 0 };

    let h16 = (if is_landscape { ly } else { lx }) as u16
        | (((63.0 * p_scale).round() as u16) << 3)
        | (((63.0 * q_scale).round() as u16) << 9)
        | if is_landscape { 1 << 15 } else { 0 };

    hash.extend_from_slice(&[
        (h24 & 255) as u8,
        ((h24 >> 8) & 255) as u8,
        (h24 >> 16) as u8,
        (h16 & 255) as u8,
        (h16 >> 8) as u8,
    ]);

    if has_alpha {
        hash.push((15.0 * a_dc).round() as u8 | (((15.0 * a_scale).round() as u8) << 4));
    }

    // Write the varying factors
    let mut is_odd = false;

    let mut apply_ac = |ac: &[f32]| {
        for f in ac {
            let u = (15.0 * f).round() as u8;
            if is_odd {
                *hash.last_mut().expect("The hash must have a last element") |= u << 4;
            } else {
                hash.push(u);
            }
            is_odd = !is_odd;
        }
    };

    for ac in [l_ac, p_ac, q_ac] {
        apply_ac(ac);
    }

    if has_alpha {
        apply_ac(a_ac);
    }

    ThumbHash::new(hash)
}
