/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use servo_config::pref;

pub(crate) fn apply_domrect_persona(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> (f64, f64, f64, f64) {
    if !pref!(bimp_js_domrect_enabled) {
        return (x, y, width, height);
    }

    if is_zero_area(width, height) {
        return (0.0, 0.0, 0.0, 0.0);
    }

    if pref!(bimp_js_domrect_clamp_transforms) && is_blink_known_rotated_square(x, y, width, height) {
        return (
            -20.710678100585938,
            -20.710678100585938,
            141.42135620117188,
            141.42135620117188,
        );
    }

    let steps = pref!(bimp_js_domrect_quantization_steps_per_px);
    if steps <= 0 {
        return (
            normalize_zero(x),
            normalize_zero(y),
            normalize_zero(width),
            normalize_zero(height),
        );
    }

    let steps = steps.clamp(1, 1024) as f64;
    (
        quantize(x, steps),
        quantize(y, steps),
        quantize(width, steps),
        quantize(height, steps),
    )
}

pub(crate) fn fill_empty_element_client_rects() -> bool {
    pref!(bimp_js_domrect_enabled) && pref!(bimp_js_domrect_fill_empty_client_rects)
}

fn is_zero_area(width: f64, height: f64) -> bool {
    width == 0.0 && height == 0.0
}

fn is_blink_known_rotated_square(x: f64, y: f64, width: f64, height: f64) -> bool {
    nearly_equal(x, y) &&
        nearly_equal(width, height) &&
        (-21.0..=-20.0).contains(&x) &&
        (141.0..=142.0).contains(&width)
}

fn nearly_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < 0.001
}

fn quantize(value: f64, steps: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }

    let phase = persona_phase();
    let scaled = value * steps + phase;
    let quantized = match pref!(bimp_js_domrect_rounding).as_str() {
        "floor" => scaled.floor(),
        "ceil" => scaled.ceil(),
        "truncate" => scaled.trunc(),
        _ => scaled.round(),
    };
    let quantized = (quantized - phase) / steps;
    normalize_zero(quantized)
}

fn persona_phase() -> f64 {
    let hash = pref!(bimp_js_domrect_seed)
        .bytes()
        .fold(2_166_136_261_u32, |hash, byte| {
            hash.wrapping_mul(16_777_619) ^ u32::from(byte)
        });
    f64::from(hash % 1024) / 1024.0
}

fn normalize_zero(value: f64) -> f64 {
    if value == -0.0 && !pref!(bimp_js_domrect_preserve_negative_zero) { 0.0 } else { value }
}
