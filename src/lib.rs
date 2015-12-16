#![cfg_attr(all(feature = "bench", test), feature(test))]

extern crate num;

mod butterflies;

use num::{Complex, Zero, One, Float, Num, FromPrimitive, Signed};
use num::traits::cast;
use std::f32;

use butterflies::{butterfly_2, butterfly_3, butterfly_4, butterfly_5};

pub struct FFT<T> {
    factors: Vec<(usize, usize)>,
    twiddles: Vec<Complex<T>>,
    scratch: Vec<Complex<T>>,
    inverse: bool,
}

impl<T> FFT<T> where T: Signed + FromPrimitive + Copy {
    /// Creates a new FFT context that will process signal of length
    /// `len`. If `inverse` is `true`, then this struct will run inverse
    /// FFTs. This implementation of the FFT doesn't do any scaling on both
    /// the forward and backward transforms, so doing a forward then backward
    /// FFT on a signal will scale the signal by its length.
    pub fn new(len: usize, inverse: bool) -> Self {
        let dir = if inverse { 1 } else { -1 };
        let factors = factor(len);
        let max_fft_len = factors.iter().map(|&(a, _)| a).max();
        let scratch = match max_fft_len {
            None | Some(0...5) => vec![Zero::zero(); 0],
            Some(l) => vec![Zero::zero(); l],
        };
        FFT {
            factors: factor(len),
            twiddles: (0..len)
                      .map(|i| dir as f32 * i as f32 * 2.0 * f32::consts::PI / len as f32)
                      .map(|phase| Complex::from_polar(&1.0, &phase))
                      .map(|c| Complex {re: FromPrimitive::from_f32(c.re).unwrap(),
                                        im: FromPrimitive::from_f32(c.im).unwrap()})
                      .collect(),
            scratch: scratch,
            inverse: inverse,
        }
    }

    /// Runs the FFT on the input `signal` buffer, and places the output in the
    /// `spectrum` buffer.
    ///
    /// # Panics
    /// This method will panic if `signal` and `spectrum` are not the length
    /// specified in the struct's constructor.
    pub fn process(&mut self, signal: &[Complex<T>], spectrum: &mut [Complex<T>]) {
        assert!(signal.len() == spectrum.len());
        assert!(signal.len() == self.twiddles.len());
        cooley_tukey(signal, spectrum, 1, &self.twiddles[..], &self.factors[..],
                     &mut self.scratch, self.inverse);
    }
}

fn cooley_tukey<T>(signal: &[Complex<T>],
                   spectrum: &mut [Complex<T>],
                   stride: usize,
                   twiddles: &[Complex<T>],
                   factors: &[(usize, usize)],
                   scratch: &mut [Complex<T>],
                   inverse: bool) where T: Signed + FromPrimitive + Copy {
    if let Some(&(n1, n2)) = factors.first() {
        if n2 == 1 {
            // An FFT of length 1 is just the identity operator
            let mut spectrum_idx = 0usize;
            let mut signal_idx = 0usize;
            while signal_idx < signal.len() {
                unsafe { *spectrum.get_unchecked_mut(spectrum_idx) =
                    *signal.get_unchecked(signal_idx); }
                spectrum_idx += 1;
                signal_idx += stride;
            }
        } else {
            // Recursive call to perform n1 ffts of length n2
            for i in 0..n1 {
                cooley_tukey(&signal[i * stride..],
                             &mut spectrum[i * n2..],
                             stride * n1, twiddles, &factors[1..],
                             scratch, inverse);
            }
        }

        match n1 {
            5 => unsafe { butterfly_5(spectrum, stride, twiddles, n2) },
            4 => unsafe { butterfly_4(spectrum, stride, twiddles, n2, inverse) },
            3 => unsafe { butterfly_3(spectrum, stride, twiddles, n2) },
            2 => unsafe { butterfly_2(spectrum, stride, twiddles, n2) },
            _ => butterfly(spectrum, stride, twiddles, n2, n1, &mut scratch[..n1]),
        }
    }
}

fn butterfly<T: Num + Copy>(data: &mut [Complex<T>], stride: usize,
                            twiddles: &[Complex<T>], num_ffts: usize,
                            fft_len: usize, scratch: &mut [Complex<T>]) {
    // for each fft we have to perform...
    for fft_idx in 0..num_ffts {

        // copy over data into scratch space
        let mut data_idx = fft_idx;
        for s in scratch.iter_mut() {
            *s = unsafe { *data.get_unchecked(data_idx) };
            data_idx += num_ffts;
        }

        // perfom the butterfly from the scratch space into the original buffer
        let mut data_idx = fft_idx;
        while data_idx < fft_len * num_ffts {
            let out_sample = unsafe { data.get_unchecked_mut(data_idx) };
            *out_sample = Zero::zero();
            let mut twiddle_idx = 0usize;
            for in_sample in scratch.iter() {
                let twiddle = unsafe { twiddles.get_unchecked(twiddle_idx) };
                *out_sample = *out_sample + in_sample * twiddle;
                twiddle_idx += stride * data_idx;
                if twiddle_idx >= twiddles.len() { twiddle_idx -= twiddles.len() }
            }
            data_idx += num_ffts;
        }

    }
}

pub fn dft<T: Float>(signal: &[Complex<T>], spectrum: &mut [Complex<T>]) {
    for (k, spec_bin) in spectrum.iter_mut().enumerate() {
        let mut sum = Zero::zero();
        for (i, &x) in signal.iter().enumerate() {
            let angle = cast::<_, T>(-1 * (i * k) as isize).unwrap()
                * cast(2.0 * f32::consts::PI).unwrap()
                / cast(signal.len()).unwrap();
            let twiddle = Complex::from_polar(&One::one(), &angle);
            sum = sum + twiddle * x;
        }
        *spec_bin = sum;
    }
}

/// Factors an integer into its prime factors.
fn factor(n: usize) -> Vec<(usize, usize)> {
    let mut factors = Vec::new();
    let mut next = n;
    while next > 1 {
        for div in 2..next + 1 {
            if next % div == 0 {
                next = next / div;
                factors.push((div, next));
                break;
            }
        }
    }
    return factors;
}
