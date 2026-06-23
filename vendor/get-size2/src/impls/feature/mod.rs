#![allow(clippy::allow_attributes, reason = "needed if features are enabled")]

#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "chrono")]
mod chrono;
#[cfg(feature = "chrono-tz")]
mod chrono_tz;
#[cfg(feature = "compact-str")]
mod compact_str;
#[cfg(feature = "hashbrown")]
mod hashbrown;
#[cfg(feature = "indexmap")]
mod indexmap;
#[cfg(feature = "ordermap")]
mod ordermap;
#[cfg(feature = "smallvec")]
mod smallvec;
#[cfg(feature = "thin-vec")]
mod thin_vec;
#[cfg(feature = "url")]
mod url;
