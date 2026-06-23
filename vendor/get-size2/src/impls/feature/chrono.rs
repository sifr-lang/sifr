use crate::{GetSize, GetSizeTracker};

impl GetSize for chrono::NaiveDate {}
impl GetSize for chrono::NaiveTime {}
impl GetSize for chrono::NaiveDateTime {}
impl GetSize for chrono::Utc {}
impl GetSize for chrono::FixedOffset {}
impl GetSize for chrono::TimeDelta {}

impl<Tz: chrono::TimeZone> GetSize for chrono::DateTime<Tz>
where
    Tz::Offset: GetSize,
{
    fn get_heap_size_with_tracker<Tr: GetSizeTracker>(&self, tracker: Tr) -> (usize, Tr) {
        <Tz::Offset>::get_heap_size_with_tracker(self.offset(), tracker)
    }
}
