use std::sync::Arc;

use vulkano::{
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::{CullMode, FrontFace, RasterizationState},
            vertex_input::{Vertex, VertexDefinition},
            viewport::ViewportState,
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::Subpass,
};

use crate::{buffer_contents, math};

pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/colour_3d_instanced_shaders/vertex_shader.vert",
    }
}

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/colour_3d_instanced_shaders/fragment_shader.frag",
    }
}

/// Creates a basic pipeline that works with this shader sometimes.
/// There is no good way to make this work for most use cases currently.
/// Use at your own risk.
pub fn create_pipeline(device: Arc<Device>, subpass: Subpass) -> Arc<GraphicsPipeline> {
    let vertex_shader_entrance = vertex_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let fragment_shader_entrance = fragment_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();

    let vertex_input_state = [
        buffer_contents::Basic3DVertex::per_vertex(),
        buffer_contents::Colour3DInstance::per_instance(),
    ]
    .definition(&vertex_shader_entrance.info().input_interface)
    .unwrap();

    let stages = [
        PipelineShaderStageCreateInfo::new(vertex_shader_entrance),
        PipelineShaderStageCreateInfo::new(fragment_shader_entrance),
    ];

    let layout = PipelineLayout::new(
        device.clone(),
        PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
            .into_pipeline_layout_create_info(device.clone())
            .unwrap(),
    )
    .unwrap();

    GraphicsPipeline::new(
        device,
        None,
        GraphicsPipelineCreateInfo {
            stages: stages.into_iter().collect(),
            vertex_input_state: Some(vertex_input_state),
            input_assembly_state: Some(InputAssemblyState {
                topology: PrimitiveTopology::TriangleList,
                ..Default::default()
            }),
            viewport_state: Some(ViewportState::default()),
            rasterization_state: Some(RasterizationState {
                cull_mode: CullMode::Back,
                front_face: FrontFace::CounterClockwise,
                ..Default::default()
            }),
            multisample_state: Some(MultisampleState::default()),
            color_blend_state: Some(ColorBlendState::with_attachment_states(
                subpass.num_color_attachments(),
                ColorBlendAttachmentState {
                    blend: Some(AttachmentBlend::alpha()),
                    ..Default::default()
                },
            )),
            depth_stencil_state: Some(DepthStencilState {
                depth: Some(DepthState {
                    write_enable: true,
                    compare_op: CompareOp::Less,
                }),
                depth_bounds: None,
                stencil: None,
                ..Default::default()
            }),
            dynamic_state: [DynamicState::Viewport].into_iter().collect(),
            subpass: Some(subpass.into()),
            ..GraphicsPipelineCreateInfo::layout(layout)
        },
    )
    .unwrap()
}

/// A more user friendly version of [vertex_shader::CameraData3D]
pub struct Camera {
    pub position: [f32; 3],

    pub ambient_strength: f32,
    pub specular_strength: f32,
    pub light_colour: [f32; 3],
    pub light_position: [f32; 3],
}

impl Camera {
    /// Converts the Camera into the uniform representation, so that the shader can use it.
    pub fn to_uniform(&self) -> vertex_shader::CameraData3D {
        vertex_shader::CameraData3D {
            position: math::neg_3d(self.position),
            ambient_strength: self.ambient_strength,
            specular_strength: self.specular_strength.into(),
            light_colour: self.light_colour.into(),
            light_position: self.light_position.into(),
            camera_to_clip: todo!(),
            world_to_camera: todo!(),
        }
    }
}
