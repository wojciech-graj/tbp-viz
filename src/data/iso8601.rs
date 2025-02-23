#![allow(clippy::ref_option)]

use serde::Deserialize;
use time::Date;

time::serde::format_description!(iso8601, Date, "[year]-[month]-[day]");

#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy, Deserialize)]
#[serde(transparent)]
pub struct Iso8601Date(#[serde(with = "iso8601")] pub Date);
