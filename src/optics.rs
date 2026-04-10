use rustfft::{num_complex::Complex64, FftPlanner};
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScaPrescription {
    pub sphere_diopters: f64,
    pub cylinder_diopters: f64,
    pub axis_degrees: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZernikeMode {
    pub n: u32,
    pub m: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZernikeCoefficient {
    pub mode: ZernikeMode,
    pub coefficient_um: f64,
    pub source: CoefficientSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoefficientSource {
    Sca,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StylesCrawfordConfig {
    pub enabled: bool,
    pub rho: f64,
}

impl Default for StylesCrawfordConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            rho: 0.12,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WavefrontRequest {
    pub pupil_samples: usize,
    pub pupil_diameter_mm: f64,
    pub prescription: Option<ScaPrescription>,
    #[serde(default)]
    pub additional_coefficients: Vec<ZernikeCoefficient>,
    #[serde(default)]
    pub styles_crawford: StylesCrawfordConfig,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WavefrontResult {
    pub width: usize,
    pub height: usize,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub pupil_mask: Vec<f64>,
    pub amplitude: Vec<f64>,
    pub wavefront_um: Vec<f64>,
    pub min_um: f64,
    pub max_um: f64,
    pub coefficients: Vec<ZernikeCoefficient>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpticalConfig {
    pub wavelength_nm: f64,
    pub pupil_diameter_mm: f64,
    pub image_samples: usize,
    pub target_fov_arcmin: f64,
    pub pupil_samples: Option<usize>,
}

impl Default for OpticalConfig {
    fn default() -> Self {
        Self {
            wavelength_nm: 555.0,
            pupil_diameter_mm: 6.0,
            image_samples: 2048,
            target_fov_arcmin: 240.0,
            pupil_samples: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PsfResult {
    pub width: usize,
    pub height: usize,
    pub data: Vec<f64>,
    pub delta_theta_arcmin: f64,
    pub fov_arcmin: f64,
    pub min_value: f64,
    pub max_value: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LandoltRingPlacement {
    pub x_arcmin: f64,
    pub y_arcmin: f64,
    pub outer_radius_arcmin: f64,
    pub acuity: f64,
    pub gap_angle_degrees: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LandoltChartResult {
    pub width: usize,
    pub height: usize,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub data: Vec<f64>,
    pub placements: Vec<LandoltRingPlacement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetinalSimulationRequest {
    #[serde(default)]
    pub optics: OpticalConfig,
    pub prescription: Option<ScaPrescription>,
    #[serde(default)]
    pub additional_coefficients: Vec<ZernikeCoefficient>,
    #[serde(default)]
    pub styles_crawford: StylesCrawfordConfig,
    #[serde(default = "default_acuities")]
    pub acuities: Vec<f64>,
    #[serde(default = "default_grid_size")]
    pub grid_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetinalSimulationResult {
    pub optics: OpticalConfig,
    pub pupil_samples: usize,
    pub wavefront: WavefrontResult,
    pub psf: PsfResult,
    pub chart: LandoltChartResult,
    pub retinal_image: Vec<f64>,
}

pub fn sca_to_zernike(
    prescription: ScaPrescription,
    pupil_diameter_mm: f64,
) -> [ZernikeCoefficient; 3] {
    let radius_mm = pupil_diameter_mm / 2.0;
    let axis_radians = prescription.axis_degrees.to_radians();
    let mean_spherical_equivalent =
        prescription.sphere_diopters + prescription.cylinder_diopters / 2.0;

    let c2_0 = -(mean_spherical_equivalent * radius_mm.powi(2)) / (4.0 * 3.0_f64.sqrt());
    let c2_m2 = -(prescription.cylinder_diopters
        * radius_mm.powi(2)
        * (2.0 * axis_radians).sin())
        / (4.0 * 6.0_f64.sqrt());
    let c2_p2 = -(prescription.cylinder_diopters
        * radius_mm.powi(2)
        * (2.0 * axis_radians).cos())
        / (4.0 * 6.0_f64.sqrt());

    [
        ZernikeCoefficient {
            mode: ZernikeMode { n: 2, m: -2 },
            coefficient_um: c2_m2,
            source: CoefficientSource::Sca,
        },
        ZernikeCoefficient {
            mode: ZernikeMode { n: 2, m: 0 },
            coefficient_um: c2_0,
            source: CoefficientSource::Sca,
        },
        ZernikeCoefficient {
            mode: ZernikeMode { n: 2, m: 2 },
            coefficient_um: c2_p2,
            source: CoefficientSource::Sca,
        },
    ]
}

pub fn zernike_radial(n: u32, m: i32, r: f64) -> f64 {
    let abs_m = m.unsigned_abs();

    if r > 1.0 || abs_m > n || (n - abs_m) % 2 != 0 {
        return 0.0;
    }

    let mut radial = 0.0;
    let max_s = (n - abs_m) / 2;

    for s in 0..=max_s {
        let numerator = if s % 2 == 0 { 1.0 } else { -1.0 } * factorial(n - s);
        let denominator = factorial(s)
            * factorial((n + abs_m) / 2 - s)
            * factorial((n - abs_m) / 2 - s);
        radial += (numerator / denominator) * r.powi((n - 2 * s) as i32);
    }

    radial
}

pub fn zernike_value(n: u32, m: i32, r: f64, theta: f64) -> f64 {
    if r > 1.0 {
        return 0.0;
    }

    let normalization = if m == 0 {
        (f64::from(n + 1)).sqrt()
    } else {
        (2.0 * f64::from(n + 1)).sqrt()
    };
    let radial = zernike_radial(n, m, r);

    if m < 0 {
        -normalization * radial * (f64::from(m.abs()) * theta).sin()
    } else {
        normalization * radial * (f64::from(m) * theta).cos()
    }
}

pub fn styles_crawford_weight(normalized_radius: f64, config: StylesCrawfordConfig) -> f64 {
    if normalized_radius > 1.0 {
        return 0.0;
    }

    if !config.enabled {
        return 1.0;
    }

    10.0_f64.powf(-config.rho * normalized_radius.powi(2))
}

pub fn derive_pupil_samples(
    wavelength_nm: f64,
    pupil_diameter_mm: f64,
    target_fov_arcmin: f64,
) -> usize {
    let fov_radians = arcmin_to_radians(target_fov_arcmin);
    let wavelength_mm = wavelength_nm * 1e-6;
    let samples = (fov_radians * pupil_diameter_mm / wavelength_mm).floor();
    samples.max(2.0) as usize
}

pub fn image_plane_sampling(
    wavelength_nm: f64,
    pupil_diameter_mm: f64,
    pupil_samples: usize,
    image_samples: usize,
) -> (f64, f64) {
    let delta_theta_rad =
        (wavelength_nm * 1e-6 * pupil_samples as f64) / (image_samples as f64 * pupil_diameter_mm);
    let delta_theta_arcmin = radians_to_arcmin(delta_theta_rad);
    let fov_arcmin = image_samples as f64 * delta_theta_arcmin;
    (delta_theta_arcmin, fov_arcmin)
}

pub fn build_wavefront(request: &WavefrontRequest) -> WavefrontResult {
    let mut coefficients = Vec::new();

    if let Some(prescription) = request.prescription {
        coefficients.extend(sca_to_zernike(prescription, request.pupil_diameter_mm));
    }

    coefficients.extend(request.additional_coefficients.iter().cloned());

    let samples = request.pupil_samples.max(2);
    let axis = linspace(samples, -1.0, 1.0);
    let mut pupil_mask = Vec::with_capacity(samples * samples);
    let mut amplitude = Vec::with_capacity(samples * samples);
    let mut wavefront_um = Vec::with_capacity(samples * samples);
    let mut min_um = f64::INFINITY;
    let mut max_um = f64::NEG_INFINITY;

    for y in &axis {
        for x in &axis {
            let radius = (x * x + y * y).sqrt();
            let inside_pupil = radius <= 1.0;
            let mask_value = if inside_pupil { 1.0 } else { 0.0 };
            let theta = y.atan2(*x);
            let mut value_um = 0.0;

            if inside_pupil {
                for coefficient in &coefficients {
                    value_um += coefficient.coefficient_um
                        * zernike_value(coefficient.mode.n, coefficient.mode.m, radius, theta);
                }
            }

            let amplitude_value = if inside_pupil {
                styles_crawford_weight(radius, request.styles_crawford)
            } else {
                0.0
            };

            min_um = min_um.min(value_um);
            max_um = max_um.max(value_um);
            pupil_mask.push(mask_value);
            amplitude.push(amplitude_value);
            wavefront_um.push(value_um * mask_value);
        }
    }

    if !min_um.is_finite() || !max_um.is_finite() {
        min_um = 0.0;
        max_um = 0.0;
    }

    WavefrontResult {
        width: samples,
        height: samples,
        x: axis.clone(),
        y: axis,
        pupil_mask,
        amplitude,
        wavefront_um,
        min_um,
        max_um,
        coefficients,
    }
}

pub fn build_pupil_function(wavefront: &WavefrontResult, wavelength_nm: f64) -> Vec<Complex64> {
    wavefront
        .wavefront_um
        .iter()
        .zip(wavefront.amplitude.iter())
        .map(|(wavefront_um, amplitude)| {
            let phase = wavefront_phase_radians(wavelength_nm, *wavefront_um);
            Complex64::new(amplitude * phase.cos(), amplitude * phase.sin())
        })
        .collect()
}

pub fn compute_psf(
    wavefront: &WavefrontResult,
    wavelength_nm: f64,
    image_samples: usize,
    pupil_diameter_mm: f64,
) -> Result<PsfResult, String> {
    if image_samples < wavefront.width || image_samples < wavefront.height {
        return Err("image_samples must be >= pupil_samples".to_string());
    }

    let pupil_function = build_pupil_function(wavefront, wavelength_nm);
    let padded = center_pad_complex(&pupil_function, wavefront.width, wavefront.height, image_samples, image_samples)?;
    let shifted_input = fftshift_complex(&padded, image_samples, image_samples);
    let psf_amplitude = fftshift_complex(&fft2(&shifted_input, image_samples, image_samples, false), image_samples, image_samples);

    let mut data: Vec<f64> = psf_amplitude.iter().map(|value| value.norm_sqr()).collect();
    let total_energy: f64 = data.iter().sum();
    if total_energy > 0.0 {
        for value in &mut data {
            *value /= total_energy;
        }
    }

    let min_value = data.iter().copied().fold(f64::INFINITY, f64::min);
    let max_value = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let (delta_theta_arcmin, fov_arcmin) = image_plane_sampling(
        wavelength_nm,
        pupil_diameter_mm,
        wavefront.width,
        image_samples,
    );

    Ok(PsfResult {
        width: image_samples,
        height: image_samples,
        data,
        delta_theta_arcmin,
        fov_arcmin,
        min_value: if min_value.is_finite() { min_value } else { 0.0 },
        max_value: if max_value.is_finite() { max_value } else { 0.0 },
    })
}

pub fn create_landolt_chart_grid(
    acuities: &[f64],
    fov_arcmin: f64,
    grid_size: usize,
    image_samples: usize,
) -> LandoltChartResult {
    let axis = linspace(image_samples, -fov_arcmin / 2.0, fov_arcmin / 2.0);
    let mut data = vec![1.0; image_samples * image_samples];
    let mut placements = Vec::new();
    let mut sorted_acuities = acuities.to_vec();
    sorted_acuities.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));

    let cell_arcmin = fov_arcmin / grid_size as f64;
    let gap_angles = [0.0, 90.0, 180.0, 270.0];

    for (index, acuity) in sorted_acuities.iter().copied().enumerate() {
        let row = index / grid_size;
        let column = index % grid_size;
        let x_center = -fov_arcmin / 2.0 + (column as f64 + 0.5) * cell_arcmin;
        let y_center = fov_arcmin / 2.0 - (row as f64 + 0.5) * cell_arcmin;
        let gap_arcmin = 1.0 / acuity;
        let outer_radius = gap_arcmin * 2.5;
        let inner_radius = gap_arcmin * 1.5;
        let gap_angle_degrees = gap_angles[index % gap_angles.len()];

        placements.push(LandoltRingPlacement {
            x_arcmin: x_center,
            y_arcmin: y_center,
            outer_radius_arcmin: outer_radius,
            acuity,
            gap_angle_degrees,
        });

        rasterize_landolt_ring(
            &mut data,
            image_samples,
            &axis,
            x_center,
            y_center,
            outer_radius,
            inner_radius,
            gap_arcmin,
            gap_angle_degrees,
        );
    }

    LandoltChartResult {
        width: image_samples,
        height: image_samples,
        x: axis.clone(),
        y: axis,
        data,
        placements,
    }
}

pub fn circular_convolution(
    image: &[f64],
    kernel: &[f64],
    width: usize,
    height: usize,
) -> Result<Vec<f64>, String> {
    if image.len() != width * height || kernel.len() != width * height {
        return Err("image and kernel dimensions must match width * height".to_string());
    }

    let shifted_kernel = ifftshift_real(kernel, width, height);
    let image_complex: Vec<Complex64> = image.iter().map(|value| Complex64::new(*value, 0.0)).collect();
    let kernel_complex: Vec<Complex64> = shifted_kernel
        .iter()
        .map(|value| Complex64::new(*value, 0.0))
        .collect();

    let image_fft = fft2(&image_complex, width, height, false);
    let kernel_fft = fft2(&kernel_complex, width, height, false);
    let multiplied: Vec<Complex64> = image_fft
        .iter()
        .zip(kernel_fft.iter())
        .map(|(left, right)| left * right)
        .collect();

    let spatial = fft2(&multiplied, width, height, true);
    Ok(spatial
        .iter()
        .map(|value| value.re.clamp(0.0, 1.0))
        .collect())
}

pub fn simulate_retinal_image(
    request: &RetinalSimulationRequest,
) -> Result<RetinalSimulationResult, String> {
    if request.optics.wavelength_nm <= 0.0 {
        return Err("wavelength_nm must be positive".to_string());
    }
    if request.optics.pupil_diameter_mm <= 0.0 {
        return Err("pupil_diameter_mm must be positive".to_string());
    }
    if request.optics.image_samples < 2 {
        return Err("image_samples must be at least 2".to_string());
    }
    if request.grid_size == 0 {
        return Err("grid_size must be at least 1".to_string());
    }
    if request.acuities.is_empty() {
        return Err("acuities must not be empty".to_string());
    }

    let derived_pupil_samples = request.optics.pupil_samples.unwrap_or_else(|| {
        derive_pupil_samples(
            request.optics.wavelength_nm,
            request.optics.pupil_diameter_mm,
            request.optics.target_fov_arcmin,
        )
    });
    let pupil_samples = derived_pupil_samples.min(request.optics.image_samples).max(2);

    let wavefront = build_wavefront(&WavefrontRequest {
        pupil_samples,
        pupil_diameter_mm: request.optics.pupil_diameter_mm,
        prescription: request.prescription,
        additional_coefficients: request.additional_coefficients.clone(),
        styles_crawford: request.styles_crawford,
    });
    let psf = compute_psf(
        &wavefront,
        request.optics.wavelength_nm,
        request.optics.image_samples,
        request.optics.pupil_diameter_mm,
    )?;
    let chart = create_landolt_chart_grid(
        &request.acuities,
        psf.fov_arcmin,
        request.grid_size,
        request.optics.image_samples,
    );
    let retinal_image = circular_convolution(&chart.data, &psf.data, chart.width, chart.height)?;

    Ok(RetinalSimulationResult {
        optics: request.optics.clone(),
        pupil_samples,
        wavefront,
        psf,
        chart,
        retinal_image,
    })
}

fn factorial(value: u32) -> f64 {
    if value == 0 {
        return 1.0;
    }

    (1..=value).fold(1.0, |accumulator, item| accumulator * f64::from(item))
}

fn linspace(samples: usize, start: f64, end: f64) -> Vec<f64> {
    if samples <= 1 {
        return vec![start];
    }

    let step = (end - start) / ((samples - 1) as f64);
    (0..samples)
        .map(|index| start + step * index as f64)
        .collect()
}

fn default_acuities() -> Vec<f64> {
    vec![0.1, 0.2, 0.3, 0.5, 0.7, 1.0, 1.2, 1.5, 2.0]
}

fn default_grid_size() -> usize {
    3
}

fn arcmin_to_radians(arcmin: f64) -> f64 {
    arcmin * PI / (180.0 * 60.0)
}

fn radians_to_arcmin(radians: f64) -> f64 {
    radians * 180.0 * 60.0 / PI
}

fn rasterize_landolt_ring(
    data: &mut [f64],
    image_samples: usize,
    axis: &[f64],
    x_center: f64,
    y_center: f64,
    outer_radius: f64,
    inner_radius: f64,
    gap_arcmin: f64,
    gap_angle_degrees: f64,
) {
    let gap_angle_radians = gap_angle_degrees.to_radians();
    let min_x = (x_center - outer_radius).max(*axis.first().unwrap_or(&x_center));
    let max_x = (x_center + outer_radius).min(*axis.last().unwrap_or(&x_center));
    let min_y = (y_center - outer_radius).max(*axis.first().unwrap_or(&y_center));
    let max_y = (y_center + outer_radius).min(*axis.last().unwrap_or(&y_center));
    let x_start = axis.partition_point(|value| *value < min_x).min(image_samples.saturating_sub(1));
    let x_end = axis.partition_point(|value| *value <= max_x).max(x_start + 1).min(image_samples);
    let y_start = axis.partition_point(|value| *value < min_y).min(image_samples.saturating_sub(1));
    let y_end = axis.partition_point(|value| *value <= max_y).max(y_start + 1).min(image_samples);

    for y_index in y_start..y_end {
        let y = axis[y_index];
        for x_index in x_start..x_end {
            let x = axis[x_index];
            let dx = x - x_center;
            let dy = y - y_center;
            let distance = (dx * dx + dy * dy).sqrt();
            if distance > outer_radius || distance < inner_radius {
                continue;
            }

            let local_x = dx * gap_angle_radians.cos() + dy * gap_angle_radians.sin();
            let local_y = -dx * gap_angle_radians.sin() + dy * gap_angle_radians.cos();
            let is_gap = local_x > 0.0 && local_y.abs() <= gap_arcmin / 2.0;
            if !is_gap {
                data[y_index * image_samples + x_index] = 0.0;
            }
        }
    }
}

fn center_pad_complex(
    input: &[Complex64],
    input_width: usize,
    input_height: usize,
    output_width: usize,
    output_height: usize,
) -> Result<Vec<Complex64>, String> {
    if output_width < input_width || output_height < input_height {
        return Err("output dimensions must be >= input dimensions".to_string());
    }

    let mut output = vec![Complex64::new(0.0, 0.0); output_width * output_height];
    let pad_x = (output_width - input_width) / 2;
    let pad_y = (output_height - input_height) / 2;

    for y in 0..input_height {
        for x in 0..input_width {
            output[(y + pad_y) * output_width + (x + pad_x)] = input[y * input_width + x];
        }
    }

    Ok(output)
}

fn fft2(input: &[Complex64], width: usize, height: usize, inverse: bool) -> Vec<Complex64> {
    let mut planner = FftPlanner::<f64>::new();
    let row_fft = if inverse {
        planner.plan_fft_inverse(width)
    } else {
        planner.plan_fft_forward(width)
    };
    let column_fft = if inverse {
        planner.plan_fft_inverse(height)
    } else {
        planner.plan_fft_forward(height)
    };

    let mut output = input.to_vec();
    for row in output.chunks_mut(width) {
        row_fft.process(row);
    }

    let mut column = vec![Complex64::new(0.0, 0.0); height];
    for x in 0..width {
        for y in 0..height {
            column[y] = output[y * width + x];
        }
        column_fft.process(&mut column);
        for y in 0..height {
            output[y * width + x] = column[y];
        }
    }

    if inverse {
        let scale = (width * height) as f64;
        for value in &mut output {
            *value /= scale;
        }
    }

    output
}

fn fftshift_complex(input: &[Complex64], width: usize, height: usize) -> Vec<Complex64> {
    shift_complex(input, width, height, width / 2, height / 2)
}

fn ifftshift_real(input: &[f64], width: usize, height: usize) -> Vec<f64> {
    shift_real(input, width, height, width.div_ceil(2), height.div_ceil(2))
}

fn shift_complex(
    input: &[Complex64],
    width: usize,
    height: usize,
    x_shift: usize,
    y_shift: usize,
) -> Vec<Complex64> {
    let mut output = vec![Complex64::new(0.0, 0.0); width * height];
    for y in 0..height {
        for x in 0..width {
            let new_x = (x + x_shift) % width;
            let new_y = (y + y_shift) % height;
            output[new_y * width + new_x] = input[y * width + x];
        }
    }
    output
}

fn shift_real(input: &[f64], width: usize, height: usize, x_shift: usize, y_shift: usize) -> Vec<f64> {
    let mut output = vec![0.0; width * height];
    for y in 0..height {
        for x in 0..width {
            let new_x = (x + x_shift) % width;
            let new_y = (y + y_shift) % height;
            output[new_y * width + new_x] = input[y * width + x];
        }
    }
    output
}

pub fn osa_axis_arrow_degrees(axis_degrees: f64) -> f64 {
    (axis_degrees - 90.0).rem_euclid(360.0)
}

pub fn wavefront_phase_radians(wavelength_nm: f64, wavefront_um: f64) -> f64 {
    let wavelength_um = wavelength_nm * 1e-3;
    (2.0 * PI / wavelength_um) * wavefront_um
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        let delta = (actual - expected).abs();
        assert!(
            delta <= tolerance,
            "expected {expected}, got {actual}, delta {delta}, tolerance {tolerance}"
        );
    }

    #[test]
    fn converts_sphere_only_prescription_to_defocus() {
        let coefficients = sca_to_zernike(
            ScaPrescription {
                sphere_diopters: -2.0,
                cylinder_diopters: 0.0,
                axis_degrees: 0.0,
            },
            6.0,
        );

        assert_close(coefficients[0].coefficient_um, 0.0, 1e-12);
        assert_close(coefficients[1].coefficient_um, 9.0 / (2.0 * 3.0_f64.sqrt()), 1e-12);
        assert_close(coefficients[2].coefficient_um, 0.0, 1e-12);
    }

    #[test]
    fn maps_cylinder_axis_into_astigmatism_terms() {
        let coefficients = sca_to_zernike(
            ScaPrescription {
                sphere_diopters: 0.0,
                cylinder_diopters: -1.5,
                axis_degrees: 45.0,
            },
            6.0,
        );

        assert_close(coefficients[0].coefficient_um, 13.5 / (4.0 * 6.0_f64.sqrt()), 1e-12);
        assert_close(coefficients[2].coefficient_um, 0.0, 1e-12);
    }

    #[test]
    fn evaluates_standard_radial_component() {
        assert_close(zernike_radial(2, 0, 0.5), -0.5, 1e-12);
        assert_close(zernike_radial(2, 2, 0.5), 0.25, 1e-12);
    }

    #[test]
    fn styles_crawford_can_be_disabled() {
        let config = StylesCrawfordConfig::default();
        assert_close(styles_crawford_weight(0.8, config), 1.0, 1e-12);
    }

    #[test]
    fn derives_pupil_samples_from_target_fov() {
        assert_eq!(derive_pupil_samples(555.0, 6.0, 240.0), 754);
    }

    #[test]
    fn psf_is_normalized() {
        let wavefront = build_wavefront(&WavefrontRequest {
            pupil_samples: 16,
            pupil_diameter_mm: 6.0,
            prescription: None,
            additional_coefficients: Vec::new(),
            styles_crawford: StylesCrawfordConfig::default(),
        });

        let psf = compute_psf(&wavefront, 555.0, 32, 6.0).expect("psf generation should succeed");
        assert_close(psf.data.iter().sum(), 1.0, 1e-9);
        assert_eq!(psf.width, 32);
        assert_eq!(psf.height, 32);
    }

    #[test]
    fn chart_generation_creates_dark_landolt_pixels() {
        let chart = create_landolt_chart_grid(&default_acuities(), 120.0, 3, 128);
        assert_eq!(chart.placements.len(), 9);
        assert!(chart.data.iter().any(|value| *value == 0.0));
    }

    #[test]
    fn wavefront_builder_combines_prescription_and_manual_terms() {
        let result = build_wavefront(&WavefrontRequest {
            pupil_samples: 5,
            pupil_diameter_mm: 6.0,
            prescription: Some(ScaPrescription {
                sphere_diopters: 0.0,
                cylinder_diopters: -1.0,
                axis_degrees: 0.0,
            }),
            additional_coefficients: vec![ZernikeCoefficient {
                mode: ZernikeMode { n: 4, m: 0 },
                coefficient_um: -0.12,
                source: CoefficientSource::Manual,
            }],
            styles_crawford: StylesCrawfordConfig {
                enabled: true,
                rho: 0.12,
            },
        });

        assert_eq!(result.width, 5);
        assert_eq!(result.height, 5);
        assert_eq!(result.coefficients.len(), 4);
        assert_eq!(result.wavefront_um.len(), 25);
        assert_eq!(result.amplitude.len(), 25);
        assert!(result.max_um >= result.min_um);
        assert!(result
            .amplitude
            .iter()
            .zip(result.pupil_mask.iter())
            .all(|(amplitude, mask)| *mask == 0.0 || *amplitude <= 1.0));
    }

    #[test]
    fn full_retinal_simulation_returns_expected_dimensions() {
        let result = simulate_retinal_image(&RetinalSimulationRequest {
            optics: OpticalConfig {
                wavelength_nm: 555.0,
                pupil_diameter_mm: 6.0,
                image_samples: 128,
                target_fov_arcmin: 30.0,
                pupil_samples: Some(64),
            },
            prescription: Some(ScaPrescription {
                sphere_diopters: 0.0,
                cylinder_diopters: -0.5,
                axis_degrees: 45.0,
            }),
            additional_coefficients: vec![ZernikeCoefficient {
                mode: ZernikeMode { n: 4, m: 0 },
                coefficient_um: -0.08,
                source: CoefficientSource::Manual,
            }],
            styles_crawford: StylesCrawfordConfig {
                enabled: true,
                rho: 0.12,
            },
            acuities: vec![0.5, 1.0, 2.0, 0.7],
            grid_size: 2,
        })
        .expect("simulation should succeed");

        assert_eq!(result.wavefront.width, 64);
        assert_eq!(result.psf.width, 128);
        assert_eq!(result.chart.width, 128);
        assert_eq!(result.retinal_image.len(), 128 * 128);
    }

    #[test]
    fn simulation_clamps_derived_pupil_samples_to_image_samples() {
        let result = simulate_retinal_image(&RetinalSimulationRequest {
            optics: OpticalConfig {
                wavelength_nm: 555.0,
                pupil_diameter_mm: 6.0,
                image_samples: 512,
                target_fov_arcmin: 240.0,
                pupil_samples: None,
            },
            prescription: None,
            additional_coefficients: Vec::new(),
            styles_crawford: StylesCrawfordConfig::default(),
            acuities: default_acuities(),
            grid_size: 3,
        })
        .expect("simulation should clamp instead of erroring");

        assert_eq!(result.pupil_samples, 512);
        assert_eq!(result.wavefront.width, 512);
        assert_eq!(result.psf.width, 512);
    }
}
