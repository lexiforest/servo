/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use servo_config::pref;

use crate::dom::bindings::codegen::Bindings::ScreenOrientationBinding::ScreenOrientationMethods;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct ScreenOrientation {
    reflector_: Reflector,
}

impl ScreenOrientation {
    fn new_inherited() -> ScreenOrientation {
        ScreenOrientation {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<ScreenOrientation> {
        reflect_dom_object(Box::new(Self::new_inherited()), window, can_gc)
    }
}

impl ScreenOrientationMethods<crate::DomTypeHolder> for ScreenOrientation {
    fn Type(&self) -> DOMString {
        let value = pref!(bimp_js_screen_orientation_type);
        DOMString::from(if value.is_empty() {
            "landscape-primary".to_string()
        } else {
            value
        })
    }

    fn Angle(&self) -> u16 {
        pref!(bimp_js_screen_orientation_angle).clamp(0, 359) as u16
    }
}
