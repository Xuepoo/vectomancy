use cudarc::cufft::safe::{CudaFft, FftDirection};
use cudarc::cufft::sys::{cufftType, double2};
use cudarc::driver::CudaContext;
use rustfft::num_complex::Complex;

pub fn perform_fft_gpu(points: &[Complex<f64>]) -> Result<Vec<Complex<f64>>, String> {
    // 1. Initialize Device (Context)
    let ctx = CudaContext::new(0).map_err(|e| format!("Failed to init CUDA device: {:?}", e))?;
    let stream = ctx.default_stream();
    let n = points.len();

    // 2. Initialize cuFFT Plan
    // nx: size, type: Z2Z, batch: 1
    let fft = CudaFft::plan_1d(n as i32, cufftType::CUFFT_Z2Z, 1, stream.clone())
        .map_err(|e| format!("Failed to create FFT plan: {:?}", e))?;

    // 3. Move data to GPU
    let host_slice_double2: &[double2] =
        unsafe { std::slice::from_raw_parts(points.as_ptr() as *const double2, n) };
    let mut dev_data = stream
        .clone_htod(host_slice_double2)
        .map_err(|e| format!("Failed to copy data to GPU: {:?}", e))?;

    // 4. Perform 1D Z2Z Forward FFT
    let mut dev_out = dev_data.clone(); // allocate new space
    fft.exec_z2z(&mut dev_data, &mut dev_out, FftDirection::Forward)
        .map_err(|e| format!("FFT forward execution failed: {:?}", e))?;

    // 5. Retrieve the result back to CPU
    let h_out_double2 = stream
        .clone_dtoh(&dev_out)
        .map_err(|e| format!("Failed to copy data back to CPU: {:?}", e))?;

    let h_out: Vec<Complex<f64>> = unsafe {
        std::slice::from_raw_parts(h_out_double2.as_ptr() as *const Complex<f64>, n).to_vec()
    };

    Ok(h_out)
}
