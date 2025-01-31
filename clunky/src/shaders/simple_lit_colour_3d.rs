use std::sync::Arc;

use vulkano::{
    buffer::BufferContents,
    device::Device,
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            depth_stencil::{CompareOp, DepthState, DepthStencilState},
            vertex_input::{Vertex as VertexTrait, VertexDefinition},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        PipelineLayout, PipelineShaderStageCreateInfo,
    },
    render_pass::Subpass,
};

use crate::math::{self, Degrees, Matrix4, Radians};

pub mod vertex_shader {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/simple_lit_colour_3d/vertex_shader.vert",
    }
}
pub use vertex_shader::Camera as CameraUniform;

pub mod fragment_shader {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/simple_lit_colour_3d/fragment_shader.frag",
    }
}

/// The vertex this shader requires.
#[derive(BufferContents, VertexTrait, Copy, Clone, Debug)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],

    #[format(R32G32B32A32_SFLOAT)]
    pub colour: [f32; 4],
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

        for (position, (normal, colour)) in reader.read_positions().unwrap().zip(
            reader
                .read_normals()
                .unwrap()
                .zip(reader.read_colors(0).unwrap().into_rgba_f32()),
        ) {
            vertices.push(Vertex {
                position,
                normal,
                colour,
            });
        }

        vertices
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

    let vertex_input_state = Vertex::per_vertex()
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
            depth: Some(DepthState {
                write_enable: true,
                compare_op: CompareOp::Less,
            }),
            depth_bounds: None,
            stencil: None,
            ..Default::default()
        }),
        subpass: Some(subpass.into()),
        ..GraphicsPipelineCreateInfo::layout(layout)
    }
}

/// A more user friendly version of [vertex_shader::CameraData3D]
/// Rotation is in degrees. I understand that this should be a quaternion.
#[derive(Debug, Clone)]
pub struct Camera {
    pub position: [f32; 3],
    pub rotation: [f32; 3],

    pub ambient_strength: f32,
    pub specular_strength: f32,
    pub light_colour: [f32; 3],
    pub light_position: [f32; 3],

    pub near_distance: f32,
    pub far_distance: f32,
    pub aspect_ratio: f32,
    pub fov_y: Radians<f32>,
}

impl Camera {
    /// Converts the Camera into the uniform representation, so that the shader can use it.
    pub fn to_uniform(&self) -> vertex_shader::Camera {
        vertex_shader::Camera {
            position: self.position,
            ambient_strength: self.ambient_strength,
            specular_strength: self.specular_strength.into(),
            light_colour: self.light_colour.into(),
            light_position: self.light_position.into(),
            camera_to_clip: Matrix4::from_perspective(
                self.fov_y,
                self.aspect_ratio,
                self.near_distance,
                self.far_distance,
            )
            .as_2d_array(),
            world_to_camera: (Matrix4::from_angle_x(Degrees(self.rotation[0]).to_radians())
                * Matrix4::from_angle_y(Degrees(self.rotation[1]).to_radians())
                * Matrix4::from_angle_z(Degrees(self.rotation[2]).to_radians())
                * Matrix4::from_translation(math::neg_3d(self.position)))
            .as_2d_array(),
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            rotation: [0.0; 3],

            ambient_strength: 0.3,
            specular_strength: 0.5,
            light_colour: [0.5; 3],
            light_position: [0.0, -10.0, 0.0],

            near_distance: 0.01,
            far_distance: 250.0,
            aspect_ratio: 1.0,
            fov_y: Radians(std::f32::consts::FRAC_PI_2),
        }
    }
}
