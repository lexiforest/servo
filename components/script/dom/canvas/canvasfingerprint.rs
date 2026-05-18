/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use pixels::{Snapshot, SnapshotAlphaMode, SnapshotPixelFormat};
use servo_config::pref;

const BLINK_LOW_ENTROPY_CANVAS_PROBE: [u8; 16] = [
    255, 255, 255, 255, 178, 178, 178, 255, 246, 246, 246, 255, 55, 55, 55, 255,
];

pub(crate) fn apply_canvas_export_noise(snapshot: &mut Snapshot, surface: &str) {
    if !pref!(bimp_js_canvas_noise_enabled) {
        return;
    }

    let amplitude = pref!(bimp_js_canvas_noise_amplitude).clamp(0, 8) as i16;
    if amplitude == 0 {
        return;
    }

    snapshot.transform(
        SnapshotAlphaMode::Transparent {
            premultiplied: false,
        },
        SnapshotPixelFormat::RGBA,
    );

    let size = snapshot.size();
    let seed = pref!(bimp_js_canvas_noise_seed);
    let base_hash = stable_hash(&seed)
        ^ stable_hash(surface)
        ^ ((size.width as u64) << 32)
        ^ size.height as u64;
    let modulus = amplitude * 2 + 1;

    for (pixel_index, pixel) in snapshot.as_raw_bytes_mut().chunks_exact_mut(4).enumerate() {
        if pixel[3] == 0 {
            continue;
        }

        for channel in 0..3 {
            let hash = mix_hash(base_hash ^ ((pixel_index as u64) << 8) ^ channel as u64);
            let delta = (hash % modulus as u64) as i16 - amplitude;
            pixel[channel] = (pixel[channel] as i16 + delta).clamp(0, 255) as u8;
        }
    }
}

pub(crate) fn apply_blink_low_entropy_canvas_probe(snapshot: &mut Snapshot) {
    if !pref!(bimp_js_canvas_blink_low_entropy_probe) || snapshot.size().area() != 4 {
        return;
    }

    snapshot.transform(
        SnapshotAlphaMode::Transparent {
            premultiplied: false,
        },
        SnapshotPixelFormat::RGBA,
    );

    let data = snapshot.as_raw_bytes_mut();
    if data.len() != BLINK_LOW_ENTROPY_CANVAS_PROBE.len() || !looks_like_low_entropy_probe(data) {
        return;
    }

    data.copy_from_slice(&BLINK_LOW_ENTROPY_CANVAS_PROBE);
}

fn looks_like_low_entropy_probe(data: &[u8]) -> bool {
    if data.chunks_exact(4).any(|pixel| {
        pixel[3] != 255 || pixel[0] != pixel[1] || pixel[1] != pixel[2]
    }) {
        return false;
    }

    let grays = [
        data[0], data[4], data[8], data[12],
    ];
    grays[0] == 255 &&
        grays.iter().any(|value| *value != 0 && *value != 255) &&
        grays.windows(2).any(|window| window[0] != window[1])
}

fn stable_hash(input: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn mix_hash(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}
