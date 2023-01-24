use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};

pub fn utc_datetime_as_fixed_offset_datetime(d: DateTime<Utc>) -> DateTime<FixedOffset> {
    d.with_timezone(&FixedOffset::east_opt(0).unwrap())
}

pub fn fixed_offset_datetime_as_utc_datetime(d: DateTime<FixedOffset>) -> DateTime<Utc> {
    let nanos_since_epoch = d.timestamp_nanos();
    Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap() + Duration::nanoseconds(nanos_since_epoch)
}

#[cfg(test)]
mod tests {
    use crate::transform::util::chrono_utils::*;
    use chrono::{FixedOffset, NaiveDate, Utc};

    #[test]
    fn utc_date_as_fixed_offset() {
        let utc_date = NaiveDate::from_ymd_opt(2022, 7, 20)
            .unwrap() // This date exists for sure. Unwrap is safe here
            .and_hms_milli_opt(10, 1, 1, 123)
            .unwrap() // This time exists for sure. Unwrap is safe here
            .and_local_timezone(Utc)
            .unwrap(); // This timezone (UTC) exists for sure. Unwrap is safe here

        let fo_date = NaiveDate::from_ymd_opt(2022, 7, 20)
            .unwrap() // This date exists for sure. Unwrap is safe here
            .and_hms_milli_opt(10, 1, 1, 123)
            .unwrap() // This time exists for sure. Unwrap is safe here
            .and_local_timezone(FixedOffset::east_opt(0).unwrap())
            .unwrap(); // This timezone (UTC) exists for sure. Unwrap is safe here

        assert_eq!(
            format!("{:?}", fo_date),
            format!("{:?}", utc_datetime_as_fixed_offset_datetime(utc_date))
        );
    }

    #[test]
    fn fixed_offset_date_as_utc() {
        let utc_date = NaiveDate::from_ymd_opt(2022, 7, 20)
            .unwrap() // This date exists for sure. Unwrap is safe here
            .and_hms_milli_opt(10, 1, 1, 123)
            .unwrap() // This time exists for sure. Unwrap is safe here
            .and_local_timezone(Utc)
            .unwrap(); // This timezone (UTC) exists for sure. Unwrap is safe here

        let fo_date = NaiveDate::from_ymd_opt(2022, 7, 20)
            .unwrap() // This date exists for sure. Unwrap is safe here
            .and_hms_milli_opt(10, 1, 1, 123)
            .unwrap() // This time exists for sure. Unwrap is safe here
            .and_local_timezone(FixedOffset::east_opt(0).unwrap())
            .unwrap(); // This timezone (UTC) exists for sure. Unwrap is safe here

        assert_eq!(
            format!("{:?}", utc_date),
            format!("{:?}", fixed_offset_datetime_as_utc_datetime(fo_date))
        );
    }
}
