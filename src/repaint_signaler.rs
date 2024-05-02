use std::sync::Mutex;

use epi::RepaintSignal;
use winit::event_loop::EventLoopProxy;

/// Used by Epi (backend agnostic interface for egui) to signal a repaint.
pub struct RepaintSignaler(pub Mutex<EventLoopProxy<()>>);

impl RepaintSignal for RepaintSignaler {
    fn request_repaint(&self) {
        self.0.lock().unwrap().send_event(()).ok();
    }
}
