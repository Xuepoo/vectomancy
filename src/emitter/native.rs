use crate::config::OutputFormat;
use crate::error::VectomancyError;
use crate::models::MathExpressionAST;
use lyon_tessellation::{
    math::Point, path::Path as LyonPath, StrokeOptions, StrokeTessellator, StrokeVertexConstructor,
    VertexBuffers,
};
use std::path::Path;
use std::sync::OnceLock;

struct RenderGpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
}

static RENDER_GPU_CONTEXT: OnceLock<Result<RenderGpuContext, String>> = OnceLock::new();

fn get_or_init_render_context(
    target_dimensions: (u32, u32),
) -> Result<&'static RenderGpuContext, VectomancyError> {
    let ctx =
        RENDER_GPU_CONTEXT.get_or_init(|| pollster::block_on(init_render_gpu(target_dimensions)));
    ctx.as_ref()
        .map_err(|e| VectomancyError::InvalidInput(format!("GPU context error: {}", e)))
}

async fn init_render_gpu(_target_dimensions: (u32, u32)) -> Result<RenderGpuContext, String> {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .map_err(|e| format!("Failed to request GPU adapter: {}", e))?;

    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor::default())
        .await
        .map_err(|e| format!("Failed to request device: {}", e))?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 4,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    });

    Ok(RenderGpuContext {
        device,
        queue,
        pipeline,
    })
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

struct VertexCtor {
    color: [f32; 4],
    target_dimensions: (u32, u32),
}

impl StrokeVertexConstructor<Vertex> for VertexCtor {
    fn new_vertex(&mut self, vertex: lyon_tessellation::StrokeVertex) -> Vertex {
        let p = vertex.position();
        let ndc_x = (p.x / self.target_dimensions.0 as f32) * 2.0 - 1.0;
        let ndc_y = 1.0 - (p.y / self.target_dimensions.1 as f32) * 2.0;
        Vertex {
            position: [ndc_x, ndc_y],
            color: self.color,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_to_image(
    ast: &MathExpressionAST,
    output_path: &Path,
    format: &OutputFormat,
    transparent: bool,
    original_dimensions: (u32, u32),
    target_dimensions: (u32, u32),
    stroke_width: f32,
    bit_depth: Option<u8>,
    color_space: Option<String>,
) -> Result<(), VectomancyError> {
    if let Some(cs) = color_space {
        tracing::debug!(
            "Color space {} requested, but ICC profiles are not fully supported yet.",
            cs
        );
    }

    pollster::block_on(render_wgpu(
        ast,
        output_path,
        format,
        transparent,
        original_dimensions,
        target_dimensions,
        stroke_width,
        bit_depth,
    ))
}

#[allow(clippy::too_many_arguments)]
async fn render_wgpu(
    ast: &MathExpressionAST,
    output_path: &Path,
    format: &OutputFormat,
    transparent: bool,
    original_dimensions: (u32, u32),
    target_dimensions: (u32, u32),
    stroke_width: f32,
    bit_depth: Option<u8>,
) -> Result<(), VectomancyError> {
    let ctx = get_or_init_render_context(target_dimensions)?;
    let device = &ctx.device;
    let queue = &ctx.queue;
    let render_pipeline = &ctx.pipeline;

    let texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: target_dimensions.0,
            height: target_dimensions.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 4,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("Multisampled Texture"),
        view_formats: &[],
    };
    let multisampled_texture = device.create_texture(&texture_desc);
    let multisampled_view =
        multisampled_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let resolve_texture_desc = wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: target_dimensions.0,
            height: target_dimensions.1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        label: Some("Resolve Texture"),
        view_formats: &[],
    };
    let resolve_texture = device.create_texture(&resolve_texture_desc);
    let resolve_view = resolve_texture.create_view(&wgpu::TextureViewDescriptor::default());

    let u32_size = std::mem::size_of::<u32>() as u32;
    let bytes_per_row = (u32_size * target_dimensions.0 + 255) & !255;
    let output_buffer_size = (bytes_per_row * target_dimensions.1) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        label: Some("Output Buffer"),
        mapped_at_creation: false,
    };
    let output_buffer = device.create_buffer(&output_buffer_desc);

    let mut geometry: VertexBuffers<Vertex, u32> = VertexBuffers::new();
    let mut tessellator = StrokeTessellator::new();
    let stroke_options = StrokeOptions::default().with_line_width(stroke_width);

    let scale_x = target_dimensions.0 as f32 / original_dimensions.0 as f32;
    let scale_y = target_dimensions.1 as f32 / original_dimensions.1 as f32;
    let scale = scale_x.min(scale_y);

    let offset_x = (target_dimensions.0 as f32 - original_dimensions.0 as f32 * scale) / 2.0;
    let offset_y = (target_dimensions.1 as f32 - original_dimensions.1 as f32 * scale) / 2.0;

    let transform_point = |x: f32, y: f32| -> Point {
        let tx = x * scale + offset_x;
        let ty = y * scale + offset_y;
        lyon_tessellation::math::point(tx, ty)
    };

    match ast {
        MathExpressionAST::Polyline {
            paths,
            bounding_box: _,
        } => {
            for path in paths {
                let mut builder = LyonPath::builder();
                let mut first = true;
                for pt in &path.data {
                    let p = transform_point(pt.x as f32, pt.y as f32);
                    if first {
                        builder.begin(p);
                        first = false;
                    } else {
                        builder.line_to(p);
                    }
                }
                if !first {
                    builder.end(false);
                }
                let lyon_path = builder.build();
                let color = if let Some(style) = &path.color_style {
                    style.to_solid_rgba()
                } else {
                    [0.0, 0.0, 0.0, 1.0]
                };
                tessellator
                    .tessellate_path(
                        &lyon_path,
                        &stroke_options,
                        &mut lyon_tessellation::BuffersBuilder::new(
                            &mut geometry,
                            VertexCtor {
                                color,
                                target_dimensions,
                            },
                        ),
                    )
                    .unwrap();
            }
        }
        MathExpressionAST::Spline {
            equations,
            bounding_box: _,
        } => {
            for path in equations {
                let mut builder = LyonPath::builder();
                let mut first = true;
                for eq in &path.data {
                    let steps = 50;
                    for i in 0..=steps {
                        let t = i as f64 / steps as f64;
                        let mut x = 0.0;
                        let mut y = 0.0;
                        for (j, coef) in eq.x_poly.iter().enumerate() {
                            x += coef * t.powi(j as i32);
                        }
                        for (j, coef) in eq.y_poly.iter().enumerate() {
                            y += coef * t.powi(j as i32);
                        }
                        let p = transform_point(x as f32, y as f32);
                        if first {
                            builder.begin(p);
                            first = false;
                        } else {
                            builder.line_to(p);
                        }
                    }
                }
                if !first {
                    builder.end(false);
                }
                let lyon_path = builder.build();
                let color = if let Some(style) = &path.color_style {
                    style.to_solid_rgba()
                } else {
                    [0.0, 0.0, 0.0, 1.0]
                };
                tessellator
                    .tessellate_path(
                        &lyon_path,
                        &stroke_options,
                        &mut lyon_tessellation::BuffersBuilder::new(
                            &mut geometry,
                            VertexCtor {
                                color,
                                target_dimensions,
                            },
                        ),
                    )
                    .unwrap();
            }
        }
        MathExpressionAST::Fourier {
            strokes,
            bounding_box: _,
        } => {
            let steps = target_dimensions.0.max(target_dimensions.1) as usize;
            for path in strokes {
                let mut builder = LyonPath::builder();
                let mut first = true;
                for i in 0..=steps {
                    let t = i as f64 / steps as f64;
                    let mut x = 0.0;
                    let mut y = 0.0;
                    for term in &path.data {
                        let angle = term.frequency * t * std::f64::consts::TAU + term.phase;
                        x += term.amplitude * angle.cos();
                        y += term.amplitude * angle.sin();
                    }
                    let p = transform_point(x as f32, y as f32);
                    if first {
                        builder.begin(p);
                        first = false;
                    } else {
                        builder.line_to(p);
                    }
                }
                if !first {
                    builder.end(false);
                }
                let lyon_path = builder.build();
                let color = if let Some(style) = &path.color_style {
                    style.to_solid_rgba()
                } else {
                    [0.0, 0.0, 0.0, 1.0]
                };
                tessellator
                    .tessellate_path(
                        &lyon_path,
                        &stroke_options,
                        &mut lyon_tessellation::BuffersBuilder::new(
                            &mut geometry,
                            VertexCtor {
                                color,
                                target_dimensions,
                            },
                        ),
                    )
                    .unwrap();
            }
        }
    }

    let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Vertex Buffer"),
        size: (geometry.vertices.len() * std::mem::size_of::<Vertex>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&geometry.vertices));

    let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Index Buffer"),
        size: (geometry.indices.len() * std::mem::size_of::<u32>()) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&geometry.indices));

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Render Encoder"),
    });

    {
        let clear_color = if transparent {
            wgpu::Color::TRANSPARENT
        } else {
            wgpu::Color::WHITE
        };

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &multisampled_view,
                resolve_target: Some(&resolve_view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            multiview_mask: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if !geometry.indices.is_empty() {
            render_pass.set_pipeline(render_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..geometry.indices.len() as u32, 0, 0..1);
        }
    }

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: &resolve_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(target_dimensions.1),
            },
        },
        wgpu::Extent3d {
            width: target_dimensions.0,
            height: target_dimensions.1,
            depth_or_array_layers: 1,
        },
    );

    let submission_index = queue.submit(Some(encoder.finish()));

    let buffer_slice = output_buffer.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission_index),
        timeout: None,
    });
    rx.recv()
        .unwrap()
        .map_err(|e| VectomancyError::InvalidInput(format!("Buffer map error: {}", e)))?;

    let data = buffer_slice.get_mapped_range();
    let mut pixels = Vec::with_capacity((target_dimensions.0 * target_dimensions.1 * 4) as usize);
    for row in 0..target_dimensions.1 {
        let start = (row * bytes_per_row) as usize;
        let end = start + (target_dimensions.0 * 4) as usize;
        pixels.extend_from_slice(&data[start..end]);
    }

    drop(data);
    output_buffer.unmap();

    let img = image::RgbaImage::from_raw(target_dimensions.0, target_dimensions.1, pixels)
        .ok_or_else(|| {
            VectomancyError::InvalidInput("Failed to create image from buffer".to_string())
        })?;

    let mut dyn_img = image::DynamicImage::ImageRgba8(img);

    if bit_depth == Some(16) {
        if transparent {
            let rgba16 = dyn_img.into_rgba16();
            dyn_img = image::DynamicImage::ImageRgba16(rgba16);
        } else {
            let rgb16 = dyn_img.into_rgb16();
            dyn_img = image::DynamicImage::ImageRgb16(rgb16);
        }
    } else if !transparent {
        let rgb8 = dyn_img.into_rgb8();
        dyn_img = image::DynamicImage::ImageRgb8(rgb8);
    }

    match format {
        OutputFormat::Png => {
            dyn_img
                .save_with_format(output_path, image::ImageFormat::Png)
                .map_err(|e| VectomancyError::InvalidInput(format!("Image save error: {}", e)))?;
        }
        OutputFormat::Jpg => {
            dyn_img
                .into_rgb8()
                .save_with_format(output_path, image::ImageFormat::Jpeg)
                .map_err(|e| VectomancyError::InvalidInput(format!("Image save error: {}", e)))?;
        }
        OutputFormat::Webp => {
            dyn_img
                .save_with_format(output_path, image::ImageFormat::WebP)
                .map_err(|e| VectomancyError::InvalidInput(format!("Image save error: {}", e)))?;
        }
        _ => {
            return Err(VectomancyError::InvalidInput(
                "Unsupported format for native rendering".to_string(),
            ))
        }
    }

    Ok(())
}
