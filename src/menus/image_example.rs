use vulkano::pipeline::graphics::input_assembly::PrimitiveTopology;
use winit::event::Event;
use winit::event::KeyboardInput;
use winit::event::VirtualKeyCode;
use winit::event::WindowEvent;

use crate::buffer_contents;
use crate::events;
use crate::lost_code::is_pressed;
use crate::menu_rendering;
use crate::menus;

pub const MENU: menus::Data = menus::Data {
    start: |_user_storage, render_storage| {
        render_storage.entire_render_datas = vec![menu_rendering::EntireRenderData {
            render_buffers: menu_rendering::RenderBuffers {
                vertex_buffer: menu_rendering::VertexBuffer::Uv(
                    menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                        menu_rendering::FrequentAccessRenderBuffer {
                            buffer: vec![
                                buffer_contents::UvVertex {
                                    position: [0.0, 0.0, 0.0],
                                    uv: [0.0, 0.0],
                                };
                                4
                            ],
                        },
                    ),
                ),
                index_buffer: None,
                instance_buffer: None,
                shader_accessible_buffers: Some(menu_rendering::ShaderAccessibleBuffers {
                    uniform_buffer: Some(menu_rendering::UniformBuffer::CameraData2D(
                        menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                            menu_rendering::FrequentAccessRenderBuffer {
                                buffer: vec![
                                    crate::colour_2d_vertex_shader::CameraData2D {
                                        aspect_ratio: render_storage.aspect_ratio,
                                        position: [0.0, 0.0],
                                        scale: 1.0,
                                    };
                                    1
                                ],
                            },
                        ),
                    )),
                    image: Some(0),
                }),
            },
            render_call: menu_rendering::RenderCall {
                vertex_shader: menu_rendering::VertexShader::Uv2D,
                fragment_shader: menu_rendering::FragmentShader::Uv2D,
                topology: PrimitiveTopology::TriangleStrip,
                depth: true,
            },
        }];

        let entire_render_data = &mut render_storage.entire_render_datas[0];

        // TODO: create macro for assuming a buffer is of a type

        let vertex_buffer = match &mut entire_render_data.render_buffers.vertex_buffer {
            menu_rendering::VertexBuffer::Uv(ref mut vertex_buffer) => {
                if let menu_rendering::BufferTypes::FrequentAccessRenderBuffer(
                    ref mut vertex_buffer,
                ) = vertex_buffer
                {
                    vertex_buffer
                } else {
                    panic!()
                }
            }
            _ => panic!(),
        };

        vertex_buffer.buffer[0] = buffer_contents::UvVertex {
            // top left
            position: [-0.5, 0.5, 0.0],
            uv: [0.0, 1.0],
        };
        vertex_buffer.buffer[1] = buffer_contents::UvVertex {
            // top right
            position: [0.5, 0.5, 0.0],
            uv: [1.0, 1.0],
        };
        vertex_buffer.buffer[2] = buffer_contents::UvVertex {
            // bottom left
            position: [-0.5, -0.5, 0.0],
            uv: [0.0, 0.0],
        };
        vertex_buffer.buffer[3] = buffer_contents::UvVertex {
            // bottom right
            position: [0.5, -0.5, 0.0],
            uv: [1.0, 0.0],
        };

        render_storage.force_run_window_dependent_setup = true;
    },
    update: |_user_storage, _render_storage, _delta_time, _average_fps| {
        //println!("{}", average_fps);
    },
    fixed_update: (0.04, |user_storage, render_storage| {
        let entire_render_data = &mut render_storage.entire_render_datas[0];

        let motion = match user_storage.wasd_held {
            (true, false, false, false) => (0.0, -1.0),
            (false, false, true, false) => (0.0, 1.0),
            (false, false, false, true) => (1.0, 0.0),
            (false, true, false, false) => (-1.0, 0.0),

            (true, true, false, false) => (-0.7, -0.7),
            (true, false, false, true) => (0.7, -0.7),

            (false, true, true, false) => (-0.7, 0.7),
            (false, false, true, true) => (0.7, 0.7),

            _ => (0.0, 0.0),
        };

        let zoom_motion = match user_storage.zoom_held {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        let Some(uniform_buffer) = &mut entire_render_data.render_buffers.shader_accessible_buffers
        else {
            panic!()
        };
        let Some(ref mut uniform_buffer) = uniform_buffer.uniform_buffer else {
            panic!()
        };
        let menu_rendering::UniformBuffer::CameraData2D(uniform_buffer) = uniform_buffer else {
            panic!();
        };
        let menu_rendering::BufferTypes::FrequentAccessRenderBuffer(uniform_buffer) =
            uniform_buffer
        else {
            panic!()
        };

        let speed = 1.0 * MENU.fixed_update.0;
        let zoom_speed = 1.0 * MENU.fixed_update.0;

        uniform_buffer.buffer[0].position[0] += motion.0 * speed;
        uniform_buffer.buffer[0].position[1] += motion.1 * speed;
        uniform_buffer.buffer[0].scale += zoom_motion * zoom_speed;
        uniform_buffer.buffer[0].aspect_ratio = render_storage.aspect_ratio;
    }),
    handle_events: |user_storage, render_storage, event| match event {
        Event::WindowEvent {
            event: WindowEvent::KeyboardInput { input, .. },
            ..
        } => {
            on_keyboard_input(user_storage, render_storage, input);
        }
        _ => {}
    },
    create_pipelines: |_user_storage, _render_storage| vec![],
    on_draw: |_user_storage, _render_storage, _builder| {},
    end: |_user_storage, _render_storage| {},
};

fn on_keyboard_input(
    user_storage: &mut events::UserStorage,
    render_storage: &mut crate::RenderStorage,
    input: KeyboardInput,
) {
    if let Some(key_code) = input.virtual_keycode {
        match key_code {
            VirtualKeyCode::W => user_storage.wasd_held.0 = is_pressed(input.state),
            VirtualKeyCode::A => user_storage.wasd_held.1 = is_pressed(input.state),
            VirtualKeyCode::S => user_storage.wasd_held.2 = is_pressed(input.state),
            VirtualKeyCode::D => user_storage.wasd_held.3 = is_pressed(input.state),
            VirtualKeyCode::Up => user_storage.zoom_held.0 = is_pressed(input.state),
            VirtualKeyCode::Down => user_storage.zoom_held.1 = is_pressed(input.state),

            VirtualKeyCode::F => if is_pressed(input.state) {
                println!("Switching to example 1.");
                render_storage.menu = menus::Menu::Example1;
                (render_storage.menu.get_data().start)(user_storage, render_storage);
            },

            VirtualKeyCode::P => if is_pressed(input.state) {
                println!("Switching to example 3d.");
                render_storage.menu = menus::Menu::Example3D;
                (render_storage.menu.get_data().start)(user_storage, render_storage);
            },
            _ => (),
        }
    }
}