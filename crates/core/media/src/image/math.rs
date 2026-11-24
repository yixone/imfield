#![allow(clippy::needless_range_loop)]

// TODO: For optimization, add precomputations

use std::f32::consts::PI;

/// Computes the 2D DCT-II
///
/// Returns the unnormalized DCT coefficents.
/// The output contains `nx * ny` coefficents
pub(crate) fn dct2(b: &[f32], w: usize, h: usize, nx: usize, ny: usize) -> Vec<f32> {
    let mut dct = Vec::with_capacity(nx * ny);

    let mut fx = [0.0].repeat(w);

    // TODO: Optimize basis computations

    for cy in 0..ny {
        for cx in 0..nx {
            for x in 0..w {
                fx[x] = basis(w as f32, x as f32, cx as f32);
            }
            let mut f = 0.0;
            for y in 0..h {
                let fy = basis(h as f32, y as f32, cy as f32);
                for x in 0..w {
                    f += b[x + y * w] * fx[x] * fy;
                }
            }

            dct.push(f);
        }
    }

    dct
}

/// Computes the orthonormal 2D DCT-II
///
/// Applies orthonormal normalization to the coefficents returned by [`dct2`]
pub(crate) fn dct2_ortho(b: &[f32], w: usize, h: usize, nx: usize, ny: usize) -> Vec<f32> {
    let mut dct = dct2(b, w, h, nx, ny);

    for cy in 0..ny {
        for cx in 0..nx {
            let alpha_w = alpha(cx, w as f32);
            let alpha_h = alpha(cy, h as f32);

            dct[cx + cy * nx] *= alpha_w * alpha_h;
        }
    }

    dct
}

///
pub(crate) fn dct2_thumbhash(
    b: &[f32],
    w: usize,
    h: usize,
    nx: usize,
    ny: usize,
) -> (Vec<f32>, f32) {
    let dct = dct2(b, w, h, nx, ny);

    let mut scale = 0.0;
    let mut scaled = Vec::with_capacity(nx * ny);

    for cy in 0..ny {
        let mut cx = 0;
        while cx * ny < nx * (ny - cy) {
            let f = dct[cx + cy * nx] / (w * h) as f32;

            scaled.push(f);
            if cx > 0 || cy > 0 {
                scale = f.abs().max(scale);
            }

            cx += 1;
        }
    }

    if scale > 0.0 {
        for (idx, i) in scaled.iter_mut().enumerate() {
            if idx == 0 {
                continue;
            }
            *i = 0.5 + 0.5 / scale * *i;
        }
    }

    (scaled, scale)
}

fn basis(n_n: f32, n: f32, k: f32) -> f32 {
    ((PI / n_n) * (n + 0.5) * k).cos()
}

fn alpha(k: usize, n: f32) -> f32 {
    if k == 0 {
        (1f32 / n).sqrt()
    } else {
        (2f32 / n).sqrt()
    }
}
