use nalgebra as na;
use na::{
    Vector3, Matrix3, Matrix3x1, Matrix1x3, Matrix1,
    VectorN, MatrixN,
};
use na::{DefaultAllocator, DimName, RealField};
use na::allocator::Allocator;

use super::Timestamp;
use crate::dwt_utils::{
    dwt_ticks_to_us,
    // dwt_us_to_ticks,
    dwt_time_diff,
};


type T = f64;

#[allow(non_upper_case_globals)]
const TrackerDim: usize = 3;


// Tracker Status
#[derive(Default)]
pub struct CONSTRUCTED;
#[derive(Default)]
pub struct INITIALIZED;


#[allow(non_snake_case)]
#[derive(Debug)]
pub struct ClockTracker<S> {
    timestamp_noise: f64,
    acceleration_nosie: f64,
    x: Vector3<T>,
    P: Matrix3<T>,
    t: Timestamp,
    outlier_ratio_th: f64,
    status: S,
}


#[allow(non_snake_case)]
impl<S> ClockTracker<S> {

    pub fn new(acc_noise: f64, ts_noise: f64) -> ClockTracker<CONSTRUCTED> {
        assert!(acc_noise > 0.0 && ts_noise > 0.0);
        ClockTracker {
            timestamp_noise: ts_noise,
            acceleration_nosie: acc_noise,
            x: Vector3::<T>::zeros(),
            P: 8e8 * Matrix3::<T>::identity(),
            t: Timestamp::default(),
            outlier_ratio_th: 2.8,
            status: CONSTRUCTED,
        }
    }

    fn clone<S2: Default>(&self) -> ClockTracker<S2> {
        ClockTracker {
            timestamp_noise: self.timestamp_noise,
            acceleration_nosie: self.acceleration_nosie,
            x: self.x.clone(),
            P: self.P.clone(),
            t: self.t,
            outlier_ratio_th: self.outlier_ratio_th,
            status: S2::default(),
        }
    }

    pub fn t(&self) -> &Timestamp {
        &self.t
    }

    pub fn x(&self) -> &Vector3<T> {
        &self.x
    }

    pub fn P(&self) -> &Matrix3<T> {
        &self.P
    }

    pub fn set_outlier_ratio_th(&mut self, outlier_ratio_th: f64) {
        assert!(outlier_ratio_th > 0.0);
        self.outlier_ratio_th = outlier_ratio_th;
    }
}

impl ClockTracker<CONSTRUCTED> {
    #[allow(non_snake_case)]
    pub fn init_with_mat(self, t0: Timestamp, x0: &Vector3<T>, P0: &Matrix3<T>) -> ClockTracker<INITIALIZED> {
        ClockTracker {
            x: x0.clone(),
            P: P0.clone(),
            t: t0,
            ..self.clone::<INITIALIZED>()
        }
    }

    pub fn init_with_matdiag(self, t0: Timestamp, x0: &Vector3<T>, p0: &Vector3<T>) -> ClockTracker<INITIALIZED> {
        self.init_with_mat(t0, &x0, &Matrix3::<T>::from_diagonal(&p0))
    }
}

#[allow(non_snake_case)]
impl ClockTracker<INITIALIZED> {
    pub fn predict_x(self: &Self, t: Timestamp) -> Vector3<T> {
        assert!(t >= self.t);
        let mut new_x = self.x.clone();
        if t == self.t {
            return new_x;
        }
        let dt = dwt_ticks_to_us::<_, f64>(dwt_time_diff(t, self.t)) * 1e-6;
        let dx_no_last = self.x.rows(1, TrackerDim - 1) * dt;
        let new_x_no_last = dx_no_last + self.x.rows(0, TrackerDim - 1);
        new_x.rows_mut(0, TrackerDim - 1).copy_from(&new_x_no_last);
        new_x
    }
    pub fn predict_P(self: &Self, t: Timestamp) -> Matrix3<T> {
        assert!(t >= self.t);
        if t == self.t {
            return self.P.clone();
        }
        let dt = dwt_ticks_to_us::<_, f64>(dwt_time_diff(t, self.t)) * 1e-6;
        let mut A: Matrix3<T> = Matrix3::<T>::new(
            1.0, dt, 0.5 * dt * dt,
            0.0, 1.0, dt,
            0.0, 0.0, 1.0,
        );
        let dt2 = dt * dt;
        let dt3 = dt * dt2;
        let dt4 = dt * dt3;
        let dt5 = dt * dt4;
        let Q: Matrix3<T> = Matrix3::<T>::new(
            dt5 / 20.0, dt4 / 8.0, dt3 / 6.0,
            dt4 / 8.0, dt3 / 3.0, dt2 / 2.0,
            dt3 / 6.0, dt2 / 2.0, dt,
        ) * self.acceleration_nosie * self.acceleration_nosie;
        let new_P = A * self.P * A.transpose() + Q;
        new_P
    }
    pub fn predict_mut(self: &mut Self, t: Timestamp) -> bool {
        let new_x = self.predict_x(t);
        let new_P = self.predict_P(t);
        self.x = new_x;
        self.P = new_P;
        self.t = t;
        // println!("pred: {}", self.x().transpose());
        true
    }

    pub fn update(self: &mut Self, ts: Timestamp) -> bool {
        let H: Matrix1x3<T> = Matrix1x3::new(1.0, 0.0, 0.0);
        // let err = Matrix1::new(dwt_ticks_to_us::<_, T>(ts) * 1e-6) - H * self.x;
        let err = Matrix1::new(dwt_ticks_to_us::<_, T>(ts) * 1e-6 - self.x[0]);
        let PHt = self.P * H.transpose();
        let s2 = H * PHt + Matrix1::new(self.timestamp_noise * self.timestamp_noise);
        let is_outlier = err[0] * err[0] > self.outlier_ratio_th * self.outlier_ratio_th * s2[0];
        if is_outlier {
            // println!("outlier: err: {:e}, {}", err[0], err[0] * err[0] / s2[0]);
            false
        } else {
            let K: Matrix3x1<T> = PHt * Matrix1::new(1.0 / s2[0]);
            let new_x = self.x + K * err;
            let new_P = (Matrix3::<T>::identity() - K * H) * self.P;
            self.x = new_x;
            self.P = new_P;
            // println!("correct: {}", self.x().transpose());
            true
        }
    }
}