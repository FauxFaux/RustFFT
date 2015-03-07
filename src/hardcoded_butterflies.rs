use num::{Complex, Zero};
use test::Bencher;
use std::f32;
use std::iter::repeat;

pub fn butterfly_2(data: &mut [Complex<f32>], stride: usize,
                   twiddles: &[Complex<f32>], num_ffts: usize) {
    let mut idx_1 = 0us;
    let mut idx_2 = num_ffts;
    let mut twiddle_idx = 0us;
    // TODO speed: use pointers instead of pointer offsets?
    for _ in 0..num_ffts {
        unsafe {
            let twiddle = twiddles.get_unchecked(twiddle_idx);
            let temp = data.get_unchecked(idx_2) * twiddle;
            *data.get_unchecked_mut(idx_2) = data.get_unchecked(idx_1) - temp;
            *data.get_unchecked_mut(idx_1) = data.get_unchecked(idx_1) + temp;
        }
        twiddle_idx += stride;
        idx_1 += 1;
        idx_2 += 1;
    }
}

pub fn butterfly_4(data: &mut [Complex<f32>], stride: usize,
                   twiddles: &[Complex<f32>], num_ffts: usize) {
    let mut idx = 0us;
    let mut tw_idx_1 = 0us;
    let mut tw_idx_2 = 0us;
    let mut tw_idx_3 = 0us;
    let mut scratch: [Complex<f32>; 6] = [Zero::zero(); 6];
    //TODO does using get_unchecked on scratch help with speed?
    for _ in 0..num_ffts {
        unsafe {
            scratch[0] = data.get_unchecked(idx + 1 * num_ffts) * twiddles[tw_idx_1];
            scratch[1] = data.get_unchecked(idx + 2 * num_ffts) * twiddles[tw_idx_2];
            scratch[2] = data.get_unchecked(idx + 3 * num_ffts) * twiddles[tw_idx_3];
            scratch[5] = data.get_unchecked(idx) - scratch[1];
            *data.get_unchecked_mut(idx) = data.get_unchecked(idx) + scratch[1];
            scratch[3] = scratch[0] + scratch[2];
            scratch[4] = scratch[0] - scratch[2];
            *data.get_unchecked_mut(idx + 2 * num_ffts) = data.get_unchecked(idx) - scratch[3];
            *data.get_unchecked_mut(idx) = data.get_unchecked(idx) + scratch[3];
            data.get_unchecked_mut(idx + num_ffts).re = scratch[5].re + scratch[4].im;
            data.get_unchecked_mut(idx + num_ffts).im = scratch[5].im - scratch[4].re;
            data.get_unchecked_mut(idx + 3 * num_ffts).re = scratch[5].re - scratch[4].im;
            data.get_unchecked_mut(idx + 3 * num_ffts).im = scratch[5].im + scratch[4].re;

            tw_idx_1 += 1 * stride;
            tw_idx_2 += 2 * stride;
            tw_idx_3 += 3 * stride;
            idx += 1;
        }
    }
}

#[bench]
fn bench_butterfly_2(b: &mut Bencher) {
    let stride = 4us;
    let num_ffts = 1000us;

    let len = 2 * stride * num_ffts;
    let twiddles: Vec<Complex<f32>> = (0..len)
        .map(|i| -1. * (i as f32) * f32::consts::PI_2 / (len as f32))
        .map(|phase| Complex::from_polar(&1., &phase))
        .collect();

    let mut data: Vec<Complex<f32>> = repeat(Zero::zero()).take(len).collect();

    b.iter(|| butterfly_2(&mut data, stride, &twiddles, num_ffts));
}
