#[cfg(not(any(target_os = "linux")))]
compile_error!("only linux platforms are supported");

#[cfg(target_os = "linux")]
#[path = "linux/mod.rs"]
mod platform;

mod trigger;
pub use trigger::{KeyboardTriggers, Triggers};

#[doc(no_inline)]
pub use keycode::KeyMappingCode;
