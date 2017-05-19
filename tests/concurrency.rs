//! Test that we can use an `FFT` object from multiple threads

extern crate rustfft;

use std::thread;

use rustfft::FFTplanner;
use rustfft::num_complex::Complex32;

#[test]
fn two_threads() {
    let inverse = false;
    let mut planner = FFTplanner::new(inverse);
    let fft = planner.plan_fft(100);

    let threads: Vec<thread::JoinHandle<_>> = (0..2).map(|_| {
        let fft_copy = fft.clone();
        thread::spawn(move || {
            let mut signal = vec![Complex32::new(0.0, 0.0); 100];
            let mut spectrum = vec![Complex32::new(0.0, 0.0); 100];
            fft_copy.process(&mut signal, &mut spectrum);
        })
    }).collect();

    for thread in threads {
        thread.join().unwrap();
    }
}
