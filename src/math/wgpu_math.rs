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

struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader: wgpu::ShaderModule,
}

static GPU_CONTEXT: std::sync::OnceLock<Result<GpuContext, String>> = std::sync::OnceLock::new();

pub fn init_context(power_pref: wgpu::PowerPreference) {
    let _ = GPU_CONTEXT.get_or_init(|| pollster::block_on(init_gpu(power_pref)));
}

async fn init_gpu(power_preference: wgpu::PowerPreference) -> Result<GpuContext, String> {
    let instance = wgpu::Instance::default();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference,
            force_fallback_adapter: false,
            compatible_surface: None,
        })
        .await
        .map_err(|_| "Failed to find an appropriate GPU adapter".to_string())?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|e| format!("Failed to create GPU device: {}", e))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("FFT Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("fft.wgsl").into()),
    });

    Ok(GpuContext {
        device,
        queue,
        shader,
    })
}

async fn perform_fft_gpu_async(
    points: &[Point2D],
    terms: usize,
) -> Result<Vec<FourierTerm>, VectomancyError> {
    debug!("Performing GPU FFT. Terms: {}", terms);

    let gpu_ctx_result = GPU_CONTEXT
        .get_or_init(|| pollster::block_on(init_gpu(wgpu::PowerPreference::HighPerformance)));

    let gpu_ctx = match gpu_ctx_result {
        Ok(ctx) => ctx,
        Err(e) => return Err(VectomancyError::MathError(e.clone())),
    };
    let device = &gpu_ctx.device;
    let queue = &gpu_ctx.queue;
    let shader = &gpu_ctx.shader;

    let original_n = points.len();
    let n = original_n.next_power_of_two();
    let log2_n = n.trailing_zeros();

    let mut gpu_data: Vec<ComplexF32> = Vec::with_capacity(n);
    if original_n == 0 {
        return Ok(vec![]);
    } else if original_n == 1 {
        gpu_data.resize(
            n,
            ComplexF32 {
                re: points[0].x as f32,
                im: points[0].y as f32,
            },
        );
    } else {
        for i in 0..n {
            let t = (i as f64) / ((n - 1) as f64) * ((original_n - 1) as f64);
            let idx = t.floor() as usize;
            let idx_next = (idx + 1).min(original_n - 1);
            let frac = (t - idx as f64) as f32;

            let p1 = points[idx];
            let p2 = points[idx_next];

            let re = p1.x as f32 * (1.0 - frac) + p2.x as f32 * frac;
            let im = p1.y as f32 * (1.0 - frac) + p2.y as f32 * frac;

            gpu_data.push(ComplexF32 { re, im });
        }
    }

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

pub fn perform_fft_batch_gpu(
    paths: &[&[Point2D]],
    terms: usize,
) -> Result<Vec<Vec<FourierTerm>>, VectomancyError> {
    pollster::block_on(perform_fft_batch_gpu_async(paths, terms))
}

async fn perform_fft_batch_gpu_async(
    paths: &[&[Point2D]],
    terms: usize,
) -> Result<Vec<Vec<FourierTerm>>, VectomancyError> {
    debug!(
        "Performing GPU FFT Batch. Paths: {}, Terms: {}",
        paths.len(),
        terms
    );

    if paths.is_empty() {
        return Ok(vec![]);
    }

    let gpu_ctx_result = GPU_CONTEXT
        .get_or_init(|| pollster::block_on(init_gpu(wgpu::PowerPreference::HighPerformance)));

    let gpu_ctx = match gpu_ctx_result {
        Ok(ctx) => ctx,
        Err(e) => return Err(VectomancyError::MathError(e.clone())),
    };
    let device = &gpu_ctx.device;
    let queue = &gpu_ctx.queue;
    let shader = &gpu_ctx.shader;

    let max_original_n = paths.iter().map(|p| p.len()).max().unwrap_or(0);
    if max_original_n == 0 {
        return Ok(vec![vec![]; paths.len()]);
    }

    let n = max_original_n.next_power_of_two();
    let log2_n = n.trailing_zeros();
    let num_paths = paths.len();

    let mut gpu_data: Vec<ComplexF32> = Vec::with_capacity(num_paths * n);

    for points in paths {
        let original_n = points.len();
        if original_n == 0 {
            gpu_data.resize(gpu_data.len() + n, ComplexF32 { re: 0.0, im: 0.0 });
        } else if original_n == 1 {
            gpu_data.resize(
                gpu_data.len() + n,
                ComplexF32 {
                    re: points[0].x as f32,
                    im: points[0].y as f32,
                },
            );
        } else {
            for i in 0..n {
                let t = (i as f64) / ((n - 1) as f64) * ((original_n - 1) as f64);
                let idx = t.floor() as usize;
                let idx_next = (idx + 1).min(original_n - 1);
                let frac = (t - idx as f64) as f32;

                let p1 = points[idx];
                let p2 = points[idx_next];

                let re = p1.x as f32 * (1.0 - frac) + p2.x as f32 * frac;
                let im = p1.y as f32 * (1.0 - frac) + p2.y as f32 * frac;

                gpu_data.push(ComplexF32 { re, im });
            }
        }
    }

    let data_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Batch Data Buffer"),
        contents: bytemuck::cast_slice(&gpu_data),
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC
            | wgpu::BufferUsages::COPY_DST,
    });

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Batch Params Buffer"),
        size: std::mem::size_of::<FFTParams>() as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Batch Bind Group Layout"),
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
        label: Some("Batch Bind Group"),
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
        label: Some("Batch Pipeline Layout"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let bit_rev_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Batch Bit Reversal Pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("bit_reversal"),
        compilation_options: Default::default(),
        cache: None,
    });

    let butterfly_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("Batch Butterfly Pipeline"),
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
        label: Some("Batch Bit Reversal Encoder"),
    });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Batch Bit Reversal Pass"),
            timestamp_writes: None,
        });
        cpass.set_pipeline(&bit_rev_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        let workgroups_x = (n as u32).div_ceil(256);
        cpass.dispatch_workgroups(workgroups_x, num_paths as u32, 1);
    }
    queue.submit(Some(encoder.finish()));

    let mut last_submission = None;
    for stage in 0..log2_n {
        initial_params.stage = stage;
        queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&initial_params));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Batch Butterfly Encoder"),
        });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("Batch Butterfly Pass"),
                timestamp_writes: None,
            });
            cpass.set_pipeline(&butterfly_pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            let workgroups_x = (n as u32 / 2).div_ceil(256);
            cpass.dispatch_workgroups(workgroups_x, num_paths as u32, 1);
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
        label: Some("Batch Staging Buffer"),
        size: data_buffer.size(),
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Batch Copy Encoder"),
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

    let mut all_results = Vec::with_capacity(num_paths);
    let n_f64 = n as f64;

    for path_idx in 0..num_paths {
        let offset = path_idx * n;
        let path_result = &gpu_result[offset..offset + n];

        let mut all_terms = Vec::with_capacity(n);
        for (i, val) in path_result.iter().enumerate() {
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

        all_terms.truncate(terms);
        all_results.push(all_terms);
    }

    Ok(all_results)
}
