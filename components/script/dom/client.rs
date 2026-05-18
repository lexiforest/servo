/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::default::Default;

use dom_struct::dom_struct;
use js::jsapi::{Heap, JSObject};
use js::rust::{CustomAutoRooter, CustomAutoRooterGuard, HandleValue};
use servo_base::generic_channel::GenericCallback;
use servo_constellation_traits::ClientDOMMessage;
use servo_url::ServoUrl;
use uuid::Uuid;

use crate::dom::bindings::codegen::Bindings::MessagePortBinding::StructuredSerializeOptions;
use crate::dom::bindings::codegen::Bindings::ClientBinding::{ClientMethods, FrameType};
use crate::dom::bindings::error::ErrorResult;
use crate::dom::bindings::reflector::{DomGlobal, Reflector, reflect_dom_object};
use crate::dom::bindings::root::{DomRoot, MutNullableDom};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::bindings::structuredclone;
use crate::dom::bindings::trace::RootedTraceableBox;
use crate::dom::globalscope::GlobalScope;
use crate::dom::serviceworker::ServiceWorker;
use crate::dom::window::Window;
use crate::script_runtime::{CanGc, JSContext};

#[dom_struct]
pub(crate) struct Client {
    reflector_: Reflector,
    active_worker: MutNullableDom<ServiceWorker>,
    #[no_trace]
    url: ServoUrl,
    #[no_trace]
    message_sender: Option<GenericCallback<ClientDOMMessage>>,
    frame_type: FrameType,
    #[no_trace]
    id: Uuid,
}

impl Client {
    fn new_inherited(
        url: ServoUrl,
        message_sender: Option<GenericCallback<ClientDOMMessage>>,
    ) -> Client {
        Client {
            reflector_: Reflector::new(),
            active_worker: Default::default(),
            url,
            message_sender,
            frame_type: FrameType::None,
            id: Uuid::new_v4(),
        }
    }

    pub(crate) fn new(window: &Window, can_gc: CanGc) -> DomRoot<Client> {
        reflect_dom_object(
            Box::new(Client::new_inherited(window.get_url(), None)),
            window,
            can_gc,
        )
    }

    pub(crate) fn new_for_serviceworker(
        global: &GlobalScope,
        url: ServoUrl,
        message_sender: Option<GenericCallback<ClientDOMMessage>>,
        can_gc: CanGc,
    ) -> DomRoot<Client> {
        reflect_dom_object(
            Box::new(Client::new_inherited(url, message_sender)),
            global,
            can_gc,
        )
    }

    pub(crate) fn creation_url(&self) -> ServoUrl {
        self.url.clone()
    }

    pub(crate) fn get_controller(&self) -> Option<DomRoot<ServiceWorker>> {
        self.active_worker.get()
    }

    #[expect(dead_code)]
    pub(crate) fn set_controller(&self, worker: &ServiceWorker) {
        self.active_worker.set(Some(worker));
    }
}

impl ClientMethods<crate::DomTypeHolder> for Client {
    /// <https://w3c.github.io/ServiceWorker/#client-url-attribute>
    fn Url(&self) -> USVString {
        USVString(self.url.as_str().to_owned())
    }

    /// <https://w3c.github.io/ServiceWorker/#client-frametype>
    fn FrameType(&self) -> FrameType {
        self.frame_type
    }

    /// <https://w3c.github.io/ServiceWorker/#client-id>
    fn Id(&self) -> DOMString {
        format!("{}", self.id).into()
    }

    fn PostMessage(
        &self,
        cx: JSContext,
        message: HandleValue,
        options: RootedTraceableBox<StructuredSerializeOptions>,
    ) -> ErrorResult {
        let mut rooted = CustomAutoRooter::new(
            options
                .transfer
                .iter()
                .map(|js: &RootedTraceableBox<Heap<*mut JSObject>>| js.get())
                .collect(),
        );
        let transfer = CustomAutoRooterGuard::new(cx.raw_cx(), &mut rooted);
        let data = structuredclone::write(cx, message, Some(transfer))?;
        if let Some(sender) = &self.message_sender {
            let global = self.global();
            let _ = sender.send(ClientDOMMessage {
                origin: global.origin().immutable().clone(),
                data,
            });
        }
        Ok(())
    }
}
