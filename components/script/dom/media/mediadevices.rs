/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::context::JSContext;
use js::realm::CurrentRealm;
use script_bindings::reflector::reflect_dom_object;
use servo_media::streams::MediaStreamType;
use servo_media::streams::capture::{Constrain, ConstrainRange, MediaTrackConstraintSet};

use crate::dom::bindings::codegen::Bindings::MediaDeviceInfoBinding::MediaDeviceKind;
use crate::dom::bindings::codegen::Bindings::MediaDevicesBinding::{
    MediaDevicesMethods, MediaStreamConstraints,
};
use crate::dom::bindings::codegen::UnionTypes::{
    BooleanOrMediaTrackConstraints, ClampedUnsignedLongOrConstrainULongRange as ConstrainULong,
    DoubleOrConstrainDoubleRange as ConstrainDouble,
};
use crate::dom::bindings::reflector::DomGlobal;
use crate::dom::bindings::root::DomRoot;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::dom::media::mediadeviceinfo::MediaDeviceInfo;
use crate::dom::media::mediastream::MediaStream;
use crate::dom::media::mediastreamtrack::MediaStreamTrack;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MediaDevices {
    eventtarget: EventTarget,
}

impl MediaDevices {
    pub(crate) fn new_inherited() -> MediaDevices {
        MediaDevices {
            eventtarget: EventTarget::new_inherited(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<MediaDevices> {
        reflect_dom_object(Box::new(MediaDevices::new_inherited()), global, can_gc)
    }
}

impl MediaDevicesMethods<crate::DomTypeHolder> for MediaDevices {
    /// <https://w3c.github.io/mediacapture-main/#dom-mediadevices-getusermedia>
    fn GetUserMedia(
        &self,
        cx: &mut CurrentRealm,
        constraints: &MediaStreamConstraints,
    ) -> Rc<Promise> {
        let p = Promise::new_in_realm(cx);
        let media = servo_media::ServoMedia::get();
        let stream = MediaStream::new(cx, &self.global());
        if let Some(constraints) = convert_constraints(&constraints.audio) &&
            let Some(audio) = media.create_audioinput_stream(constraints)
        {
            let track = MediaStreamTrack::new(cx, &self.global(), audio, MediaStreamType::Audio);
            stream.add_track(&track);
        }
        if let Some(constraints) = convert_constraints(&constraints.video) &&
            let Some(video) = media.create_videoinput_stream(constraints)
        {
            let track = MediaStreamTrack::new(cx, &self.global(), video, MediaStreamType::Video);
            stream.add_track(&track);
        }

        p.resolve_native(cx, &stream);
        p
    }

    /// <https://w3c.github.io/mediacapture-main/#dom-mediadevices-enumeratedevices>
    fn EnumerateDevices(&self, cx: &mut JSContext) -> Rc<Promise> {
        // Step 1.
        let mut realm = CurrentRealm::assert(cx);
        let p = Promise::new_in_realm(&mut realm);

        // Step 2.
        // XXX These steps should be run in parallel.
        // XXX Steps 2.1 - 2.4

        // Step 2.5. Use persona-backed device counts instead of leaking host hardware.
        let result_list = persona_media_devices(cx, &self.global());

        p.resolve_native(cx, &result_list);

        // Step 3.
        p
    }
}

fn persona_media_devices(
    cx: &mut JSContext,
    global: &GlobalScope,
) -> Vec<DomRoot<MediaDeviceInfo>> {
    let mut devices = Vec::new();
    push_persona_media_devices(
        &mut devices,
        cx,
        global,
        MediaDeviceKind::Audioinput,
        "audio-input",
        servo_config::pref!(bimp_js_media_audio_inputs),
    );
    push_persona_media_devices(
        &mut devices,
        cx,
        global,
        MediaDeviceKind::Videoinput,
        "video-input",
        servo_config::pref!(bimp_js_media_video_inputs),
    );
    push_persona_media_devices(
        &mut devices,
        cx,
        global,
        MediaDeviceKind::Audiooutput,
        "audio-output",
        servo_config::pref!(bimp_js_media_audio_outputs),
    );
    devices
}

fn push_persona_media_devices(
    devices: &mut Vec<DomRoot<MediaDeviceInfo>>,
    cx: &mut JSContext,
    global: &GlobalScope,
    kind: MediaDeviceKind,
    prefix: &str,
    count: i64,
) {
    for index in 0..count.clamp(0, 16) {
        let device_id = format!("{prefix}-{index}");
        let group_id = format!("group-{index}");
        devices.push(MediaDeviceInfo::new(
            cx, global, &device_id, kind, "", &group_id,
        ));
    }
}

fn convert_constraints(js: &BooleanOrMediaTrackConstraints) -> Option<MediaTrackConstraintSet> {
    match js {
        BooleanOrMediaTrackConstraints::Boolean(false) => None,
        BooleanOrMediaTrackConstraints::Boolean(true) => Some(Default::default()),
        BooleanOrMediaTrackConstraints::MediaTrackConstraints(c) => Some(MediaTrackConstraintSet {
            height: c.parent.height.as_ref().and_then(convert_culong),
            width: c.parent.width.as_ref().and_then(convert_culong),
            aspect: c.parent.aspectRatio.as_ref().and_then(convert_cdouble),
            frame_rate: c.parent.frameRate.as_ref().and_then(convert_cdouble),
            sample_rate: c.parent.sampleRate.as_ref().and_then(convert_culong),
        }),
    }
}

fn convert_culong(js: &ConstrainULong) -> Option<Constrain<u32>> {
    match js {
        ConstrainULong::ClampedUnsignedLong(val) => Some(Constrain::Value(*val)),
        ConstrainULong::ConstrainULongRange(range) => {
            if range.parent.min.is_some() || range.parent.max.is_some() {
                Some(Constrain::Range(ConstrainRange {
                    min: range.parent.min,
                    max: range.parent.max,
                    ideal: range.ideal,
                }))
            } else {
                range.exact.map(Constrain::Value)
            }
        },
    }
}

fn convert_cdouble(js: &ConstrainDouble) -> Option<Constrain<f64>> {
    match js {
        ConstrainDouble::Double(val) => Some(Constrain::Value(**val)),
        ConstrainDouble::ConstrainDoubleRange(range) => {
            if range.parent.min.is_some() || range.parent.max.is_some() {
                Some(Constrain::Range(ConstrainRange {
                    min: range.parent.min.map(|x| *x),
                    max: range.parent.max.map(|x| *x),
                    ideal: range.ideal.map(|x| *x),
                }))
            } else {
                range.exact.map(|exact| Constrain::Value(*exact))
            }
        },
    }
}
