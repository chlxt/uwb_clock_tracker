use std::error::Error;
use std::str::FromStr;
use nalgebra as na;
use na::{
    U1, U2, U3, Dynamic,
    MatrixMN,
    DMatrix,
};
use uwb_clock_tracker::{
    dwt_utils,
    clock_tracker,
    clock_tracker::ClockTracker,
};
use clap::{ Arg, App, SubCommand };
use csv::{ Reader, ReaderBuilder, Writer, StringRecord };
use string_error;


const TS_NOISE: f64 = 0.18e-9;


type DMat = MatrixMN::<dwt_utils::Timestamp, Dynamic, Dynamic>;

fn read_dataset(dataset_name: &str) -> Result<DMat, Box<dyn Error>> {
    let mut rder_builder = ReaderBuilder::new();
    rder_builder.has_headers(false).delimiter(b' ');
    let mut rder = rder_builder.from_path(dataset_name)?;
    let nsamples = rder.records().count();
    let mut rder = rder_builder.from_path(dataset_name)?;
    let mut dataset = DMat::zeros(nsamples, 2);
    for i  in 0..nsamples {
        let mut record = StringRecord::new();
        if rder.read_record(&mut record)? {
            // assert!(record.len() == 2);
            assert!(record.len() >= 2);
            // let t_s = dwt_utils::Timestamp::from_str(&record[0]).unwrap();
            // let t_r = dwt_utils::Timestamp::from_str(&record[1]).unwrap();
            let t_s = dwt_utils::Timestamp::from_str_radix(&record[0].trim_start_matches("0x".trim_start_matches("0X")), 16).unwrap();
            let t_r = dwt_utils::Timestamp::from_str_radix(&record[1].trim_start_matches("0x".trim_start_matches("0X")), 16).unwrap();
            dataset.row_mut(i)[0] = t_s;
            dataset.row_mut(i)[1] = t_r;
        } else {
            return Err::<DMat, _>(string_error::new_err("dataset file format un-correct!"));
        }
    }
    Ok(dataset)
}

fn main() {
    let app = App::new("uwb_clock_tracker demo")
        .version("1.0")
        .author("drivextech@outlook.com")
        .arg(Arg::with_name("start_counter")
            .short("s")
            .long("start-counter")
            .value_name("START COUNTER")
            .help("start counter of dataset")
            .takes_value(true)
            .default_value("0"))
        .arg(Arg::with_name("end_counter")
            .short("e")
            .long("end-counter")
            .value_name("END COUNTER")
            .help("end counter of dataset")
            .takes_value(true)
            .default_value("-1"))
        .arg(Arg::with_name("dataset")
            .help("dataset file")
            .required(true)
            .default_value("dataset")
            .index(1));
    let args = app.get_matches();

    let dataset_name = args.value_of("dataset").unwrap();
    println!("dataset file: {}", dataset_name);
    
    let dataset = read_dataset(dataset_name).expect("dataset should be valid");
    let dataset_nsamples = dataset.nrows();
    println!("dataset file record num: {}", dataset_nsamples);

    let start_counter = usize::from_str(args.value_of("start_counter").unwrap()).expect("start_counter MUST >= 0");
    let end_counter = isize::from_str(args.value_of("end_counter").unwrap()).expect("end_counter MUST > 0 or == -1");
    let end_counter = if end_counter < 0 {
        dataset_nsamples as usize
    } else {
        end_counter as usize
    };

    let tracker = ClockTracker::<clock_tracker::CONSTRUCTED>::new(1e-8, TS_NOISE);
    // let tracker = ClockTracker::<clock_tracker::CONSTRUCTED>::new(1e-9, TS_NOISE);

    let t0: dwt_utils::Timestamp = dataset.row(start_counter)[0];
    let to0: f64 = dwt_utils::dwt_ticks_to_us(dataset.row(start_counter)[1]);
    let to0 = to0 * 1e-6;
    let mut tracker = tracker.init_with_matdiag(t0, &na::Vector3::<f64>::new(to0, 1.0, 0.0),
                                                                        &na::Vector3::<f64>::new(8e4, 4e2, 0.8e0));

    let mut wter = Writer::from_path([dataset_name, ".out"].concat()).expect("output file should be valid");
    // for i in 0..1000 {
    for i in start_counter..end_counter {
        tracker.predict_mut(dataset.row(i)[0]);
        if !tracker.update(dataset.row(i)[1]) {
            println!("i: {}, update failed!", i);
        }

        // println!("i: {}, x: {}", i, tracker.x().transpose());
        wter.write_record(vec![tracker.x()[0].to_string(), tracker.x()[1].to_string(), tracker.x()[2].to_string()]).unwrap();
    }
}
