use num::{Complex, Zero};
use common::{FFTnum, verify_length, verify_length_divisible};

use algorithm::FFTAlgorithm;
use twiddles;

pub struct DFT<T> {
    twiddles: Vec<Complex<T>>,
}

impl<T: FFTnum> DFT<T> {
    pub fn new(len: usize, inverse: bool) -> Self {
        DFT {
            twiddles: twiddles::generate_twiddle_factors(len, inverse),
        }
    }

    #[inline(always)]
    fn perform_fft(&self, signal: &[Complex<T>], spectrum: &mut [Complex<T>]) {
        for k in 0..spectrum.len() {
            let output_cell = spectrum.get_mut(k).unwrap();

            *output_cell = Zero::zero();
            let mut twiddle_index = 0;

            for input_cell in signal {
                let twiddle = self.twiddles[twiddle_index];
                *output_cell = *output_cell + twiddle * input_cell;

                twiddle_index += k;
                if twiddle_index >= self.twiddles.len() {
                    twiddle_index -= self.twiddles.len();
                }
            }
        }
    }
}

impl<T: FFTnum> FFTAlgorithm<T> for DFT<T> {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_length(input, output, self.len());

        self.perform_fft(input, output);
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_length_divisible(input, output, self.len());

        for (in_chunk, out_chunk) in input.chunks_mut(self.len()).zip(output.chunks_mut(self.len())) {
            self.perform_fft(in_chunk, out_chunk);
        }
    }
    fn len(&self) -> usize {
        self.twiddles.len()
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;
    use std::f32;
    use test_utils::{random_signal, compare_vectors};
    use num::{Complex, Zero};

    fn dft(signal: &[Complex<f32>], spectrum: &mut [Complex<f32>]) {
        for (k, spec_bin) in spectrum.iter_mut().enumerate() {
            let mut sum = Zero::zero();
            for (i, &x) in signal.iter().enumerate() {
                let angle = -1f32 * (i * k) as f32 * 2f32 * f32::consts::PI / signal.len() as f32;
                let twiddle = Complex::from_polar(&1f32, &angle);

                sum = sum + twiddle * x;
            }
            *spec_bin = sum;
        }
    }

    #[test]
    fn test_matches_dft() {
        for len in 1..50 {
            let mut input = random_signal(len);
            let mut expected = input.clone();
            dft(&input, &mut expected);

            let mut actual = input.clone();
            let dft_instance = DFT::new(len, false);
            dft_instance.process(&mut input, &mut actual);

            assert!(compare_vectors(&expected, &actual), "length = {}", len);
        }

        //verify that it doesn't crash if we have a length of 0
        let zero_dft = DFT::new(0, false);
        let mut zero_input: Vec<Complex<f32>> = Vec::new();
        let mut zero_output: Vec<Complex<f32>> = Vec::new();

        zero_dft.process(&mut zero_input, &mut zero_output);
    }

    /// Returns true if our `dft` function calculates the given spectrum from the
    /// given signal, and if rustfft's DFT struct does the same
    fn test_dft_correct(signal: &[Complex<f32>], spectrum: &[Complex<f32>]) -> bool {
        assert_eq!(signal.len(), spectrum.len());

        let expected_signal = signal.to_vec();
        let mut expected_spectrum = vec![Zero::zero(); spectrum.len()];

        let mut actual_signal = signal.to_vec();
        let mut actual_spectrum = vec![Zero::zero(); spectrum.len()];

        dft(&expected_signal, &mut expected_spectrum);

        let dft_instance = DFT::new(signal.len(), false);
        dft_instance.process(&mut actual_signal, &mut actual_spectrum);

        return compare_vectors(spectrum, &expected_spectrum) && compare_vectors(spectrum, &actual_spectrum);
    }

    #[test]
    fn test_dft_known_len_2() {
        let signal = [Complex{re: 1f32, im: 0f32},
                      Complex{re:-1f32, im: 0f32}];
        let spectrum = [Complex{re: 0f32, im: 0f32},
                        Complex{re: 2f32, im: 0f32}];
        assert!(test_dft_correct(&signal[..], &spectrum[..]));
    }

    #[test]
    fn test_dft_known_len_3() {
        let signal = [Complex{re: 1f32, im: 1f32},
                      Complex{re: 2f32, im:-3f32},
                          Complex{re:-1f32, im: 4f32}];
        let spectrum = [Complex{re: 2f32, im: 2f32},
                        Complex{re: -5.562177f32, im: -2.098076f32},
                        Complex{re: 6.562178f32, im: 3.09807f32}];
        assert!(test_dft_correct(&signal[..], &spectrum[..]));
    }

    #[test]
    fn test_dft_known_len_4() {
        let signal = [Complex{re: 0f32, im: 1f32},
                      Complex{re: 2.5f32, im:-3f32},
                      Complex{re:-1f32, im: -1f32},
                      Complex{re: 4f32, im: 0f32}];
        let spectrum = [Complex{re: 5.5f32, im: -3f32},
                        Complex{re: -2f32, im: 3.5f32},
                        Complex{re: -7.5f32, im: 3f32},
                        Complex{re: 4f32, im: 0.5f32}];
        assert!(test_dft_correct(&signal[..], &spectrum[..]));
    }

    #[test]
    fn test_dft_known_len_6() {
        let signal = [Complex{re: 1f32, im: 1f32},
                      Complex{re: 2f32, im: 2f32},
                      Complex{re: 3f32, im: 3f32},
                      Complex{re: 4f32, im: 4f32},
                      Complex{re: 5f32, im: 5f32},
                      Complex{re: 6f32, im: 6f32}];
        let spectrum = [Complex{re: 21f32, im: 21f32},
                        Complex{re: -8.16f32, im: 2.16f32},
                        Complex{re: -4.76f32, im: -1.24f32},
                        Complex{re: -3f32, im: -3f32},
                        Complex{re: -1.24f32, im: -4.76f32},
                        Complex{re: 2.16f32, im: -8.16f32}];
        assert!(test_dft_correct(&signal[..], &spectrum[..]));
    }
}