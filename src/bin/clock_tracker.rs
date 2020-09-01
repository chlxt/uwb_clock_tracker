
use nalgebra as na;
use uwb_clock_tracker::{
    dwt_utils,
    clock_tracker,
    clock_tracker::ClockTracker,
};

fn main() {
    println!("Hello, dwt: {}", na::Vector3::<f32>::x());

    let tracker = ClockTracker::<clock_tracker::CONSTRUCTED>::new(1e-8, 0.18e-9);

    let mut tracker = tracker.init_with_matdiag(0, &na::Vector3::<f64>::x(), &na::Vector3::<f64>::zeros());

    tracker.predict_mut(1);
    tracker.update(1);
}
