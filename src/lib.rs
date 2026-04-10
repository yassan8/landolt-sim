mod optics;

pub use optics::{
    build_wavefront, circular_convolution, compute_psf, create_landolt_chart_grid,
    derive_pupil_samples, image_plane_sampling, osa_axis_arrow_degrees, sca_to_zernike,
    simulate_retinal_image, styles_crawford_weight, wavefront_phase_radians,
    CoefficientSource, LandoltChartResult, LandoltRingPlacement, OpticalConfig, PsfResult,
    RetinalSimulationRequest, RetinalSimulationResult, ScaPrescription, StylesCrawfordConfig,
    WavefrontRequest, WavefrontResult, ZernikeCoefficient, ZernikeMode,
};

use wasm_bindgen::prelude::*;

fn to_js_error(message: impl Into<String>) -> JsValue {
    JsValue::from_str(&message.into())
}

#[wasm_bindgen]
pub fn sca_to_zernike_js(input: JsValue) -> Result<JsValue, JsValue> {
    #[derive(serde::Deserialize)]
    struct Request {
        sphere_diopters: f64,
        cylinder_diopters: f64,
        axis_degrees: f64,
        pupil_diameter_mm: f64,
    }

    let request: Request =
        serde_wasm_bindgen::from_value(input).map_err(|error| to_js_error(error.to_string()))?;
    let coefficients = sca_to_zernike(
        ScaPrescription {
            sphere_diopters: request.sphere_diopters,
            cylinder_diopters: request.cylinder_diopters,
            axis_degrees: request.axis_degrees,
        },
        request.pupil_diameter_mm,
    );

    serde_wasm_bindgen::to_value(&coefficients).map_err(|error| to_js_error(error.to_string()))
}

#[wasm_bindgen]
pub fn generate_wavefront_js(input: JsValue) -> Result<JsValue, JsValue> {
    let request: WavefrontRequest =
        serde_wasm_bindgen::from_value(input).map_err(|error| to_js_error(error.to_string()))?;

    if request.pupil_diameter_mm <= 0.0 {
        return Err(to_js_error("pupil_diameter_mm must be positive"));
    }

    if request.pupil_samples < 2 {
        return Err(to_js_error("pupil_samples must be at least 2"));
    }

    let result = build_wavefront(&request);
    serde_wasm_bindgen::to_value(&result).map_err(|error| to_js_error(error.to_string()))
}

#[wasm_bindgen]
pub fn simulate_retinal_image_js(input: JsValue) -> Result<JsValue, JsValue> {
    let request: RetinalSimulationRequest =
        serde_wasm_bindgen::from_value(input).map_err(|error| to_js_error(error.to_string()))?;
    let result = simulate_retinal_image(&request).map_err(to_js_error)?;
    serde_wasm_bindgen::to_value(&result).map_err(|error| to_js_error(error.to_string()))
}