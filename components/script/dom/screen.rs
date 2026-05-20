/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use embedder_traits::{EmbedderMsg, ScreenMetrics};
use servo_base::generic_channel;
use servo_config::pref;

use crate::dom::bindings::codegen::Bindings::ScreenBinding::ScreenMethods;
use crate::dom::bindings::num::Finite;
use crate::dom::bindings::reflector::{Reflector, reflect_dom_object};
use crate::dom::bindings::root::{Dom, DomRoot, MutNullableDom};
use crate::dom::screenorientation::ScreenOrientation;
use crate::dom::window::Window;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct Screen {
    reflector_: Reflector,
    window: Dom<Window>,
    orientation: MutNullableDom<ScreenOrientation>,
}

impl Screen {
    fn new_inherited(window: &Window) -> Screen {
        Screen {
            reflector_: Reflector::new(),
            window: Dom::from_ref(window),
            orientation: MutNullableDom::new(Some(&ScreenOrientation::new(
                window,
                CanGc::deprecated_note(),
            ))),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<Screen> {
        reflect_dom_object(Box::new(Screen::new_inherited(window)), window, can_gc)
    }

    /// Retrives [`ScreenMetrics`] from the embedder.
    fn screen_metrics(&self) -> ScreenMetrics {
        let (sender, receiver) = generic_channel::channel().expect("Failed to create IPC channel!");

        self.window.send_to_embedder(EmbedderMsg::GetScreenMetrics(
            self.window.webview_id(),
            sender,
        ));

        receiver.recv().unwrap_or_default()
    }
}

impl ScreenMethods<crate::DomTypeHolder> for Screen {
    /// <https://drafts.csswg.org/cssom-view/#dom-screen-availwidth>
    fn AvailWidth(&self) -> Finite<f64> {
        let value = pref!(bimp_js_screen_avail_width);
        Finite::wrap(if value > 0 {
            value as f64
        } else {
            self.screen_metrics().available_size.width as f64
        })
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-availheight>
    fn AvailHeight(&self) -> Finite<f64> {
        let value = pref!(bimp_js_screen_avail_height);
        Finite::wrap(if value > 0 {
            value as f64
        } else {
            self.screen_metrics().available_size.height as f64
        })
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-availleft>
    fn AvailLeft(&self) -> i32 {
        pref!(bimp_js_screen_avail_left).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-availtop>
    fn AvailTop(&self) -> i32 {
        pref!(bimp_js_screen_avail_top).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-width>
    fn Width(&self) -> Finite<f64> {
        let value = pref!(bimp_js_screen_width);
        Finite::wrap(if value > 0 {
            value as f64
        } else {
            self.screen_metrics().screen_size.width as f64
        })
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-height>
    fn Height(&self) -> Finite<f64> {
        let value = pref!(bimp_js_screen_height);
        Finite::wrap(if value > 0 {
            value as f64
        } else {
            self.screen_metrics().screen_size.height as f64
        })
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-left>
    fn Left(&self) -> i32 {
        pref!(bimp_js_screen_left).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-top>
    fn Top(&self) -> i32 {
        pref!(bimp_js_screen_top).clamp(i32::MIN as i64, i32::MAX as i64) as i32
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-colordepth>
    fn ColorDepth(&self) -> u32 {
        let value = pref!(bimp_js_screen_color_depth);
        if value > 0 { value as u32 } else { 24 }
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-pixeldepth>
    fn PixelDepth(&self) -> u32 {
        let value = pref!(bimp_js_screen_pixel_depth);
        if value > 0 { value as u32 } else { 24 }
    }

    /// <https://drafts.csswg.org/cssom-view/#dom-screen-isextended>
    fn IsExtended(&self) -> bool {
        pref!(bimp_js_screen_is_extended)
    }

    /// <https://w3c.github.io/screen-orientation/#dom-screen-orientation>
    fn Orientation(&self) -> DomRoot<ScreenOrientation> {
        self.orientation
            .get()
            .expect("ScreenOrientation should be initialized with Screen")
    }
}
