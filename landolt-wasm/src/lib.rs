use wasm_bindgen::prelude::*;

/// Draw a Landolt C ring and apply a Gaussian PSF to simulate the retinal image.
///
/// # Parameters
/// - `width` / `height`: canvas dimensions in pixels
/// - `cx` / `cy`: centre of the ring in pixels
/// - `outer_r`: outer radius in pixels
/// - `ring_width`: width (thickness) of the ring in pixels
/// - `gap_deg`: angular width of the gap in degrees
/// - `rotation_deg`: rotation of the gap opening, measured clockwise from 12 o'clock (top)
/// - `sigma`: standard deviation of the Gaussian PSF in pixels
///             (0.0 = no blur, larger = more blur / lower visual acuity)
/// - `bg`: background luminance 0–255 (default white = 255)
/// - `fg`: ring luminance 0–255 (default black = 0)
///
/// Returns an RGBA `Uint8Array` with `width * height * 4` bytes.
#[wasm_bindgen]
pub fn render_landolt(
    width: u32,
    height: u32,
    cx: f64,
    cy: f64,
    outer_r: f64,
    ring_width: f64,
    gap_deg: f64,
    rotation_deg: f64,
    sigma: f64,
    bg: u8,
    fg: u8,
) -> js_sys::Uint8Array {
    let w = width as usize;
    let h = height as usize;
    let n = w * h;

    // --- 1. Draw binary Landolt C (float image, 0.0 = fg, 1.0 = bg) --------
    // We work in linear float to make convolution easier.
    let bg_f = bg as f64 / 255.0;
    let fg_f = fg as f64 / 255.0;

    let mut img: Vec<f64> = vec![bg_f; n];

    let inner_r = outer_r - ring_width;
    // Convert gap half-angle to radians
    let gap_half = (gap_deg / 2.0).to_radians();
    // rotation_deg is clockwise from top; convert to math angle:
    // top = 90° in math coords (counter-clockwise from right)
    let rot = std::f64::consts::FRAC_PI_2 - rotation_deg.to_radians();

    for row in 0..h {
        for col in 0..w {
            let dx = col as f64 + 0.5 - cx;
            let dy = cy - (row as f64 + 0.5); // flip y for math coords
            let r2 = dx * dx + dy * dy;
            let ir2 = inner_r * inner_r;
            let or2 = outer_r * outer_r;

            if r2 >= ir2 && r2 <= or2 {
                // Inside the ring annulus – check for gap
                let angle = dy.atan2(dx); // math angle from centre
                let diff = angle_diff(angle, rot);
                if diff.abs() > gap_half {
                    img[row * w + col] = fg_f;
                }
            }
        }
    }

    // --- 2. Apply Gaussian PSF if sigma > 0 ---------------------------------
    let result = if sigma > 0.5 {
        gaussian_blur(&img, w, h, sigma)
    } else {
        img
    };

    // --- 3. Pack into RGBA Uint8Array ----------------------------------------
    let mut rgba = vec![0u8; n * 4];
    for i in 0..n {
        let v = (result[i].clamp(0.0, 1.0) * 255.0).round() as u8;
        rgba[i * 4] = v;
        rgba[i * 4 + 1] = v;
        rgba[i * 4 + 2] = v;
        rgba[i * 4 + 3] = 255;
    }

    let out = js_sys::Uint8Array::new_with_length((n * 4) as u32);
    out.copy_from(&rgba);
    out
}

/// Smallest signed difference between two angles (result in [-π, π]).
fn angle_diff(a: f64, b: f64) -> f64 {
    let d = a - b;
    let pi = std::f64::consts::PI;
    let tau = 2.0 * pi;
    // Normalise to [-π, π]
    ((d + pi).rem_euclid(tau)) - pi
}

/// Separable Gaussian blur (two-pass 1-D convolution).
fn gaussian_blur(img: &[f64], w: usize, h: usize, sigma: f64) -> Vec<f64> {
    let kernel = gaussian_kernel(sigma);
    let k = kernel.len();
    let half = (k / 2) as isize;

    // Horizontal pass
    let mut tmp = vec![0.0f64; w * h];
    for row in 0..h {
        for col in 0..w {
            let mut sum = 0.0f64;
            let mut wsum = 0.0f64;
            for ki in 0..k {
                let src_col = col as isize + ki as isize - half;
                if src_col >= 0 && src_col < w as isize {
                    sum += img[row * w + src_col as usize] * kernel[ki];
                    wsum += kernel[ki];
                }
            }
            tmp[row * w + col] = sum / wsum;
        }
    }

    // Vertical pass
    let mut out = vec![0.0f64; w * h];
    for row in 0..h {
        for col in 0..w {
            let mut sum = 0.0f64;
            let mut wsum = 0.0f64;
            for ki in 0..k {
                let src_row = row as isize + ki as isize - half;
                if src_row >= 0 && src_row < h as isize {
                    sum += tmp[src_row as usize * w + col] * kernel[ki];
                    wsum += kernel[ki];
                }
            }
            out[row * w + col] = sum / wsum;
        }
    }
    out
}

/// Build a 1-D Gaussian kernel truncated at ±3σ.
fn gaussian_kernel(sigma: f64) -> Vec<f64> {
    let radius = (3.0 * sigma).ceil() as usize;
    let size = 2 * radius + 1;
    let mut k = Vec::with_capacity(size);
    let inv2s2 = 1.0 / (2.0 * sigma * sigma);
    for i in 0..size {
        let x = i as f64 - radius as f64;
        k.push((-x * x * inv2s2).exp());
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_angle_diff_zero() {
        assert!((angle_diff(1.0, 1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_angle_diff_wrap() {
        use std::f64::consts::PI;
        // Angles just across the ±π boundary should give small difference
        let d = angle_diff(PI - 0.1, -PI + 0.1);
        assert!(d.abs() < 0.21 + 1e-10);
    }

    #[test]
    fn test_gaussian_kernel_sum() {
        let k = gaussian_kernel(2.0);
        let s: f64 = k.iter().sum();
        // Sum should be positive
        assert!(s > 0.0);
    }
}
