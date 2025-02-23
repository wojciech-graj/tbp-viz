use std::ops::Range;

use plotters::{
    coord::ranged1d::{KeyPointHint, NoDefaultFormatting, ValueFormatter},
    prelude::Ranged,
};
use time::OffsetDateTime;

#[derive(Debug)]
pub struct OffsetDateTimeRange {
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
}

impl Ranged for OffsetDateTimeRange {
    type FormatOption = NoDefaultFormatting;

    type ValueType = OffsetDateTime;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        limit.0
            + ((*value - self.start) / (self.end - self.start) * f64::from(limit.1 - limit.0))
                as i32
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        (0..(hint.max_num_points()))
            .map(|i| {
                self.start + (self.end - self.start) / (hint.max_num_points() - 1) as u32 * i as u32
            })
            .collect()
    }

    fn range(&self) -> Range<Self::ValueType> {
        self.start..self.end
    }
}

impl ValueFormatter<OffsetDateTime> for OffsetDateTimeRange {
    fn format(value: &OffsetDateTime) -> String {
        format!("{}", value.year())
    }

    fn format_ext(&self, value: &OffsetDateTime) -> String {
        Self::format(value)
    }
}
