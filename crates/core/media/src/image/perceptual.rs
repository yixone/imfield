#![allow(clippy::needless_range_loop)]

use imfield_models::types::media::PerceptualHash;

mod dct {
    use std::{f32::consts::PI, sync::LazyLock};

    /// Dimensionality of DCT matrices
    const N: usize = 32;

    /// LazyLock Basis for DCT
    static BASIS: LazyLock<[[f32; N]; N]> = LazyLock::new(|| {
        let mut b = [[0.0; N]; N];
        for i in 0..N {
            for n in 0..N {
                let basis = (((2 * n + 1) as f32) * i as f32 * PI) / (2 * N) as f32;
                b[i][n] = basis.cos();
            }
        }
        b
    });

    /// LazyLock Alpha for DCT
    static ALPHA: LazyLock<[f32; N]> = LazyLock::new(|| {
        let mut a = [0f32; N];
        for i in 0..N {
            let alpha = if i == 0 {
                (1f32 / N as f32).sqrt()
            } else {
                (2f32 / N as f32).sqrt()
            };
            a[i] = alpha;
        }
        a
    });

    /// Applies the DCT-II (Discrete Cosine Transform II) to the matrix
    ///
    /// Read more here:
    /// [Discrete_Cosine_Transform](https://en.wikipedia.org/wiki/Discrete_cosine_transform#DCT-II)
    pub fn apply_dct2(matrix: &[[u8; N]; N]) -> [[f32; N]; N] {
        let mut dct = [[0f32; N]; N];
        for row in 0..N {
            dct[row] = apply_dct2_1d(&matrix[row].map(|i| (i as f32) / 255.0));
        }
        for col in 0..N {
            let mut tmp = [0f32; N];
            for row in 0..N {
                tmp[row] = dct[row][col];
            }
            let tmp = apply_dct2_1d(&tmp);
            for row in 0..N {
                dct[row][col] = tmp[row];
            }
        }
        dct
    }

    /// Applies the DCT-II transform to a one-dimensional row
    fn apply_dct2_1d(ik: &[f32; N]) -> [f32; N] {
        let mut dct = [0f32; N];
        for i in 0..N {
            let mut sum = 0f32;
            for n in 0..N {
                let sig = ik[n] * BASIS[i][n];
                sum += sig;
            }
            dct[i] = ALPHA[i] * sum;
        }
        dct
    }
}

/// Calculates a perceptual hash for a 32x32 matrix
pub(crate) fn p_hash(b: [[u8; 32]; 32]) -> PerceptualHash {
    // Apply DCT
    let dct = dct::apply_dct2(&b);

    // Crop the matrix to 8x8
    let mut vals = Vec::with_capacity(63);
    for row in 0..8 {
        for col in 0..8 {
            if col == 0 && row == 0 {
                continue;
            }
            vals.push(dct[row][col]);
        }
    }

    // Calculate the median value
    let median = {
        let mut sorted = vals.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("Cannot sort pHash values"));
        sorted[sorted.len() / 2]
    };

    // Calculating the hash
    let mut hash = 0_i64;
    for v in vals {
        hash <<= 1;
        if v > median {
            hash |= 1;
        }
    }

    PerceptualHash::new(hash)
}

/// Calculates the aHash for 64 image pixels (8x8)
pub(crate) fn a_hash(b: [u8; 64]) -> PerceptualHash {
    let avg = b.iter().map(|p| *p as u32).sum::<u32>() / b.len() as u32;
    let mut hash = 0_i64;
    for (idx, p) in b.into_iter().enumerate() {
        if p as u32 > avg {
            hash |= 1 << idx;
        }
    }

    PerceptualHash::new(hash)
}
