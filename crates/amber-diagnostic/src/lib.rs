use crate::diagnostic::Diagnostic;

pub mod diagnostic;
pub mod stream_listener;

pub trait Listener {
    fn report(&mut self, diagnostic: Diagnostic);
}
