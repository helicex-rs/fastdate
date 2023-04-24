use crate::error::Error;
use crate::sys::Timespec;
use crate::{Date, Time};
use once_cell::sync::Lazy;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Deref, Sub};
use std::str::FromStr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Obtain the offset of Utc time and Local time in seconds, using Lazy only once to improve performance
pub static GLOBAL_OFFSET: Lazy<i32> = Lazy::new(|| Timespec::now().local().tm_utcoff);

/// offset with utc 0.zone
pub fn offset_sec() -> i32 {
    GLOBAL_OFFSET.deref().clone()
}

/// Log timestamp type.
///
/// Parse using `FromStr` impl.
/// Format using the `Display` trait.
/// Convert timestamp into/from `SytemTime` to use.
/// Supports comparsion and sorting.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct DateTime {
    /// 0...999999999
    pub nano: u32,
    /// 0...59
    pub sec: u8,
    /// 0...59
    pub min: u8,
    /// 0...23
    pub hour: u8,
    /// 1...31
    pub day: u8,
    /// 1...12
    pub mon: u8,
    /// 1970...9999
    pub year: u16,
}

impl DateTime {
    ///utc time
    pub fn utc() -> Self {
        Self::from(SystemTime::now())
    }
    ///local zone time
    pub fn now() -> Self {
        let offset = GLOBAL_OFFSET.deref().clone();
        if offset > 0 {
            Self::from(SystemTime::now() + Duration::from_secs(offset as u64))
        } else {
            Self::from(SystemTime::now() - Duration::from_secs(offset.abs() as u64))
        }
    }

    /// set offset
    /// ```rust
    /// let mut  dt = fastdate::DateTime::utc();
    /// dt = dt.set_offset(fastdate::offset_sec());
    /// ```
    pub fn set_offset(self, offset_sec: i32) -> DateTime {
        let time: SystemTime = self.into();
        if offset_sec > 0 {
            Self::from(time + Duration::from_secs(offset_sec as u64))
        } else {
            Self::from(time - Duration::from_secs(offset_sec.abs() as u64))
        }
    }

    /// add Duration
    pub fn add(self, d: Duration) -> Self {
        Self::from(SystemTime::from(self) + d)
    }

    /// sub Duration
    pub fn sub(self, d: Duration) -> Self {
        Self::from(SystemTime::from(self) - d)
    }

    ///add/sub sec
    pub fn add_sub_sec(self, sec: i64) -> Self {
        if sec > 0 {
            self.add(Duration::from_secs(sec as u64))
        } else {
            self.sub(Duration::from_secs((-sec) as u64))
        }
    }

    /// is self before on other?
    pub fn before(&self, other: &DateTime) -> bool {
        self < other
    }

    /// is self after on other?
    pub fn after(&self, other: &DateTime) -> bool {
        self > other
    }

    /// unix_timestamp sec
    pub fn unix_timestamp(&self) -> i64 {
        if self.year >= 1970 {
            SystemTime::from(self.clone())
                .duration_since(UNIX_EPOCH)
                .expect("duration_since fail").as_secs() as i64
        } else {
            -(UNIX_EPOCH.duration_since(SystemTime::from(self.clone()))
                .expect("duration_since fail").as_secs() as i64)
        }
    }

    ///unix_timestamp micros
    pub fn unix_timestamp_micros(&self) -> i64 {
        if self.year >= 1970 {
            SystemTime::from(self.clone())
                .duration_since(UNIX_EPOCH)
                .expect("duration_since fail").as_micros() as i64
        } else {
            -(UNIX_EPOCH.duration_since(SystemTime::from(self.clone()))
                .expect("duration_since fail").as_micros() as i64)
        }
    }

    ///unix_timestamp millis
    pub fn unix_timestamp_millis(&self) -> i64 {
        if self.year >= 1970 {
            SystemTime::from(self.clone())
                .duration_since(UNIX_EPOCH)
                .expect("duration_since fail").as_millis() as i64
        } else {
            -(UNIX_EPOCH
                .duration_since(SystemTime::from(self.clone()))
                .expect("duration_since fail").as_millis() as i64)
        }
    }

    ///unix_timestamp nano
    pub fn unix_timestamp_nano(&self) -> i128 {
        if self.year >= 1970 {
            SystemTime::from(self.clone())
                .duration_since(UNIX_EPOCH)
                .expect("duration_since fail")
                .as_nanos() as i128
        } else {
            -(UNIX_EPOCH.duration_since(SystemTime::from(self.clone()))
                .expect("duration_since fail")
                .as_nanos() as i128)
        }
    }

    ///from timestamp sec
    pub fn from_timestamp(sec: i64) -> DateTime {
        if sec > 0 {
            Self::from(UNIX_EPOCH + Duration::from_secs(sec as u64))
        } else {
            Self::from(UNIX_EPOCH - Duration::from_secs((-sec) as u64))
        }
    }
    ///from timestamp micros
    pub fn from_timestamp_micros(micros: i64) -> DateTime {
        if micros > 0 {
            Self::from(UNIX_EPOCH + Duration::from_micros(micros as u64))
        } else {
            Self::from(UNIX_EPOCH - Duration::from_micros((-micros) as u64))
        }
    }
    ///from timestamp millis
    pub fn from_timestamp_millis(ms: i64) -> DateTime {
        if ms > 0 {
            Self::from(UNIX_EPOCH + Duration::from_millis(ms as u64))
        } else {
            Self::from(UNIX_EPOCH - Duration::from_millis((-ms) as u64))
        }
    }
    ///from timestamp nano
    pub fn from_timestamp_nano(nano: i128) -> DateTime {
        if nano > 0 {
            Self::from(UNIX_EPOCH + Duration::from_nanos(nano as u64))
        } else {
            Self::from(UNIX_EPOCH - Duration::from_nanos((-nano) as u64))
        }
    }

    /// parse an string by format.
    /// format str must be:
    /// ```rust
    ///  fastdate::DateTime::parse("YYYY-MM-DD hh:mm:ss.000000","2022-12-13 11:12:14.123456").unwrap();
    /// ```
    /// or any position
    /// ```rust
    ///  fastdate::DateTime::parse("hh:mm:ss.000000,YYYY-MM-DD","11:12:14.123456,2022-12-13").unwrap();
    /// ```
    pub fn parse(format: &str, arg: &str) -> Result<DateTime, Error> {
        let bytes = arg.as_bytes();
        let mut buf: [u8; 26] = *b"0000-00-00 00:00:00.000000";
        let format_bytes = format.as_bytes();
        let mut idx_year = 0;
        let mut idx_mon = 5;
        let mut idx_day = 8;
        let mut idx_hour = 11;
        let mut idx_min = 14;
        let mut idx_sec = 17;
        let mut idx_micro = 20;
        let mut v = 0;
        for x in format_bytes {
            if x == &('Y' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong YYYY format!"));
                }
                buf[idx_year] = bytes[v];
                idx_year += 1;
            }
            if x == &('M' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong MM format!"));
                }
                buf[idx_mon] = bytes[v];
                idx_mon += 1;
            }
            if x == &('D' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong DD format!"));
                }
                buf[idx_day] = bytes[v];
                idx_day += 1;
            }
            if x == &('h' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong hh format!"));
                }
                buf[idx_hour] = bytes[v];
                idx_hour += 1;
            }
            if x == &('m' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong mm format!"));
                }
                buf[idx_min] = bytes[v];
                idx_min += 1;
            }
            if x == &('s' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong ss format!"));
                }
                buf[idx_sec] = bytes[v];
                idx_sec += 1;
            }
            if x == &('0' as u8) {
                if v >= bytes.len() {
                    return Err(Error::from("wrong .000000 format!"));
                }
                buf[idx_micro] = bytes[v];
                idx_micro += 1;
            }
            v += 1;
        }
        DateTime::from_str(std::str::from_utf8(&buf[..]).unwrap_or_default())
    }

    /// get week_day
    pub fn week_day(&self) -> u8 {
        let secs_since_epoch = self.unix_timestamp();
        /* 2000-03-01 (mod 400 year, immediately after feb29 */
        const LEAPOCH: i64 = 11017;
        let days = (secs_since_epoch / 86400) as i64 - LEAPOCH;
        let mut wday = (3 + days) % 7;
        if wday <= 0 {
            wday += 7
        };
        wday as u8
    }
    /// 0...999999999
    pub fn set_nano(mut self, arg: u32) -> Self {
        self.nano = arg;
        self
    }
    /// get nano 0...999999999
    pub fn get_nano(&self) -> u32 {
        self.nano
    }
    /// 0...999999
    pub fn set_micro(mut self, arg: u32) -> Self {
        self.nano = arg * 1000;
        self
    }
    /// 0...59
    pub fn set_sec(mut self, arg: u8) -> Self {
        self.sec = arg;
        self
    }
    /// 0...59
    pub fn set_min(mut self, arg: u8) -> Self {
        self.min = arg;
        self
    }
    /// 0...23
    pub fn set_hour(mut self, arg: u8) -> Self {
        self.hour = arg;
        self
    }
    /// 1...31
    pub fn set_day(mut self, arg: u8) -> Self {
        self.day = arg;
        self
    }
    /// 1...12
    pub fn set_mon(mut self, arg: u8) -> Self {
        self.mon = arg;
        self
    }
    /// 1970...9999
    pub fn set_year(mut self, arg: u16) -> Self {
        self.year = arg;
        self
    }

    /// get micro secs
    pub fn get_micro(&self) -> u32 {
        self.nano / 1000
    }

    /// get sec
    pub fn get_sec(&self) -> u8 {
        self.sec
    }

    /// get sec
    pub fn get_min(&self) -> u8 {
        self.min
    }

    /// get hour
    pub fn get_hour(&self) -> u8 {
        self.hour
    }

    /// get day
    pub fn get_day(&self) -> u8 {
        self.day
    }

    /// get mon
    pub fn get_mon(&self) -> u8 {
        self.mon
    }

    /// get year
    pub fn get_year(&self) -> u16 {
        self.year
    }
}

impl Add<Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: Duration) -> Self::Output {
        self.add(rhs)
    }
}

impl Sub<Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: Duration) -> Self::Output {
        self.sub(rhs)
    }
}

impl Add<&Duration> for DateTime {
    type Output = DateTime;

    fn add(self, rhs: &Duration) -> Self::Output {
        self.add(rhs.clone())
    }
}

impl Sub<&Duration> for DateTime {
    type Output = DateTime;

    fn sub(self, rhs: &Duration) -> Self::Output {
        self.sub(rhs.clone())
    }
}

impl Sub<DateTime> for DateTime {
    type Output = Duration;

    fn sub(self, rhs: DateTime) -> Self::Output {
        let nano = self.unix_timestamp_nano() - rhs.unix_timestamp_nano();
        Duration::from_nanos(nano as u64)
    }
}

impl From<SystemTime> for DateTime {
    fn from(v: SystemTime) -> DateTime {
        DateTime::from_system_time(v)
    }
}

impl From<DateTime> for SystemTime {
    fn from(v: DateTime) -> SystemTime {
        let mut r = UNIX_EPOCH;
        let mut years = 0;
        if v.year >= 1970 {
            years = v.year - 1970;
        } else {
            years = 1970 - v.year;
        }
        if years > 1 {
            for idx in 0..years {
                let y;
                if v.year >= 1970 {
                    y = 1970 + (idx + 1);
                    if is_leap_year(y) {
                        r = r + Duration::from_secs(366 * 24 * 3600);
                    } else {
                        r = r + Duration::from_secs(365 * 24 * 3600);
                    }
                } else {
                    y = 1970 - (idx + 1);
                    if is_leap_year(y) {
                        r = r - Duration::from_secs(366 * 24 * 3600);
                    } else {
                        r = r - Duration::from_secs(365 * 24 * 3600);
                    }
                }
            }
        }
        let years;
        if is_leap_year(v.year) {
            years = LEAP_YEAR;
        } else {
            years = DEFAULT_YEAR;
        }
        let mut mon = 1;
        for i in years {
            if mon == v.mon {
                if v.day > 0 {
                    r = r + Duration::from_secs((v.day - 1) as u64 * 24 * 3600);
                }
                break;
            }
            r = r + Duration::from_secs(i * 24 * 3600);
            mon += 1;
        }
        r = r + Duration::from_secs(v.hour as u64 * 3600);
        r = r + Duration::from_secs(v.min as u64 * 60);
        r = r + Duration::from_secs(v.sec as u64);
        r = r + Duration::from_nanos(v.nano as u64);
        r
    }
}

impl From<Date> for DateTime {
    fn from(arg: Date) -> Self {
        Self {
            nano: 0,
            sec: 0,
            min: 0,
            hour: 0,
            day: arg.day,
            mon: arg.mon,
            year: arg.year,
        }
    }
}

impl From<Time> for DateTime {
    fn from(arg: Time) -> Self {
        Self {
            nano: arg.nano,
            sec: arg.sec,
            min: arg.min,
            hour: arg.hour,
            day: 0,
            mon: 0,
            year: 0,
        }
    }
}

impl FromStr for DateTime {
    type Err = Error;

    /// from RFC3339Nano = "0000-00-00 00:00:00.000000000"
    fn from_str(s: &str) -> Result<DateTime, Error> {
        let bytes = s.as_bytes();
        let mut date = DateTime {
            nano: 0,
            sec: 0,
            min: 0,
            hour: 0,
            day: 0,
            mon: 0,
            year: 0,
        };
        if bytes.len() >= 10 {
            let d = Date::parse_bytes_partial(&bytes)?;
            date.year = d.year;
            date.mon = d.mon;
            date.day = d.day;

            let (t, offset) = Time::parse_bytes_partial(&bytes, 11)?;
            date.hour = t.hour;
            date.min = t.min;
            date.sec = t.sec;
            date.nano = t.nano;
            let start = 11 + offset;
            //+09:00
            let mut offset_sec = 0;
            if s.len() > start {
                let remin_str = &s[start..];
                let remin_bytes = remin_str.as_bytes();
                if remin_str.len() == 3 {
                    if remin_bytes[0] == b'+' || remin_bytes[0] == b'Z' {
                        offset_sec += ((remin_bytes[1] - b'0') as i32 * 10
                            + (remin_bytes[2] - b'0') as i32)
                            * 3600;
                    } else if remin_bytes[0] == b'-' {
                        offset_sec -= ((remin_bytes[1] - b'0') as i32 * 10
                            + (remin_bytes[2] - b'0') as i32)
                            * 3600;
                    }
                } else if remin_str.len() == 6 {
                    if remin_bytes[0] == b'+' || remin_bytes[0] == b'Z' {
                        //hour
                        offset_sec += ((remin_bytes[1] - b'0') as i32 * 10
                            + (remin_bytes[2] - b'0') as i32)
                            * 3600;
                        //min
                        offset_sec += ((remin_bytes[4] - b'0') as i32 * 10
                            + (remin_bytes[5] - b'0') as i32)
                            * 60;
                    } else if remin_bytes[0] == b'-' {
                        //hour
                        offset_sec -= ((remin_bytes[1] - b'0') as i32 * 10
                            + (remin_bytes[2] - b'0') as i32)
                            * 3600;
                        //min
                        offset_sec -= ((remin_bytes[4] - b'0') as i32 * 10
                            + (remin_bytes[5] - b'0') as i32)
                            * 60;
                    }
                }
            }
            if offset_sec > 0 {
                date = date.add(Duration::from_secs(offset_sec as u64));
            } else if offset_sec < 0 {
                date = date.sub(Duration::from_secs(offset_sec.abs() as u64));
            }
            if bytes[bytes.len() - 1] == 'Z' as u8 {
                date = date.set_offset(crate::offset_sec()); //append offset
            }
        }
        Ok(date)
    }
}

impl Display for DateTime {
    /// fmt RFC3339Nano = "2006-01-02T15:04:05.999999999"
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut buf: [u8; 29] = *b"0000-00-00 00:00:00.000000000";
        buf[0] = b'0' + (self.year / 1000) as u8;
        buf[1] = b'0' + (self.year / 100 % 10) as u8;
        buf[2] = b'0' + (self.year / 10 % 10) as u8;
        buf[3] = b'0' + (self.year % 10) as u8;
        buf[5] = b'0' + (self.mon / 10) as u8;
        buf[6] = b'0' + (self.mon % 10) as u8;
        buf[8] = b'0' + (self.day / 10) as u8;
        buf[9] = b'0' + (self.day % 10) as u8;
        let time = Time::from(self.clone());
        let len = time.display_time(11, &mut buf);
        f.write_str(std::str::from_utf8(&buf[..len]).unwrap())
    }
}

impl Ord for DateTime {
    fn cmp(&self, other: &DateTime) -> cmp::Ordering {
        SystemTime::from(self.clone()).cmp(&SystemTime::from(other.clone()))
    }
}

impl PartialOrd for DateTime {
    fn partial_cmp(&self, other: &DateTime) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}


impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        use serde::de::Error;
        let s = String::deserialize(deserializer)?;
        DateTime::from_str(&s).map_err(|e| D::Error::custom(e))
    }
}


//DEFAULT_YEAR
static DEFAULT_YEAR: [u64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
//LEAP_YEAR
static LEAP_YEAR: [u64; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
//Day num
static DAY: u128 = 24 * 60 * 60 * 1000000000;

pub fn is_leap_year(y: u16) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

impl DateTime {
    pub fn from_system_time(s: SystemTime) -> Self {
        let mut dt = Self {
            nano: 0,
            sec: 0,
            min: 0,
            hour: 0,
            day: 0,
            mon: 0,
            year: 0000,
        };
        let d;
        if s < UNIX_EPOCH {
            d = UNIX_EPOCH.duration_since(s).unwrap();
            dt.year = 1969;
        } else {
            d = s.duration_since(UNIX_EPOCH).unwrap();
            dt.year = 1970;
        }
        let mut remain = d.as_nanos();
        let total_years = d.as_secs() / (365 * 24 * 3600) + 1;
        for _ in 0..total_years {
            if is_leap_year(dt.year) {
                if remain > (366 * DAY) {
                    remain -= 366 * DAY;
                } else {
                    break;
                }
            } else {
                if remain > (365 * DAY) {
                    remain -= 365 * DAY;
                } else {
                    break;
                }
            }
            if s < UNIX_EPOCH {
                dt.year -= 1;
            } else {
                dt.year += 1;
            }
        }
        let mons;
        if is_leap_year(dt.year) {
            mons = LEAP_YEAR;
        } else {
            mons = DEFAULT_YEAR;
        }
        //mon-day
        if s < UNIX_EPOCH {
            let mut t: u128 = 0;
            for m in mons {
                t += m as u128 * 24 * 3600 * 1000000000;
            }
            remain = t - remain;
        }
        let mut mon = 0;
        for m in mons {
            mon += 1;
            let mon_days = m as u128 * 24 * 3600 * 1000000000;
            if remain > mon_days {
                remain = remain.sub(mon_days);
                dt.mon = mon;
            } else {
                dt.mon = mon;
                for _ in 0..m {
                    if remain < DAY {
                        dt.day += 1;
                        break;
                    }
                    remain = remain.sub(DAY);
                    dt.day += 1;
                }
                break;
            }
        }
        dt.hour = ((remain / 1000000000) / 3600) as u8;
        dt.min = ((remain - dt.hour as u128 * 3600 * 1000000000) / (60 * 1000000000)) as u8;
        dt.sec = ((remain - dt.hour as u128 * 3600 * 1000000000 - dt.min as u128 * 60 * 1000000000) / 1000000000) as u8;
        dt.nano = (remain - dt.hour as u128 * 3600 * 1000000000 - dt.min as u128 * 60 * 1000000000 - dt.sec as u128 * 1000000000) as u32;
        dt
    }
}