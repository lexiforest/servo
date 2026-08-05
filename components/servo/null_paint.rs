/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crossbeam_channel::Sender;
use dpi::PhysicalSize;
use embedder_traits::{
    EventLoopWaker, InputEventAndId, InputEventId, InputEventResult, ScreenshotCaptureError,
    Scroll, ShutdownState, ViewportDetails, WebViewPoint, WebViewRect,
};
use euclid::Scale;
use image::RgbaImage;
use paint_api::rendering_context::RenderingContext;
use paint_api::{PaintMessage, WebRenderExternalImageIdManager, WebViewTrait};
use profile_traits::{mem, time};
use servo_base::generic_channel::RoutedReceiver;
use servo_base::id::{PainterId, PipelineId, WebViewId};
use servo_constellation_traits::EmbedderToConstellationMessage;
use servo_geometry::DeviceIndependentPixel;
use style_traits::CSSPixel;
use webrender_api::units::{DevicePixel, DevicePoint};
use webrender_api::{FontInstanceKey, FontKey, ImageKey};

/// Data used to initialize the null paint subsystem.
pub struct InitialPaintState {
    /// A port on which messages inbound to paint can be received.
    pub receiver: RoutedReceiver<PaintMessage>,
    /// A channel to the constellation.
    pub embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,
    /// A channel to the time profiler thread.
    pub time_profiler_chan: time::ProfilerChan,
    /// A channel to the memory profiler thread.
    pub mem_profiler_chan: mem::ProfilerChan,
    /// A shared state which tracks whether Servo has started or has finished
    /// shutting down.
    pub shutdown_state: Rc<Cell<ShutdownState>>,
    /// An [`EventLoopWaker`] kept for parity with the rendering paint state.
    pub event_loop_waker: Box<dyn EventLoopWaker>,
    pub paint_proxy: paint_api::PaintProxy,
}

/// A compile-time no-render replacement for the `paint` crate.
pub struct Paint {
    paint_receiver: RoutedReceiver<PaintMessage>,
    embedder_to_constellation_sender: Sender<EmbedderToConstellationMessage>,
    time_profiler_chan: time::ProfilerChan,
    shutdown_state: Rc<Cell<ShutdownState>>,
    webrender_external_image_id_manager: WebRenderExternalImageIdManager,
    _event_loop_waker: Box<dyn EventLoopWaker>,
    _mem_profiler_registration: profile_traits::mem::ProfilerRegistration,
}

impl Paint {
    pub fn new(state: InitialPaintState) -> Rc<RefCell<Self>> {
        let registration = state.mem_profiler_chan.prepare_memory_reporting(
            "paint".into(),
            state.paint_proxy.clone(),
            PaintMessage::CollectMemoryReport,
        );

        Rc::new(RefCell::new(Paint {
            paint_receiver: state.receiver,
            embedder_to_constellation_sender: state.embedder_to_constellation_sender,
            time_profiler_chan: state.time_profiler_chan,
            shutdown_state: state.shutdown_state,
            webrender_external_image_id_manager: WebRenderExternalImageIdManager::default(),
            _event_loop_waker: state.event_loop_waker,
            _mem_profiler_registration: registration,
        }))
    }

    pub fn register_rendering_context(
        &mut self,
        _rendering_context: Rc<dyn RenderingContext>,
    ) -> PainterId {
        PainterId::next()
    }

    pub fn remove_webview(&mut self, _webview_id: WebViewId) {}

    pub fn add_webview(&self, _webview: Box<dyn WebViewTrait>, _viewport_details: ViewportDetails) {
    }

    pub fn show_webview(&self, _webview_id: WebViewId) -> Result<(), ()> {
        Ok(())
    }

    pub fn hide_webview(&self, _webview_id: WebViewId) -> Result<(), ()> {
        Ok(())
    }

    pub fn set_hidpi_scale_factor(
        &self,
        _webview_id: WebViewId,
        _new_scale_factor: Scale<f32, DeviceIndependentPixel, DevicePixel>,
    ) {
    }

    pub fn resize_rendering_context(&self, _webview_id: WebViewId, _new_size: PhysicalSize<u32>) {}

    pub fn set_page_zoom(&self, _webview_id: WebViewId, _new_zoom: f32) {}

    pub fn page_zoom(&self, _webview_id: WebViewId) -> f32 {
        1.0
    }

    pub fn adjust_pinch_zoom(
        &self,
        _webview_id: WebViewId,
        _pinch_zoom_delta: f32,
        _center: DevicePoint,
    ) {
    }

    pub fn pinch_zoom(&self, _webview_id: WebViewId) -> f32 {
        1.0
    }

    pub fn device_pixels_per_page_pixel(
        &self,
        _webview_id: WebViewId,
    ) -> Scale<f32, CSSPixel, DevicePixel> {
        Scale::new(1.0)
    }

    pub fn render(&self, _webview_id: WebViewId) {}

    pub fn receiver(&self) -> &RoutedReceiver<PaintMessage> {
        &self.paint_receiver
    }

    pub fn handle_messages(&self, messages: Vec<PaintMessage>) {
        for message in messages {
            self.handle_message(message);
        }
    }

    pub fn perform_updates(&self) -> bool {
        self.shutdown_state.get() != ShutdownState::FinishedShuttingDown
    }

    pub fn webviews_needing_repaint(&self) -> Vec<WebViewId> {
        Vec::new()
    }

    pub fn finish_shutting_down(&self) {
        while self.paint_receiver.try_recv().is_ok() {}

        if let Ok((sender, receiver)) = ipc_channel::ipc::channel() {
            self.time_profiler_chan
                .send(profile_traits::time::ProfilerMsg::Exit(sender));
            let _ = receiver.recv();
        }
    }

    pub fn notify_input_event(&self, _webview_id: WebViewId, _event: InputEventAndId) -> bool {
        false
    }

    pub fn notify_scroll_event(
        &self,
        _webview_id: WebViewId,
        _scroll: Scroll,
        _point: WebViewPoint,
    ) {
    }

    pub fn notify_input_event_handled(
        &self,
        _webview_id: WebViewId,
        _input_event_id: InputEventId,
        _result: InputEventResult,
    ) {
    }

    pub fn request_screenshot(
        &self,
        _webview_id: WebViewId,
        _rect: Option<WebViewRect>,
        callback: Box<dyn FnOnce(Result<RgbaImage, ScreenshotCaptureError>) + 'static>,
    ) {
        callback(Err(ScreenshotCaptureError::CouldNotReadImage));
    }

    pub fn capture_webrender(&self, _webview_id: WebViewId) {}

    pub fn toggle_webrender_debug(&self, _option: crate::WebRenderDebugOption) {}

    pub fn webrender_external_image_id_manager(&self) -> WebRenderExternalImageIdManager {
        self.webrender_external_image_id_manager.clone()
    }

    fn handle_message(&self, message: PaintMessage) {
        match message {
            PaintMessage::GenerateImageKey(webview_id, result_sender) => {
                let _ = result_sender.send(dummy_image_key(webview_id.into()));
            },
            PaintMessage::GenerateImageKeysForPipeline(webview_id, pipeline_id) => {
                self.send_dummy_image_keys(webview_id, pipeline_id);
            },
            PaintMessage::GenerateFontKeys(
                number_of_font_keys,
                number_of_font_instance_keys,
                result_sender,
                painter_id,
            ) => {
                let font_keys = (0..number_of_font_keys)
                    .map(|_| dummy_font_key(painter_id))
                    .collect();
                let font_instance_keys = (0..number_of_font_instance_keys)
                    .map(|_| dummy_font_instance_key(painter_id))
                    .collect();
                let _ = result_sender.send((font_keys, font_instance_keys));
            },
            PaintMessage::CollectMemoryReport(sender) => {
                sender.send(profile_traits::mem::ProcessReports::new(Vec::new()));
            },
            _ => {},
        }
    }

    fn send_dummy_image_keys(&self, webview_id: WebViewId, pipeline_id: PipelineId) {
        let painter_id = webview_id.into();
        let image_keys = (0..servo_config::pref!(image_key_batch_size))
            .map(|_| dummy_image_key(painter_id))
            .collect();
        let _ = self.embedder_to_constellation_sender.send(
            EmbedderToConstellationMessage::SendImageKeysForPipeline(pipeline_id, image_keys),
        );
    }
}

fn dummy_image_key(painter_id: PainterId) -> ImageKey {
    ImageKey::new(painter_id.into(), 0)
}

fn dummy_font_key(painter_id: PainterId) -> FontKey {
    FontKey::new(painter_id.into(), 0)
}

fn dummy_font_instance_key(painter_id: PainterId) -> FontInstanceKey {
    FontInstanceKey::new(painter_id.into(), 0)
}
