use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};

pub fn utc_datetime_as_fixed_offset_datetime(d: DateTime<Utc>) -> DateTime<FixedOffset> {
    d.with_timezone(&FixedOffset::east(0))
    // chrono::FixedOffset::east(0)
    //     .ymd(d.year(), d.month(), d.day())
    //     .and_hms_nano(d.hour(), d.minute(), d.second(), d.nanosecond())
}

pub fn fixed_offset_datetime_as_utc_datetime(d: DateTime<FixedOffset>) -> DateTime<Utc> {
    let nanos_since_epoch = d.timestamp_nanos();
    Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(nanos_since_epoch)
}

#[cfg(test)]
mod tests {
    use crate::transform::util::chrono_utils::*;
    use chrono::{FixedOffset, TimeZone, Utc};

    #[test]
    fn test_utc_date_as_fixed_offset() {
        let utc_date = Utc.ymd(2022, 07, 20).and_hms_milli(10, 1, 1, 123);

        let fo_date = FixedOffset::east(0)
            .ymd(2022, 07, 20)
            .and_hms_milli(10, 1, 1, 123);

        assert_eq!(
            format!("{:?}", fo_date),
            format!("{:?}", utc_datetime_as_fixed_offset_datetime(utc_date))
        );
    }

    #[test]
    fn test_fixed_offset_date_as_utc() {
        let utc_date = Utc.ymd(2022, 07, 20).and_hms_milli(10, 1, 1, 123);

        let fo_date = FixedOffset::east(0)
            .ymd(2022, 07, 20)
            .and_hms_milli(10, 1, 1, 123);

        assert_eq!(
            format!("{:?}", utc_date),
            format!("{:?}", fixed_offset_datetime_as_utc_datetime(fo_date))
        );
    }

    #[test]
    fn test_utc_date_to_fixed_offset() {
        let utc_date = Utc.ymd(2022, 07, 20).and_hms_milli(10, 1, 1, 123);

        // This is the same point in time as above, but with a two hour shift from the timezone
        let fo_date = FixedOffset::east(2 * 3600)
            .ymd(2022, 07, 20)
            .and_hms_milli(12, 1, 1, 123);

        assert_eq!(
            format!("{:?}", utc_date),
            format!("{:?}", fixed_offset_datetime_as_utc_datetime(fo_date))
        );
    }
}
