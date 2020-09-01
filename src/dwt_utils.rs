
use std::num::Wrapping;
use num::{ Integer, Float, cast, NumCast };
use std::ops::{ Mul, Div };


pub type Timestamp = u64;
pub type TimestampDiff = i64;


// UWB microsecond (uus) to device time unit (dtu, around 15.65 ps) conversion factor.
// 1 uus = 512 / 499.2 µs and 1 µs = 499.2 * 128 dtu.
pub const UUS_TO_DTU_TIME: f64 = 65536.0;
pub const DTU_TO_UUS_TIME: f64 = 1.0 / 65536.0;
pub const US_TO_DTU_TIME: f64 = 499.2 * 128.0;
pub const DTU_TO_US_TIME: f64 = 1.0 / 128.0 / 499.2;


pub fn dwt_uus_to_us<T>(uus: T) -> T
where T: Float + Mul<f64, Output=T>,
{
    // (65536.0 / 128.0 / 499.2)
    uus * (512.0 / 499.2)
}

pub fn dwt_us_to_uus<T>(us: T) -> T
where T: Float + Mul<f64, Output=T>,
{
    // (128.0 * 499.2 / 65536.0)
    us * 0.975
}

pub fn dwt_uus_to_ticks<T, To>(uus: T) -> To
where T: Float + Mul<f64, Output=T>,
    To: Integer + NumCast,
{
    // uus * 65536.0
    cast::<T, To>(uus * 65536.0).unwrap()
}

pub fn dwt_ticks_to_uus<T, To>(ticks: T) -> To
where T: Integer + NumCast,
    To: Float + Div<f64, Output=To>,
{
    // ticks / 65536.0
    cast::<T, To>(ticks).unwrap() / 65536.0
}

pub fn dwt_us_to_ticks<T, To>(us: T) -> To
where T: Float + Mul<f64, Output=T>,
    To: Integer + NumCast,
{
    // us * 63897.6
    cast::<T, To>(us * 63897.6).unwrap()
}

pub fn dwt_ticks_to_us<T, To>(ticks: T) -> To
where T: Integer + NumCast,
    To: Float + Div<f64, Output=To>,
{
    // ticks / 63897.6
    cast::<T, To>(ticks).unwrap() / 63897.6
}


pub fn dwt_time_diff(t1: u64, t2: u64) -> i64 {
    let mut dt: i64 = (Wrapping(t1) - Wrapping(t2)).0 as i64;
    if dt >= 2i64.pow(39) {
        dt -= 2i64.pow(40);
    }
    if dt < -2i64.pow(39) {
        dt += 2i64.pow(40);
    }
    dt
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dwt_uus_to_us() {
        assert_eq!(dwt_uus_to_us(8.0), 8.0 * UUS_TO_DTU_TIME * DTU_TO_US_TIME);
    }

    #[test]
    fn test_dwt_us_to_uus() {
        assert_eq!(dwt_us_to_uus(8.0), 8.0 * US_TO_DTU_TIME * DTU_TO_UUS_TIME);
    }

    #[test]
    fn test_dwt_uus_to_ticks() {
        assert_eq!(dwt_uus_to_ticks::<_, u64>(8.0), (8.0 * UUS_TO_DTU_TIME) as u64);
    }

    #[test]
    fn test_dwt_ticks_to_uus() {
        assert_eq!(dwt_ticks_to_uus::<_, f64>(8), 8.0 * DTU_TO_UUS_TIME);
    }

    #[test]
    fn test_dwt_us_to_ticks() {
        assert_eq!(dwt_us_to_ticks::<_, u64>(8.0), (8.0 * US_TO_DTU_TIME) as u64);
    }

    #[test]
    fn test_dwt_ticks_to_us() {
        assert_eq!(dwt_ticks_to_us::<_, f64>(8), 8.0 * DTU_TO_US_TIME);
    }

    #[test]
    fn test_dwt_time_diff() {
        assert_eq!(dwt_time_diff(8, 0), 8);
        assert_eq!(dwt_time_diff(0, 8), -8);
        assert_eq!(dwt_time_diff(8, 2u64.pow(40)), 8);
        assert_eq!(dwt_time_diff(0, 2u64.pow(40) + 8), -8);
    }
}