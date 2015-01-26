#![allow(unstable)]
extern crate num;

mod hardcoded_butterflies;

use num::{Complex, Zero};
use std::iter::{repeat, range_step_inclusive, range_step};
use std::f32;

use hardcoded_butterflies::butterfly_2;

pub struct FFT {
    scratch: Vec<Complex<f32>>,
    factors: Vec<(usize, usize)>,
    twiddles: Vec<Complex<f32>>,
}

impl FFT {
    pub fn new(len: usize) -> Self {
        FFT {
            scratch: repeat(Zero::zero()).take(len).collect(),
            factors: factor(len),
            twiddles: range(0, len)
                      .map(|i| -1. * (i as f32) * f32::consts::PI_2 / (len as f32))
                      .map(|phase| Complex::from_polar(&1., &phase))
                      .collect(),
        }
    }

    pub fn process(&mut self, signal: &[Complex<f32>], spectrum: &mut [Complex<f32>]) {
        cooley_tukey(signal, 1,
                     spectrum, 1,
                     self.scratch.as_mut_slice(), 1,
                     self.twiddles.as_slice(),
                     self.factors.as_slice());
    }
}

//TODO can we collapse all these strides into one stride value?
fn cooley_tukey(signal: &[Complex<f32>],
                signal_stride: usize,
                spectrum: &mut [Complex<f32>],
                spectrum_stride: usize,
                scratch: &mut [Complex<f32>],
                scratch_stride: usize,
                twiddles: &[Complex<f32>],
                factors: &[(usize, usize)]) {
    match factors {
        [_] | [] => dft_slice(signal, signal_stride,
                              spectrum, spectrum_stride,
                              twiddles),
        [(n1, n2), other_factors..] => {
            for i in range(0, n1) {
                // perform the smaller FFTs from the signal buffer into
                // the scratch buffer, using the spectrum buffer as scratch space
                cooley_tukey(&signal[i * signal_stride..], signal_stride * n1,
                             &mut scratch[i * scratch_stride..], scratch_stride * n1,
                             &mut spectrum[i * spectrum_stride..], spectrum_stride * n1,
                             twiddles, other_factors);
            }

            match n1 {
                2 => butterfly_2(scratch, scratch_stride,
                                 spectrum, spectrum_stride,
                                 twiddles, n2),
                _ => butterfly(scratch, scratch_stride,
                               spectrum, spectrum_stride,
                               twiddles, n1, n2),
            }
        }
    }
}

fn butterfly(input: &[Complex<f32>],
             input_stride: usize,
             output: &mut [Complex<f32>],
             output_stride: usize,
             twiddles: &[Complex<f32>],
             num_cols: usize,
             num_rows: usize) {
    // for each row in input
    let mut twiddle_idx_increase_1 = 0;
    for (i, in_row) in input.chunks(num_cols * input_stride).enumerate() {
        let out_col = &mut output[i * output_stride..];
        let out_col_stride = output_stride * num_rows;
        let mut twiddle_idx_increase_2 = 0;
        for spec_bin_idx in range_step(0, out_col.len(), out_col_stride) {
            let spec_bin = unsafe { out_col.get_unchecked_mut(spec_bin_idx) };
            *spec_bin = Zero::zero();
            let mut twiddle_idx = 0;
            let twiddle_idx_increase = twiddle_idx_increase_1 + twiddle_idx_increase_2;
            for sig_bin_idx in range_step(0, in_row.len(), input_stride) {
                let sig_bin = unsafe { in_row.get_unchecked(sig_bin_idx) };
                let twiddle = unsafe { twiddles.get_unchecked(twiddle_idx) };
                *spec_bin = *spec_bin + *twiddle * *sig_bin;
                twiddle_idx += twiddle_idx_increase;
                if twiddle_idx > twiddles.len() { twiddle_idx -= twiddles.len() }
            }
            twiddle_idx_increase_2 += num_rows * input_stride;
        }
        twiddle_idx_increase_1 += input_stride;
    }
}

pub fn dft_slice(signal: &[Complex<f32>],
                 signal_stride: usize,
                 spectrum: &mut [Complex<f32>],
                 spectrum_stride: usize,
                 twiddles: &[Complex<f32>]) {
    let mut twiddle_idx_increase = 0;
    for spec_bin_idx in range_step(0, spectrum.len(), spectrum_stride) {
        let spec_bin = unsafe { spectrum.get_unchecked_mut(spec_bin_idx) };
        *spec_bin = Zero::zero();
        let mut twiddle_idx = 0;
        for signal_bin_idx in range_step(0, signal.len(), signal_stride) {
            let signal_bin = unsafe { signal.get_unchecked(signal_bin_idx) };
            let twiddle = unsafe { twiddles.get_unchecked(twiddle_idx) };
            *spec_bin = *spec_bin + twiddle * *signal_bin;
            twiddle_idx += twiddle_idx_increase;
            if twiddle_idx > twiddles.len() {twiddle_idx -= twiddles.len()}
        }
        twiddle_idx_increase += signal_stride;
    }
}

pub fn dft<'a, 'b, I, O>(signal: I, spectrum: O)
where I: Iterator<Item=&'a Complex<f32>> + ExactSizeIterator + Clone,
      O: Iterator<Item=&'b mut Complex<f32>>
{
    for (k, spec_bin) in spectrum.enumerate()
    {
        let signal_iter = signal.clone();
        let mut sum: Complex<f32> = Zero::zero();
        for (i, &x) in signal_iter.enumerate() {
            let angle = -1. * ((i * k) as f32) * f32::consts::PI_2 / (signal.len() as f32);
            let twiddle: Complex<f32> = Complex::from_polar(&1f32, &angle);
            sum = sum + twiddle * x;
        }
        *spec_bin = sum;
    }
}

fn factor(n: usize) -> Vec<(usize, usize)>
{
    let mut factors = Vec::new();
    let mut next = n;
    while next > 1 {
        for div in Some(2us).into_iter().chain(range_step_inclusive(3us, next, 2)) {
            if next % div == 0 {
                next = next / div;
                factors.push((div, next));
                break;
            }
        }
    }
    return factors;
}
