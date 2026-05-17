/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::MimeTypeBinding::MimeTypeMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::plugin::{MimeTypeMetadata, Plugin, plugin_for_name};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MimeType {
    reflector_: Reflector,
    #[no_trace]
    #[ignore_malloc_size_of = "static MIME type metadata"]
    metadata: MimeTypeMetadata,
}

impl MimeType {
    fn new_inherited(metadata: MimeTypeMetadata) -> MimeType {
        MimeType {
            reflector_: Reflector::new(),
            metadata,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        metadata: MimeTypeMetadata,
        can_gc: CanGc,
    ) -> DomRoot<MimeType> {
        reflect_dom_object(Box::new(MimeType::new_inherited(metadata)), global, can_gc)
    }
}

impl MimeTypeMethods<crate::DomTypeHolder> for MimeType {
    /// <https://html.spec.whatwg.org/multipage/#dom-mimetype-type>
    fn Type(&self) -> DOMString {
        DOMString::from(self.metadata.type_)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetype-description>
    fn Description(&self) -> DOMString {
        DOMString::from(self.metadata.description)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetype-suffixes>
    fn Suffixes(&self) -> DOMString {
        DOMString::from(self.metadata.suffixes)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetype-enabledplugin>
    fn EnabledPlugin(&self) -> DomRoot<Plugin> {
        let plugin = plugin_for_name(self.metadata.enabled_plugin_name)
            .unwrap_or_else(|| plugin_for_name("PDF Viewer").expect("PDF Viewer plugin exists"));
        Plugin::new(&self.global(), plugin, CanGc::deprecated_note())
    }
}
