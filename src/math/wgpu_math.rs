use crate::error::VectomancyError;
use crate::models::{FourierTerm, Point2D};
use bytemuck::{Pod, Zeroable};
use tracing::debug;
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct FFTParams {
    n: u32,
    log2_n: u32,
    stage: u32,
    direction: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct ComplexF32 {
    re: f32,
    im: f32,
}

pub fn perform_fft_gpu(
    points: &[Point2D],
    terms: usize,
) -> Result<Vec<FourierTerm>, VectomancyError> {
    pollster::block_on(perform_fft_gpu_async(points, terms))
}

async fn perform_fft_gpu_async(
    points: &[Point2D],
    terms: usize,
) -> Result<Vec<FourierTerm>, VectomancyError> {
    debug!("Performing GPU FFT. Terms: {}", terms);

    let original_n = points.len();
    let n = original_n.next_power_of_two();
    let log2_n = n.trailing_zeros();

    let mut gpu_data: Vec<ComplexF32> = points
        .iter()
        .map(|p| ComplexF32 {
            re: p.x as f32,
            im: p.y as f32,
        })
        .collect();

    gpu_data.resize(n, ComplexF32 { re: 0.0, im: 0.0 });

    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .map_err(|_| {
            VectomancyError::MathError("Failed to find an appropriate GPU adapter".to_string())
        })?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|e| VectomancyError::MathError(format!("Failed to create GPU device: {}", e)))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("FFT Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("fft.wgsl").into()),
    });

    let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Data Buffer"),
        contents: bytemuck::cast_slice(&gpu_data),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    });

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Params Buffer"),
        size: std::mem::size_of::<FFTParams>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind Group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: data_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Pipeline Layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let bit_rev_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Bit Reversal Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("bit_reversal"),
        compilation_options: Default::default(),
        cache: None,
    });

    let butterfly_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Butterfly Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("butterfly"),
        compilation_options: Default::default(),
        cache: None,
    });

    let mut initial_params = FFTParams {
        n: n as u32,
        log2_n,
        stage: 0,
        direction: 1.0,
    };

    queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&initial_params));
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Bit Reversal Encoder"),
    });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Bit Reversal Pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&bit_rev_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        let workgroups = (n as u32).div_ceil(256);
        cpass.dispatch_workgroups(workgroups, 1, 1);
    }
    queue.submit(Some(encoder.finish()));

    let mut last_submission = None;
    for stage in 0..log2_n {
        initial_params.stage = stage;
        queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&initial_params));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Butterfly Encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Butterfly Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&butterfly_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups = (n as u32 / 2).div_ceil(256);
            cpass.dispatch_workgroups(workgroups, 1, 1);
        }
        last_submission = Some(queue.submit(Some(encoder.finish())));
    }

    if let Some(index) = last_submission {
        let _ = device.poll(wgpu::PollType::Wait {
            submission_index: Some(index),
            timeout: Some(std::time::Duration::MAX),
        });
    }

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Staging Buffer"),
        size: data_buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Copy Encoder"),
    });
    encoder.copy_buffer_to_buffer(&data_buffer, 0, &staging_buffer, 0, data_buffer.size());
    let copy_index = queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: Some(copy_index),
        timeout: Some(std::time::Duration::MAX),
    });
    receiver.recv().unwrap().unwrap();

    let data = buffer_slice.get_mapped_range();
    let gpu_result: &[ComplexF32] = bytemuck::cast_slice(&data);

    let mut all_terms = Vec::with_capacity(n);
    let n_f64 = n as f64;

    for (i, val) in gpu_result.iter().enumerate() {
        let freq = if i <= n / 2 {
            i as f64
        } else {
            (i as f64) - n_f64
        };

        let re = val.re as f64;
        let im = val.im as f64;
        let magnitude = (re * re + im * im).sqrt() / n_f64;
        let phase = im.atan2(re);

        all_terms.push(FourierTerm {
            amplitude: magnitude,
            frequency: freq,
            phase,
        });
    }

    all_terms.sort_by(|a, b| {
        b.amplitude
            .partial_cmp(&a.amplitude)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut terms_vec = Vec::new();
    for term in all_terms
        .into_iter()
        .filter(|t| t.amplitude > 0.001)
        .take(terms)
    {
        terms_vec.push(term);
    }

    Ok(terms_vec)
}
