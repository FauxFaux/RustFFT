
use num::{FromPrimitive, Signed};

use algorithm::{FFTAlgorithm, MixedRadixTerminal, MixedRadixSingle, Radix4, RadersAlgorithm, GoodThomasAlgorithm, NoopAlgorithm};
use math_utils;

const MIN_RADERS_SIZE: usize = 100;

pub fn plan_fft<T>(len: usize, inverse: bool) -> Box<FFTAlgorithm<T>>
    where T: Signed + FromPrimitive + Copy + 'static
{
    if len < 2 {
        Box::new(NoopAlgorithm {}) as Box<FFTAlgorithm<T>>
    } else if len.is_power_of_two() {
        Box::new(Radix4::new(len, inverse)) as Box<FFTAlgorithm<T>>
    } else {
        let factors = math_utils::prime_factors(len);

        let (smallest_factor, smallest_count) = factors[0];

        // try to pull out a power of 2 and run it through good-thomas
        // good-thomas does have some overhead
        // so we only want to do this if it's a non trivial size
        if smallest_factor == 2 && smallest_count > 4 {
            // we have a power of 2, so apply a good-thomas with our power of two as
            // one fft, and everything else as another
            let p2_fft =
                Box::new(Radix4::new(1 << smallest_count, inverse)) as Box<FFTAlgorithm<T>>;
            let other_fft = plan_fft_with_factors(len >> smallest_count, &factors[1..], inverse);

            Box::new(MixedRadixSingle::new(1 << smallest_count,
                                              p2_fft,
                                              len >> smallest_count,
                                              other_fft,
                                              inverse)) as Box<FFTAlgorithm<T>>
        } else {
            plan_fft_with_factors(len, factors.as_slice(), inverse)
        }
    }
}

fn plan_fft_with_factors<T>(len: usize,
                            factors: &[(usize, usize)],
                            inverse: bool)
                            -> Box<FFTAlgorithm<T>>
    where T: Signed + FromPrimitive + Copy + 'static
{
    if factors.len() == 1 {

        let (factor, count) = factors[0];
        
        // we have only one factor -- it's either a prime number
        // or a prime number raised to a power
        if factor > MIN_RADERS_SIZE {
            // our prime is large enough that it's worth trying to run rader's algorithm on it
            if count == 1 {
                Box::new(RadersAlgorithm::new(len, inverse)) as Box<FFTAlgorithm<T>>
            } else {
                // we have a large prime raised to a power. we're going to recursively cut the count in half,
                // sending half of them in one direction and half in the other
                // we're doing this because the cooley tukey algorithm works better when both sizes are roughly equal
                let left_count = count/2;

                let left_factors = [(factor, left_count)];
                let left_len = (0..left_count).fold(1, |product, _| product * factor);
                

                let right_factors = [(factor, count - left_count)];
                let right_len = len / left_len;


                let left_fft = plan_fft_with_factors(left_len, &left_factors, inverse);
                let right_fft = plan_fft_with_factors(right_len, &right_factors, inverse);

                

                Box::new(MixedRadixSingle::new(left_len, left_fft, right_len, right_fft, inverse)) as Box<FFTAlgorithm<T>>
            }

        } else {
            // the prime is small enough that we won't gain anything by splitting it up
            // so just cooley tukey it
            Box::new(MixedRadixTerminal::new(len, factors, inverse)) as Box<FFTAlgorithm<T>>
        }
    } else {
        // we have multiple factors. if any of them are large enough to run rader's on,
        // pull them out via the good thomas algorithm
        let (largest_factor, largest_count) = factors[factors.len() - 1];
        if largest_factor > MIN_RADERS_SIZE {

            let factor_size = largest_factor.pow(largest_count as u32);
            let factor_fft =
                plan_fft_with_factors(factor_size, &factors[factors.len() - 1..], inverse);
            let other_size = len / factor_size;
            let other_fft =
                plan_fft_with_factors(other_size, &factors[..factors.len() - 1], inverse);

            Box::new(MixedRadixSingle::new(factor_size,
                                              factor_fft,
                                              other_size,
                                              other_fft,
                                              inverse)) as Box<FFTAlgorithm<T>>

        } else {
            // the only thing left is small mixed factors, which is ideal for cooley tukey
            Box::new(MixedRadixTerminal::new(len, factors, inverse)) as Box<FFTAlgorithm<T>>
        }
    }
}
