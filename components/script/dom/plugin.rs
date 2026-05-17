/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PluginBinding::PluginMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::mimetype::MimeType;
use crate::script_runtime::CanGc;

#[derive(Clone, Copy)]
pub(crate) struct MimeTypeMetadata {
    pub(crate) type_: &'static str,
    pub(crate) description: &'static str,
    pub(crate) suffixes: &'static str,
    pub(crate) enabled_plugin_name: &'static str,
}

#[derive(Clone, Copy)]
pub(crate) struct PluginMetadata {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) filename: &'static str,
    pub(crate) mime_types: &'static [MimeTypeMetadata],
}

const PDF_MIME_TYPES: [MimeTypeMetadata; 2] = [
    MimeTypeMetadata {
        type_: "application/pdf",
        description: "Portable Document Format",
        suffixes: "pdf",
        enabled_plugin_name: "PDF Viewer",
    },
    MimeTypeMetadata {
        type_: "text/pdf",
        description: "Portable Document Format",
        suffixes: "pdf",
        enabled_plugin_name: "PDF Viewer",
    },
];

const PDF_PLUGINS: [PluginMetadata; 5] = [
    pdf_plugin("PDF Viewer"),
    pdf_plugin("Chrome PDF Viewer"),
    pdf_plugin("Chromium PDF Viewer"),
    pdf_plugin("Microsoft Edge PDF Viewer"),
    pdf_plugin("WebKit built-in PDF"),
];

const fn pdf_plugin(name: &'static str) -> PluginMetadata {
    PluginMetadata {
        name,
        description: "Portable Document Format",
        filename: "internal-pdf-viewer",
        mime_types: &PDF_MIME_TYPES,
    }
}

pub(crate) fn chrome_pdf_plugins_enabled() -> bool {
    servo_config::pref!(bimp_js_pdf_viewer_enabled)
}

pub(crate) fn chrome_pdf_plugins() -> &'static [PluginMetadata] {
    if chrome_pdf_plugins_enabled() {
        &PDF_PLUGINS
    } else {
        &[]
    }
}

pub(crate) fn chrome_pdf_mime_types() -> &'static [MimeTypeMetadata] {
    if chrome_pdf_plugins_enabled() {
        &PDF_MIME_TYPES
    } else {
        &[]
    }
}

pub(crate) fn plugin_for_name(name: &str) -> Option<PluginMetadata> {
    chrome_pdf_plugins()
        .iter()
        .copied()
        .find(|plugin| plugin.name == name)
}

pub(crate) fn mime_type_for_type(type_: &str) -> Option<MimeTypeMetadata> {
    chrome_pdf_mime_types()
        .iter()
        .copied()
        .find(|mime_type| mime_type.type_ == type_)
}

#[dom_struct]
pub(crate) struct Plugin {
    reflector_: Reflector,
    #[no_trace]
    #[ignore_malloc_size_of = "static plugin metadata"]
    metadata: PluginMetadata,
}

impl Plugin {
    fn new_inherited(metadata: PluginMetadata) -> Plugin {
        Plugin {
            reflector_: Reflector::new(),
            metadata,
        }
    }

    pub(crate) fn new(
        global: &GlobalScope,
        metadata: PluginMetadata,
        can_gc: CanGc,
    ) -> DomRoot<Plugin> {
        reflect_dom_object(Box::new(Plugin::new_inherited(metadata)), global, can_gc)
    }
}

impl PluginMethods<crate::DomTypeHolder> for Plugin {
    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-name>
    fn Name(&self) -> DOMString {
        DOMString::from(self.metadata.name)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-description>
    fn Description(&self) -> DOMString {
        DOMString::from(self.metadata.description)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-filename>
    fn Filename(&self) -> DOMString {
        DOMString::from(self.metadata.filename)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-length>
    fn Length(&self) -> u32 {
        self.metadata.mime_types.len() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-item>
    fn Item(&self, index: u32) -> Option<DomRoot<MimeType>> {
        self.metadata
            .mime_types
            .get(index as usize)
            .map(|metadata| MimeType::new(&self.global(), *metadata, CanGc::deprecated_note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-nameditem>
    fn NamedItem(&self, name: DOMString) -> Option<DomRoot<MimeType>> {
        let name = name.str();
        self.metadata
            .mime_types
            .iter()
            .copied()
            .find(|metadata| metadata.type_ == &*name)
            .map(|metadata| MimeType::new(&self.global(), metadata, CanGc::deprecated_note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-plugin-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<MimeType>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DomRoot<MimeType>> {
        self.NamedItem(name)
    }

    /// <https://heycam.github.io/webidl/#dfn-supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        self.metadata
            .mime_types
            .iter()
            .map(|metadata| DOMString::from(metadata.type_))
            .collect()
    }
}
