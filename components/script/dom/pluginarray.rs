/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;

use crate::dom::bindings::codegen::Bindings::PluginArrayBinding::PluginArrayMethods;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::globalscope::GlobalScope;
use crate::dom::plugin::{Plugin, chrome_pdf_plugins, plugin_for_name};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct PluginArray {
    reflector_: Reflector,
}

impl PluginArray {
    pub(crate) fn new_inherited() -> PluginArray {
        PluginArray {
            reflector_: Reflector::new(),
        }
    }

    pub(crate) fn new(global: &GlobalScope, can_gc: CanGc) -> DomRoot<PluginArray> {
        reflect_dom_object(Box::new(PluginArray::new_inherited()), global, can_gc)
    }
}

impl PluginArrayMethods<crate::DomTypeHolder> for PluginArray {
    /// <https://html.spec.whatwg.org/multipage/#dom-pluginarray-refresh>
    fn Refresh(&self, _reload: bool) {}

    /// <https://html.spec.whatwg.org/multipage/#dom-pluginarray-length>
    fn Length(&self) -> u32 {
        chrome_pdf_plugins().len() as u32
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-pluginarray-item>
    fn Item(&self, index: u32) -> Option<DomRoot<Plugin>> {
        chrome_pdf_plugins()
            .get(index as usize)
            .copied()
            .map(|plugin| Plugin::new(&self.global(), plugin, CanGc::deprecated_note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-pluginarray-nameditem>
    fn NamedItem(&self, name: DOMString) -> Option<DomRoot<Plugin>> {
        plugin_for_name(&name.str())
            .map(|plugin| Plugin::new(&self.global(), plugin, CanGc::deprecated_note()))
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-pluginarray-item>
    fn IndexedGetter(&self, index: u32) -> Option<DomRoot<Plugin>> {
        self.Item(index)
    }

    // check-tidy: no specs after this line
    fn NamedGetter(&self, name: DOMString) -> Option<DomRoot<Plugin>> {
        self.NamedItem(name)
    }

    /// <https://heycam.github.io/webidl/#dfn-supported-property-names>
    fn SupportedPropertyNames(&self) -> Vec<DOMString> {
        chrome_pdf_plugins()
            .iter()
            .map(|plugin| DOMString::from(plugin.name))
            .collect()
    }
}
