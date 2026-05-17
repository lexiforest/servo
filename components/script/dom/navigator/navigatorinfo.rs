/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::str::DOMString;

#[expect(non_snake_case)]
pub(crate) fn Product() -> DOMString {
    DOMString::from("Gecko")
}

#[expect(non_snake_case)]
pub(crate) fn ProductSub() -> DOMString {
    let value = servo_config::pref!(bimp_js_product_sub);
    DOMString::from(if value.is_empty() {
        "20100101".to_string()
    } else {
        value
    })
}

#[expect(non_snake_case)]
pub(crate) fn Vendor() -> DOMString {
    DOMString::from(servo_config::pref!(bimp_js_vendor))
}

#[expect(non_snake_case)]
pub(crate) fn VendorSub() -> DOMString {
    DOMString::from("")
}

#[expect(non_snake_case)]
pub(crate) fn TaintEnabled() -> bool {
    false
}

#[expect(non_snake_case)]
pub(crate) fn AppName() -> DOMString {
    DOMString::from("Netscape") // Like Gecko/Webkit
}

#[expect(non_snake_case)]
pub(crate) fn AppCodeName() -> DOMString {
    DOMString::from("Mozilla")
}

#[expect(non_snake_case)]
#[cfg(target_os = "windows")]
pub(crate) fn Platform() -> DOMString {
    let value = servo_config::pref!(bimp_js_platform);
    DOMString::from(if value.is_empty() {
        "Win32".to_string()
    } else {
        value
    })
}

#[expect(non_snake_case)]
#[cfg(any(target_os = "android", target_os = "linux", target_os = "freebsd"))]
pub(crate) fn Platform() -> DOMString {
    let value = servo_config::pref!(bimp_js_platform);
    DOMString::from(if value.is_empty() {
        "Linux".to_string()
    } else {
        value
    })
}

#[expect(non_snake_case)]
#[cfg(target_os = "macos")]
pub(crate) fn Platform() -> DOMString {
    let value = servo_config::pref!(bimp_js_platform);
    DOMString::from(if value.is_empty() {
        "Mac".to_string()
    } else {
        value
    })
}

#[expect(non_snake_case)]
#[cfg(target_os = "ios")]
pub(crate) fn Platform() -> DOMString {
    let value = servo_config::pref!(bimp_js_platform);
    DOMString::from(if value.is_empty() {
        "iOS".to_string()
    } else {
        value
    })
}

#[expect(non_snake_case)]
pub(crate) fn UserAgent(user_agent: &str) -> DOMString {
    DOMString::from(user_agent)
}

#[expect(non_snake_case)]
pub(crate) fn AppVersion() -> DOMString {
    let value = servo_config::pref!(bimp_js_app_version);
    DOMString::from(if value.is_empty() {
        "4.0".to_string()
    } else {
        value
    })
}

#[expect(non_snake_case)]
pub(crate) fn Language() -> DOMString {
    let value = servo_config::pref!(bimp_js_language);
    DOMString::from(if value.is_empty() {
        net_traits::get_current_locale().0.clone()
    } else {
        value
    })
}
