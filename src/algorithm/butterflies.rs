use num::{Complex, FromPrimitive, Zero};
use common::FFTnum;

use twiddles;
use super::{FFTAlgorithm, FFTButterfly};




#[inline(always)]
unsafe fn swap_unchecked<T: Copy>(buffer: &mut [T], a: usize, b: usize) {
	let temp = *buffer.get_unchecked(a);
	*buffer.get_unchecked_mut(a) = *buffer.get_unchecked(b);
	*buffer.get_unchecked_mut(b) = temp;
}

#[inline(always)]
fn verify_size<T>(signal: &[T], spectrum: &[T], expected: usize) {
	assert_eq!(signal.len(), expected, "Signal is the wrong length. Expected {}, got {}", expected, signal.len());
	assert_eq!(spectrum.len(), expected, "Spectrum is the wrong length. Expected {}, got {}", expected, spectrum.len());
}





pub struct Butterfly2;
impl Butterfly2 {
    #[inline(always)]
    unsafe fn perform_fft<T: FFTnum>(&self, buffer: &mut [Complex<T>]) {
    	let temp = *buffer.get_unchecked(0) + *buffer.get_unchecked(1);
        
        *buffer.get_unchecked_mut(1) = *buffer.get_unchecked(0) - *buffer.get_unchecked(1);
        *buffer.get_unchecked_mut(0) = temp;
    }
}
impl<T: FFTnum> FFTButterfly<T> for Butterfly2 {
    #[inline(always)]
    unsafe fn process_multi_inplace(&self, buffer: &mut [Complex<T>]) {
    	for chunk in buffer.chunks_mut(2) {
    		self.perform_fft(chunk);
    	}
    }
}
impl<T: FFTnum> FFTAlgorithm<T> for Butterfly2 {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_size(input, output, 2);
        output.copy_from_slice(input);

        unsafe { self.perform_fft(output) };
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        output.copy_from_slice(input);

        unsafe { self.process_multi_inplace(output) };
    }
    #[inline(always)]
    fn len(&self) -> usize {
        2
    }
}



pub struct Butterfly3<T> {
	pub twiddle: Complex<T>,
}
impl<T: FFTnum> Butterfly3<T> {
	#[inline(always)]
    pub fn new(inverse: bool) -> Self {
        Self {
            twiddle: twiddles::single_twiddle(1, 3, inverse)
        }
    }

    #[inline(always)]
    pub fn inverse_of(fft: &Butterfly3<T>) -> Self {
        Self {
            twiddle: fft.twiddle.conj()
        }
    }

	#[inline(always)]
	pub unsafe fn perform_fft(&self, buffer: &mut [Complex<T>]) {
        let butterfly2 = Butterfly2{};

    	butterfly2.perform_fft(&mut buffer[1..]);
    	let temp = *buffer.get_unchecked(0);

    	*buffer.get_unchecked_mut(0) = temp + *buffer.get_unchecked(1);

    	*buffer.get_unchecked_mut(1) = *buffer.get_unchecked(1) * self.twiddle.re + temp;
    	*buffer.get_unchecked_mut(2) = *buffer.get_unchecked(2) * Complex{re: Zero::zero(), im: self.twiddle.im};

    	butterfly2.perform_fft(&mut buffer[1..]);
    }
}
impl<T: FFTnum> FFTButterfly<T> for Butterfly3<T> {
    #[inline(always)]
    unsafe fn process_multi_inplace(&self, buffer: &mut [Complex<T>]) {
        for chunk in buffer.chunks_mut(self.len()) {
            self.perform_fft(chunk);
        }
    }
}
impl<T: FFTnum> FFTAlgorithm<T> for Butterfly3<T> {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_size(input, output, self.len());
        output.copy_from_slice(input);

        unsafe { self.perform_fft(output) };
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        output.copy_from_slice(input);

        unsafe { self.process_multi_inplace(output) };
    }
    #[inline(always)]
    fn len(&self) -> usize {
        3
    }
}




pub struct Butterfly4 {
	inverse: bool,
}
impl Butterfly4
{
	#[inline(always)]
    pub fn new(inverse: bool) -> Self {
        Self { inverse:inverse }
    }

	#[inline(always)]
    pub unsafe fn perform_fft<T: FFTnum>(&self, buffer: &mut [Complex<T>]) {
    	let butterfly2 = Butterfly2{};

		//we're going to hardcode a step of mixed radix
    	//aka we're going to do the six step algorithm

    	// step 1: transpose, which in this case just means swapping 2 elements
    	swap_unchecked(buffer, 1, 2);

    	// step 2: column FFTs
    	butterfly2.perform_fft(&mut buffer[..2]);
    	butterfly2.perform_fft(&mut buffer[2..]);

    	// step 3: apply twiddle factors (only one in this case, and it's either 0 + i or 0 - i)
    	let final_value = *buffer.get_unchecked(3);

    	*buffer.get_unchecked_mut(3) = if self.inverse {
        	Complex{re:-final_value.im, im: final_value.re}
        } else {
        	Complex{re: final_value.im, im:-final_value.re}
        };

        // step 4: transpose, which in this case just means swapping 2 elements
    	swap_unchecked(buffer, 1, 2);

        // step 5: row FFTs
        butterfly2.perform_fft(&mut buffer[..2]);
    	butterfly2.perform_fft(&mut buffer[2..]);

    	// step 6: transpose, which in this case just means swapping 2 elements
    	swap_unchecked(buffer, 1, 2);
    }
}
impl<T: FFTnum> FFTButterfly<T> for Butterfly4 {
    #[inline(always)]
    unsafe fn process_multi_inplace(&self, buffer: &mut [Complex<T>]) {
        for chunk in buffer.chunks_mut(4) {
            self.perform_fft(chunk);
        }
    }
}
impl<T: FFTnum> FFTAlgorithm<T> for Butterfly4 {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_size(input, output, 4);
        output.copy_from_slice(input);

        unsafe { self.perform_fft(output) };
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        output.copy_from_slice(input);

        unsafe { self.process_multi_inplace(output) };
    }
    #[inline(always)]
    fn len(&self) -> usize {
        4
    }
}




pub struct Butterfly5<T> {
	inner_fft_multiply: Box<[Complex<T>; 4]>,
	inverse: bool,
}
impl<T: FFTnum> Butterfly5<T> {
    pub fn new(inverse: bool) -> Self {

    	//we're going to hardcode a raders algorithm of size 5 and an inner FFT of size 4
    	let quarter: T = FromPrimitive::from_f32(0.25f32).unwrap();
    	let twiddle1: Complex<T> = twiddles::single_twiddle(1, 5, inverse) * quarter;
    	let twiddle2: Complex<T> = twiddles::single_twiddle(2, 5, inverse) * quarter;

    	//our primitive root will be 2, and our inverse will be 3. the powers of 3 mod 5 are 1.3.4.2, so we hardcode to use the twiddles in that order
    	let mut fft_data = [twiddle1, twiddle2.conj(), twiddle1.conj(), twiddle2];

    	let butterfly = Butterfly4::new(inverse);
    	unsafe { butterfly.perform_fft(&mut fft_data) };

        Self { 
        	inner_fft_multiply: Box::new(fft_data),
        	inverse: inverse,
        }
    }

    #[inline(always)]
    pub unsafe fn perform_fft(&self, buffer: &mut [Complex<T>]) {
    	//we're going to reorder the buffer directly into our scratch vec
    	//our primitive root is 2. the powers of 2 mod 5 are 1, 2,4,3 so use that ordering
	    let mut scratch = [*buffer.get_unchecked(1), *buffer.get_unchecked(2), *buffer.get_unchecked(4), *buffer.get_unchecked(3)];

    	//perform the first inner FFT
    	Butterfly4::new(self.inverse).perform_fft(&mut scratch);

    	//multiply the fft result with our precomputed data
    	for i in 0..4 {
    		scratch[i] = scratch[i] * self.inner_fft_multiply[i];
    	}

    	//perform the second inner FFT
    	Butterfly4::new(!self.inverse).perform_fft(&mut scratch);
    	//the first element of the output is the sum of the rest
    	let first_input = *buffer.get_unchecked_mut(0);
    	let mut sum = first_input;
    	for i in 1..5 {
    		sum = sum + *buffer.get_unchecked_mut(i);
    	}
    	*buffer.get_unchecked_mut(0) = sum;

    	//use the inverse root ordering to copy data back out
    	*buffer.get_unchecked_mut(1) = scratch[0] + first_input;
    	*buffer.get_unchecked_mut(3) = scratch[1] + first_input;
    	*buffer.get_unchecked_mut(4) = scratch[2] + first_input;
    	*buffer.get_unchecked_mut(2) = scratch[3] + first_input;
    }
}
impl<T: FFTnum> FFTButterfly<T> for Butterfly5<T> {
    #[inline(always)]
    unsafe fn process_multi_inplace(&self, buffer: &mut [Complex<T>]) {
        for chunk in buffer.chunks_mut(self.len()) {
            self.perform_fft(chunk);
        }
    }
}
impl<T: FFTnum> FFTAlgorithm<T> for Butterfly5<T> {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_size(input, output, self.len());
        output.copy_from_slice(input);

        unsafe { self.perform_fft(output) };
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        output.copy_from_slice(input);

        unsafe { self.process_multi_inplace(output) };
    }
    #[inline(always)]
    fn len(&self) -> usize {
        5
    }
}





pub struct Butterfly6<T> {
	butterfly3: Butterfly3<T>,
}
impl<T: FFTnum> Butterfly6<T> {

    pub fn new(inverse: bool) -> Self {
        Self { butterfly3: Butterfly3::new(inverse) }
    }
    pub fn inverse_of(fft: &Butterfly6<T>) -> Self {
        Self { butterfly3: Butterfly3::inverse_of(&fft.butterfly3) }
    }

    #[inline(always)]
    unsafe fn transpose_3x2_to_2x3(buffer: &mut [Complex<T>]) {
    	let temp = *buffer.get_unchecked(3);
		*buffer.get_unchecked_mut(3) = *buffer.get_unchecked(4);
		*buffer.get_unchecked_mut(4) = *buffer.get_unchecked(2);
		*buffer.get_unchecked_mut(2) = *buffer.get_unchecked(1);
		*buffer.get_unchecked_mut(1) = temp;
    }

	#[inline(always)]
    pub unsafe fn perform_fft(&self, buffer: &mut [Complex<T>]) {
		//since GCD(2,3) == 1 we're going to hardcode a step of the Good-Thomas algorithm to avoid twiddle factors

		// step 1: reorder the input directly into the scratch. normally there's a whole thing to compute this ordering
		//but thankfully we can just precompute it and hardcode it
		let mut scratch = [
			*buffer.get_unchecked(0),
			*buffer.get_unchecked(2),
			*buffer.get_unchecked(4),
			*buffer.get_unchecked(3),
			*buffer.get_unchecked(5),
			*buffer.get_unchecked(1),
		];

    	// step 2: column FFTs
    	self.butterfly3.perform_fft(&mut scratch[..3]);
    	self.butterfly3.perform_fft(&mut scratch[3..]);

    	// step 3: apply twiddle factors -- SKIPPED because good-thomas doesn't have twiddle factors :)

        // step 4: transpose
    	Self::transpose_3x2_to_2x3(&mut scratch);

        // step 5: row FFTs
        let butterfly2 = Butterfly2{};
        butterfly2.perform_fft(&mut scratch[..2]);
        butterfly2.perform_fft(&mut scratch[2..4]);
        butterfly2.perform_fft(&mut scratch[4..]);

    	// step 6: reorder the result back into the buffer. again we would normally have to do an expensive computation
    	// but instead we can precompute and hardcode the ordering
    	*buffer.get_unchecked_mut(0) = scratch[0];
    	*buffer.get_unchecked_mut(3) = scratch[1];
    	*buffer.get_unchecked_mut(4) = scratch[2];
    	*buffer.get_unchecked_mut(1) = scratch[3];
    	*buffer.get_unchecked_mut(2) = scratch[4];
    	*buffer.get_unchecked_mut(5) = scratch[5];
    }
}
impl<T: FFTnum> FFTButterfly<T> for Butterfly6<T> {
    #[inline(always)]
    unsafe fn process_multi_inplace(&self, buffer: &mut [Complex<T>]) {
        for chunk in buffer.chunks_mut(self.len()) {
            self.perform_fft(chunk);
        }
    }
}
impl<T: FFTnum> FFTAlgorithm<T> for Butterfly6<T> {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_size(input, output, self.len());
        output.copy_from_slice(input);

        unsafe { self.perform_fft(output) };
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        output.copy_from_slice(input);

        unsafe { self.process_multi_inplace(output) };
    }
    #[inline(always)]
    fn len(&self) -> usize {
        6
    }
}


pub struct Butterfly7<T> {
    inner_fft: Butterfly6<T>,
    inner_fft_multiply: [Complex<T>; 6]
}
impl<T: FFTnum> Butterfly7<T> {
    pub fn new(inverse: bool) -> Self {

        //we're going to hardcode a raders algorithm of size 5 and an inner FFT of size 4
        let sixth: T = FromPrimitive::from_f64(1f64/6f64).unwrap();
        let twiddle1: Complex<T> = twiddles::single_twiddle(1, 7, inverse) * sixth;
        let twiddle2: Complex<T> = twiddles::single_twiddle(2, 7, inverse) * sixth;
        let twiddle3: Complex<T> = twiddles::single_twiddle(3, 7, inverse) * sixth;

        //our primitive root will be 3, and our inverse will be 5. the powers of 5 mod 7 are 1,5,4,6,2,3, so we hardcode to use the twiddles in that order
        let mut fft_data = [twiddle1, twiddle2.conj(), twiddle3.conj(), twiddle1.conj(), twiddle2, twiddle3];

        let butterfly = Butterfly6::new(inverse);
        unsafe { butterfly.perform_fft(&mut fft_data) };

        Self { 
            inner_fft: butterfly,
            inner_fft_multiply: fft_data,
        }
    }

    #[inline(always)]
    pub unsafe fn perform_fft(&self, buffer: &mut [Complex<T>]) {
        //we're going to reorder the buffer directly into our scratch vec
        //our primitive root is 3. use 3^n mod 7 to determine which index to copy from
        let mut scratch = [
            *buffer.get_unchecked(3),
            *buffer.get_unchecked(2),
            *buffer.get_unchecked(6),
            *buffer.get_unchecked(4),
            *buffer.get_unchecked(5),
            *buffer.get_unchecked(1),
            ];

        //perform the first inner FFT
        self.inner_fft.perform_fft(&mut scratch);

        //multiply the fft result with our precomputed data
        for i in 0..6 {
            scratch[i] = scratch[i] * self.inner_fft_multiply[i];
        }

        //perform the second inner FFT
        let inverse6 = Butterfly6::inverse_of(&self.inner_fft);
        inverse6.perform_fft(&mut scratch);

        //the first element of the output is the sum of the rest
        let first_input = *buffer.get_unchecked(0);
        let mut sum = first_input;
        for i in 1..7 {
            sum = sum + *buffer.get_unchecked_mut(i);
        }
        *buffer.get_unchecked_mut(0) = sum;

        //use the inverse root ordering to copy data back out
        *buffer.get_unchecked_mut(5) = scratch[0] + first_input;
        *buffer.get_unchecked_mut(4) = scratch[1] + first_input;
        *buffer.get_unchecked_mut(6) = scratch[2] + first_input;
        *buffer.get_unchecked_mut(2) = scratch[3] + first_input;
        *buffer.get_unchecked_mut(3) = scratch[4] + first_input;
        *buffer.get_unchecked_mut(1) = scratch[5] + first_input;
    }
}
impl<T: FFTnum> FFTButterfly<T> for Butterfly7<T> {
    #[inline(always)]
    unsafe fn process_multi_inplace(&self, buffer: &mut [Complex<T>]) {
        for chunk in buffer.chunks_mut(7) {
            self.perform_fft(chunk);
        }
    }
}
impl<T: FFTnum> FFTAlgorithm<T> for Butterfly7<T> {
    fn process(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        verify_size(input, output, self.len());
        output.copy_from_slice(input);

        unsafe { self.perform_fft(output) };
    }
    fn process_multi(&self, input: &mut [Complex<T>], output: &mut [Complex<T>]) {
        output.copy_from_slice(input);

        unsafe { self.process_multi_inplace(output) };
    }
    #[inline(always)]
    fn len(&self) -> usize {
        7
    }
}




#[cfg(test)]
mod unit_tests {
    use std::rc::Rc;
	use super::*;
	use test_utils::{random_signal, compare_vectors};
	use algorithm::{DFTAlgorithm, RadersAlgorithm};
	use num::Zero;

	#[test]
	fn test_butterfly2() {
		let n = 5;
		const SIZE: usize = 2;

		let butterfly2 = Butterfly2{};
		let dft = DFTAlgorithm::new(SIZE, false);
		let dft_inverse = DFTAlgorithm::new(SIZE, true);

		let input_data = random_signal(n * SIZE);

		for (i, input) in input_data.chunks(SIZE).enumerate() {
            //compute expected values
            let mut expected_input = input.to_vec();
            let mut expected = [Zero::zero(); SIZE];
            dft.process(&mut expected_input, &mut expected);

            let mut expected_inverse_input = input.to_vec();
            let mut expected_inverse = [Zero::zero(); SIZE];
            dft_inverse.process(&mut expected_inverse_input, &mut expected_inverse);

            //test process method
            let mut process_input = input.to_vec();
            let mut actual = [Zero::zero(); SIZE];
			butterfly2.process(&mut process_input, &mut actual);

            assert!(compare_vectors(&actual, &expected), "forward, i = {}", i);
            assert!(compare_vectors(&actual, &expected_inverse), "inverse, i = {}", i);

            //test perform_fft method
            let mut inplace = input.to_vec();
			unsafe { butterfly2.perform_fft(&mut inplace) };
			
			assert!(compare_vectors(&expected, &inplace), "forward inplace, i = {}", i);
		}
	}

	//the tests for all butterflies except 2 will be identical except for the identifiers used and size
	//so it's ideal for a macro
	//butterfly 2 is different because it's the only one that doesn't care about forwards vs inverse
	macro_rules! test_butterfly_func {
		($test_name:ident, $struct_name:ident, $size:expr) => (
			#[test]
			fn $test_name() {
				let n = 5;
				const SIZE: usize = $size;
				let input_data = random_signal(n * SIZE);

				//test the forward direction
				let dft = DFTAlgorithm::new(SIZE, false);
				let fft = $struct_name::new(false);
				for (i, input) in input_data.chunks(SIZE).enumerate() {
					//compute expected values
                    let mut expected_input = input.to_vec();
                    let mut expected = [Zero::zero(); SIZE];
                    dft.process(&mut expected_input, &mut expected);

                    

                    //test process method
                    let mut process_input = input.to_vec();
                    let mut actual = [Zero::zero(); SIZE];
                    fft.process(&mut process_input, &mut actual);

                    if SIZE == 7 {

                        let inner_fft = Rc::new(DFTAlgorithm::new(SIZE-1, false));
                        let raders = RadersAlgorithm::new(SIZE, inner_fft, false);
                        let mut raders_input = input.to_vec();
                        let mut raders_output = [Zero::zero(); SIZE];

                        raders.process(&mut raders_input, &mut raders_output);

                        println!("raders output: {:?}", raders_output);
                    }

                    println!("actual output: {:?}", actual);

                    println!("expect output: {:?}", expected);

                    assert!(compare_vectors(&expected, &actual), "forward, i = {}", i);

                    //test perform_fft method
                    let mut inplace = input.to_vec();
                    unsafe { fft.perform_fft(&mut inplace) };
                    
                    assert!(compare_vectors(&expected, &inplace), "forward inplace, i = {}", i);
				}

				//make sure the inverse works too
				let dft_inverse = DFTAlgorithm::new(SIZE, true);
				let fft_inverse = $struct_name::new(true);
				for (i, input) in input_data.chunks(SIZE).enumerate() {	
					//compute expected values
                    let mut expected_inverse_input = input.to_vec();
                    let mut expected_inverse = [Zero::zero(); SIZE];
                    dft_inverse.process(&mut expected_inverse_input, &mut expected_inverse);

                    //test process method
                    let mut process_input = input.to_vec();
                    let mut actual = [Zero::zero(); SIZE];
                    fft_inverse.process(&mut process_input, &mut actual);

                    assert!(compare_vectors(&expected_inverse, &actual), "inverse, i = {}", i);

                    //test perform_fft method
                    let mut inplace = input.to_vec();
                    unsafe { fft_inverse.perform_fft(&mut inplace) };
                    
                    assert!(compare_vectors(&expected_inverse, &inplace), "inverse inplace, i = {}", i);
				}
			}
		)
	}
	test_butterfly_func!(test_butterfly3, Butterfly3, 3);
	test_butterfly_func!(test_butterfly4, Butterfly4, 4);
	test_butterfly_func!(test_butterfly5, Butterfly5, 5);
	test_butterfly_func!(test_butterfly6, Butterfly6, 6);
    test_butterfly_func!(test_butterfly7, Butterfly7, 7);
}
