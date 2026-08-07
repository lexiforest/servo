/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::rc::Rc;

use dom_struct::dom_struct;
use js::realm::CurrentRealm;
use js::rust::MutableHandleValue;
use script_bindings::script_runtime::temp_cx;
use script_bindings::reflector::{Reflector, reflect_dom_object};
use servo_config::pref;

use crate::dom::bindings::codegen::Bindings::NavigatorUADataBinding::{
    NavigatorUABrandVersion, NavigatorUADataMethods, UADataValues,
};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::utils::to_frozen_array;
use crate::dom::globalscope::GlobalScope;
use crate::dom::promise::Promise;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct NavigatorUAData {
    reflector_: Reflector,
}

impl NavigatorUAData {
    fn new_inherited() -> NavigatorUAData {
        NavigatorUAData {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<NavigatorUAData> {
        reflect_dom_object(Box::new(NavigatorUAData::new_inherited()), global, can_gc)
    }
}

impl NavigatorUADataMethods<crate::DomTypeHolder> for NavigatorUAData {
    #[expect(unsafe_code, reason = "generated any getter does not provide a JS context")]
    fn Brands(&self, retval: MutableHandleValue) {
        // The generated `any` getter does not pass a context yet.
        let mut cx = unsafe { temp_cx() };
        to_frozen_array(&mut cx, &ua_brands(), retval);
    }

    fn Mobile(&self) -> bool {
        pref!(bimp_js_ua_mobile)
    }

    fn Platform(&self) -> DOMString {
        DOMString::from(pref!(bimp_js_ua_platform))
    }

    #[expect(unsafe_code, reason = "generated promise method does not provide a JS context")]
    fn GetHighEntropyValues(&self, _hints: Vec<DOMString>) -> Rc<Promise> {
        // The generated promise method does not pass a context yet.
        let mut cx = unsafe { temp_cx() };
        let mut realm = CurrentRealm::assert(&mut cx);
        let promise = Promise::new_in_realm(&mut realm);
        let values = UADataValues {
            brands: ua_brands(),
            fullVersionList: ua_full_version_list(),
            mobile: self.Mobile(),
            platform: self.Platform(),
            platformVersion: DOMString::from(pref!(bimp_js_ua_platform_version)),
            architecture: DOMString::from(pref!(bimp_js_ua_architecture)),
            bitness: DOMString::from(pref!(bimp_js_ua_bitness)),
            model: DOMString::from(pref!(bimp_js_ua_model)),
            uaFullVersion: DOMString::from(pref!(bimp_js_ua_full_version)),
            fullVersion: DOMString::from(pref!(bimp_js_ua_full_version)),
        };
        promise.resolve_native(&mut realm, &values);
        promise
    }
}

fn ua_brands() -> Vec<NavigatorUABrandVersion> {
    parse_brands(&pref!(bimp_js_ua_brands))
}

fn ua_full_version_list() -> Vec<NavigatorUABrandVersion> {
    let full_version = pref!(bimp_js_ua_full_version);
    ua_brands()
        .into_iter()
        .map(|brand| NavigatorUABrandVersion {
            version: if brand.brand == "Not.A/Brand" {
                brand.version
            } else {
                DOMString::from(full_version.clone())
            },
            brand: brand.brand,
        })
        .collect()
}

fn parse_brands(input: &str) -> Vec<NavigatorUABrandVersion> {
    let mut brands = input
        .split(',')
        .filter_map(|entry| {
            let mut brand = None;
            let mut version = None;
            for part in entry.split(';') {
                let part = part.trim();
                if let Some(value) = part
                    .strip_prefix('"')
                    .and_then(|value| value.strip_suffix('"'))
                {
                    brand = Some(value.to_string());
                } else if let Some(value) = part.strip_prefix("v=\"") {
                    version = value.strip_suffix('"').map(ToString::to_string);
                }
            }
            Some(NavigatorUABrandVersion {
                brand: DOMString::from(brand?),
                version: DOMString::from(version?),
            })
        })
        .collect::<Vec<_>>();

    if brands.is_empty() {
        brands = vec![
            NavigatorUABrandVersion {
                brand: DOMString::from("Chromium"),
                version: DOMString::from("136"),
            },
            NavigatorUABrandVersion {
                brand: DOMString::from("Google Chrome"),
                version: DOMString::from("136"),
            },
            NavigatorUABrandVersion {
                brand: DOMString::from("Not.A/Brand"),
                version: DOMString::from("99"),
            },
        ];
    }
    brands
}
