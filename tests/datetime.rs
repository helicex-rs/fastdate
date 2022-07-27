use std::str::FromStr;
use std::time::Duration;
use fastdate::{Date, DateTime, Time};

#[test]
fn test_other_space() {
    let d = DateTime::from_str("1234_12_13_11_12_13.123456").unwrap();
    println!("{}", d);
    assert_eq!("1234-12-13 11:12:13.123456".to_string(), d.to_string());
}

#[test]
fn test_date() {
    let d = DateTime::from_str("1234-12-13 11:12:13.123456").unwrap();
    println!("{}", d);
    assert_eq!("1234-12-13 11:12:13.123456".to_string(), d.to_string());
}

#[test]
fn test_date_utc() {
    let d = DateTime::now();
    println!("{}", d);
}

#[test]
fn test_date_utc_add() {
    let d = DateTime::now();
    let added = d + Duration::from_secs(1);
    println!("{},{}", d, added);
    assert_eq!(d.add(Duration::from_secs(1)), added);
}

#[test]
fn test_offset() {
    let utc = DateTime::from_str("2022-12-12 12:12:12.000000").unwrap();
    assert_eq!(format!("{}", utc.set_offset(1)), "2022-12-12 12:12:13.000000");
}

#[test]
fn test_timestamp() {
    let mut now = DateTime::utc();
    now.micro = 0;
    let timestamp = now.unix_timestamp();
    let new_time = DateTime::from_timestamp(timestamp);
    assert_eq!(now, new_time);
}

#[test]
fn test_timestamp_millis() {
    let mut now = DateTime::utc();
    now.micro = 0;
    let timestamp = now.unix_timestamp_millis();
    let new_time = DateTime::from_timestamp_millis(timestamp);
    assert_eq!(now, new_time);
}

#[test]
fn test_timestamp_nano() {
    let now = DateTime::utc();
    let timestamp = now.unix_timestamp_nano();
    let new_time = DateTime::from_timestamp_nano(timestamp);
    assert_eq!(now, new_time);
}

#[test]
fn test_unix_timestamp() {
    let d = DateTime::now().unix_timestamp();
    println!("unix:{}", d);
    let d = DateTime::utc().unix_timestamp();
    println!("unix:{}", d);

    let d = DateTime::now().unix_timestamp_millis();
    println!("unix ms:{}", d);
    let d = DateTime::utc().unix_timestamp_millis();
    println!("unix ms:{}", d);

    let d = DateTime::now().unix_timestamp_nano();
    println!("unix nano:{}", d);
    let d = DateTime::utc().unix_timestamp_nano();
    println!("unix nano:{}", d);
}

#[test]
fn test_offset_zone() {
    let utc = DateTime::from_str("2022-12-12 00:00:00-08:00").unwrap();
    println!("{}", utc);
}

#[test]
fn test_into() {
    let utc = DateTime::from_str("2022-12-12 00:00:00+08:00").unwrap();
    let date: Date = utc.into();
    let time: Time = utc.into();
    println!("{},{}", date, time);
    assert_eq!("2022-12-12", date.to_string());
    assert_eq!("08:00:00.000000", time.to_string());
}

#[test]
fn test_befor_after() {
    let date1 = DateTime::from_str("2022-12-12 00:00:00").unwrap();
    let date2 = DateTime::from_str("2022-12-12 01:00:00").unwrap();
    assert_eq!(date2.after(&date1), true);
    assert_eq!(date1.before(&date2), true);
}

#[test]
fn test_parse_z() {
    let date = DateTime::from_str("2022-12-12 00:00:00.000000Z").unwrap();
    let date_offset = date.clone().set_offset(fastdate::offset_sec());
    assert_eq!("2022-12-12 08:00:00.000000", date_offset.to_string());
}