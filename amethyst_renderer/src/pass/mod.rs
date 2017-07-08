//! Different kinds of render passes.

pub use self::blit::BlitBuffer;
pub use self::clear::ClearTarget;
pub use self::flat::DrawFlat;

mod blit;
mod clear;
mod flat;