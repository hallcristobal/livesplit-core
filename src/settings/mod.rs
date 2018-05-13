//! The settings module provides the ability to customize components and various
//! other settings.

mod alignment;
mod alternating;
mod color;
mod field;
mod gradient;
mod semantic_color;
mod settings_description;
mod value;

pub use self::alignment::Alignment;
pub use self::alternating::Alternating;
pub use self::color::Color;
pub use self::field::Field;
pub use self::gradient::Gradient;
pub use self::semantic_color::SemanticColor;
pub use self::settings_description::SettingsDescription;
pub use self::value::{Error as ValueError, Result as ValueResult, Value};
