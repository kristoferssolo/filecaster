pub use filecaster_derive::FromFile;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Marker for types that can be built from an [`Option<Shadow>`] produced by the macro.
pub trait FromFile: Sized {
    type Shadow: Default;
    fn from_file(file: Option<Self::Shadow>) -> Self;
}

#[cfg(not(feature = "serde"))]
impl<T> FromFile for T
where
    T: Default,
{
    type Shadow = T;
    fn from_file(file: Option<Self>) -> Self {
        file.unwrap_or_default()
    }
}

#[cfg(feature = "serde")]
impl<T> FromFile for T
where
    T: Default + Serialize + for<'de> Deserialize<'de>,
{
    type Shadow = T;
    fn from_file(file: Option<Self>) -> Self {
        file.unwrap_or_default()
    }
}
