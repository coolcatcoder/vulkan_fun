use std::sync::Arc;

use vulkano::{
    buffer::BufferContents,
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::DepthStencilState,
            vertex_input::{Vertex as VertexTrait, VertexDefinition},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::Subpass,
};

pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/instanced_text_sdf/vertex_shader.vert",
    }
}
pub use vertex_shader::Font as FontUniform;

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/instanced_text_sdf/fragment_shader.frag",
    }
}

/// The vertex this shader requires.
#[derive(BufferContents, VertexTrait, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],
}

impl Vertex {
    /// Gets a Vec of Vertex from a gltf file.
    /// gltf should be a slice of bytes from the glb file. This may work with gltf files, not sure.
    /// Use mesh_index to specify what mesh you want to use.
    pub fn get_array_from_gltf(gltf: &[u8], mesh_index: usize) -> Vec<Vertex> {
        let (gltf, buffers, _) = gltf::import_slice(gltf).unwrap();

        let mesh = gltf.meshes().nth(mesh_index).unwrap();
        let primitive = mesh.primitives().next().unwrap();

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let mut vertices = vec![];

        for (position, uv) in reader
            .read_positions()
            .unwrap()
            .zip(reader.read_tex_coords(0).unwrap().into_f32())
        {
            println!("position: {:?}", position);
            println!("uv: {:?}", uv);

            vertices.push(Vertex {
                position: [position[0], position[1]],
                uv,
            });
        }

        vertices
    }
}

/// The instance this shader requires.
#[derive(BufferContents, VertexTrait, Copy, Clone, Debug)]
#[repr(C)]
pub struct Instance {
    #[format(R32G32_SFLOAT)]
    pub uv_offset: [f32; 2],

    #[format(R32G32B32A32_SFLOAT)]
    pub colour: [f32; 4],

    #[format(R32_SFLOAT)]
    pub smoothing: f32,

    #[format(R32_SFLOAT)]
    pub inverse_boldness: f32,

    #[format(R32G32B32_SFLOAT)]
    pub model_to_world_0: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub model_to_world_1: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub model_to_world_2: [f32; 3],
}

impl Instance {
    /// Constructs a new Instance.
    pub fn new(
        uv_offset: [f32; 2],
        colour: [f32; 4],
        smoothing: f32,
        inverse_boldness: f32,
        model_to_world: glam::Affine2,
    ) -> Instance {
        let model_to_world = glam::Mat3::from(model_to_world);
        Instance {
            uv_offset,
            colour,
            smoothing,
            inverse_boldness,
            model_to_world_0: model_to_world.x_axis.into(),
            model_to_world_1: model_to_world.y_axis.into(),
            model_to_world_2: model_to_world.z_axis.into(),
        }
    }

    pub fn set_model_to_world(&mut self, model_to_world: glam::Affine2) {
        let model_to_world = glam::Mat3::from(model_to_world);
        self.model_to_world_0 = model_to_world.x_axis.into();
        self.model_to_world_1 = model_to_world.y_axis.into();
        self.model_to_world_2 = model_to_world.z_axis.into();
    }
}

/// Gives you a GraphicsPipelineCreateInfo with everything specific to this shader.
/// You will still need to set non-shader-specifics, yourself.
///
/// This specifically sets:
///     stages,
///     vertex_input_state,
///     color_blend_state,
///     depth_stencil_state,
///     subpass,     
pub fn graphics_pipeline_create_info(
    device: Arc<Device>,
    subpass: Subpass,
) -> GraphicsPipelineCreateInfo {
    let vertex_shader_entrance = vertex_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();
    let fragment_shader_entrance = fragment_shader::load(device.clone())
        .unwrap()
        .entry_point("main")
        .unwrap();

    let vertex_input_state = [Vertex::per_vertex(), Instance::per_instance()]
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

    GraphicsPipelineCreateInfo {
        stages: stages.into_iter().collect(),
        vertex_input_state: Some(vertex_input_state),
        color_blend_state: Some(ColorBlendState::with_attachment_states(
            subpass.num_color_attachments(),
            ColorBlendAttachmentState {
                blend: Some(AttachmentBlend::alpha()),
                ..Default::default()
            },
        )),
        depth_stencil_state: Some(DepthStencilState {
            depth: None,
            depth_bounds: None,
            stencil: None,
            ..Default::default()
        }),
        subpass: Some(subpass.into()),
        ..GraphicsPipelineCreateInfo::layout(layout)
    }
}
