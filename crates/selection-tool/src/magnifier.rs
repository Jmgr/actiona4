use bytemuck::{Pod, Zeroable};
use pixels::{PixelsContext, wgpu};

use crate::screenshot::Screenshot;

pub const MAGNIFIER_BOX_SIZE: f32 = 200.0;
pub const MAGNIFIER_OFFSET: f32 = 24.0;

/// Matches the WGSL `Params` struct layout (3 x vec4 = 48 bytes).
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MagnifierParams {
    /// xy = cursor in global pixels, zw = screenshot dimensions
    cursor_screen: [f32; 4],
    /// xy = magnifier box top-left in window pixels, zw = box size
    box_data: [f32; 4],
    /// x = zoom, yzw = padding
    zoom_pad: [f32; 4],
}

pub struct MagnifierPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group: wgpu::BindGroup,
    pub params_buffer: wgpu::Buffer,
}

pub struct MagnifierRenderInput {
    pub cursor_position: [f32; 2],
    pub desktop_origin: [f32; 2],
    pub screenshot_size: [f32; 2],
    pub window_size: [f32; 2],
    pub zoom: f32,
}

pub fn create_magnifier_pipeline(
    context: &PixelsContext,
    surface_format: wgpu::TextureFormat,
    screenshot: &Screenshot,
) -> MagnifierPipeline {
    let device = &context.device;
    let queue = &context.queue;

    // Upload screenshot as a GPU texture.
    let texture_size = wgpu::Extent3d {
        width: screenshot.width,
        height: screenshot.height,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("screenshot"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &screenshot.rgba,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(screenshot.width * 4),
            rows_per_image: Some(screenshot.height),
        },
        texture_size,
    );
    let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("mag_samp"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mag_params"),
        size: std::mem::size_of::<MagnifierParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("mag_shader"),
        source: wgpu::ShaderSource::Wgsl(include_str!("magnifier.wgsl").into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mag_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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
        label: Some("mag_bg"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: params_buffer.as_entire_binding(),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("mag_pl"),
        bind_group_layouts: &[Some(&bind_group_layout)],
        immediate_size: 0,
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("magnifier_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    });

    MagnifierPipeline {
        pipeline,
        bind_group,
        params_buffer,
    }
}

pub fn update_magnifier_params(
    queue: &wgpu::Queue,
    pipeline: &MagnifierPipeline,
    render_input: MagnifierRenderInput,
) {
    let magnifier_origin = compute_magnifier_origin(
        render_input.cursor_position,
        render_input.window_size,
        MAGNIFIER_BOX_SIZE,
        MAGNIFIER_OFFSET,
    );
    let params = MagnifierParams {
        cursor_screen: [
            render_input.cursor_position[0] + render_input.desktop_origin[0],
            render_input.cursor_position[1] + render_input.desktop_origin[1],
            render_input.screenshot_size[0],
            render_input.screenshot_size[1],
        ],
        box_data: [
            magnifier_origin[0],
            magnifier_origin[1],
            MAGNIFIER_BOX_SIZE,
            MAGNIFIER_BOX_SIZE,
        ],
        zoom_pad: [render_input.zoom, 0.0, 0.0, 0.0],
    };

    queue.write_buffer(&pipeline.params_buffer, 0, bytemuck::bytes_of(&params));
}

pub fn compute_magnifier_origin(
    cursor_position: [f32; 2],
    window_size: [f32; 2],
    magnifier_box_size: f32,
    magnifier_offset: f32,
) -> [f32; 2] {
    let x_position = if cursor_position[0] + magnifier_offset + magnifier_box_size < window_size[0]
    {
        cursor_position[0] + magnifier_offset
    } else {
        cursor_position[0] - magnifier_offset - magnifier_box_size
    };
    let y_position = if cursor_position[1] + magnifier_offset + magnifier_box_size < window_size[1]
    {
        cursor_position[1] + magnifier_offset
    } else {
        cursor_position[1] - magnifier_offset - magnifier_box_size
    };

    [x_position, y_position]
}
