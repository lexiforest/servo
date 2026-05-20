/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::MimeTypeArrayBinding::MimeTypeArrayMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mimetype::MimeType;
use crate::dom::plugin::{chrome_pdf_mime_types, mime_type_for_type};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct MimeTypeArray {
    reflector_: Reflector,
}

impl MimeTypeArray {
    pub(crate) fn new_inherited() -> MimeTypeArray {
        MimeTypeArray {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<MimeTypeArray> {
        reflect_dom_object(Box::new(MimeTypeArray::new_inherited()), global, can_gc)
    }
}

impl MimeTypeArrayMethods<crate::DomTypeHolder> for MimeTypeArray {
    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-length>
    fn Length(&self) -> u32 {
        chrome_pdf_mime_types().len() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item>
    fn Item(&self, index: u32) -> Option<DomRoot<MimeType>> {
        chrome_pdf_mime_types()
            .get(index as usize)
            .copied()
            .map(|mime_type| MimeType::new(&self.global(), mime_type, CanGc::deprecated_note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-nameditem>
    fn NamedItem(&self, name: DOMString) -> Option<DomRoot<MimeType>> {
        mime_type_for_type(&name.str())
            .map(|mime_type| MimeType::new(&self.global(), mime_type, CanGc::deprecated_note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-mimetypearray-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<MimeType>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DomRoot<MimeType>> {
        self.NamedItem(name)
    }

    /// <https://heycam.github.io/webidl/#dfn-supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        chrome_pdf_mime_types()
            .iter()
            .map(|mime_type| DOMString::from(mime_type.type_))
            .collect()
    }
}
