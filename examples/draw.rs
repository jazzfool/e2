// This example tests the functionality of sprite rendering.
// In particular, it stress tests batched rendering by rendering 10,000 sprites with DrawArray.
// Using DrawArray isn't necessary to leverage batched rendering, but it can greatly improve performance.

use e2::{glam::*, image, wgpu};
use rand::Rng;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: u32 = 800;
const HEIGHT: u32 = 600;

fn random_draw(ortho: Mat4, width: f32, height: f32) -> e2::BatchDraw {
    let color = e2::Color::new(rand::random(), rand::random(), rand::random(), 0.);
    let transform = ortho
        * Mat4::from_scale_rotation_translation(
            vec3(
                rand::thread_rng().gen_range(10.0..50.0),
                rand::thread_rng().gen_range(10.0..50.0),
                0.,
            ),
            Quat::from_rotation_z(rand::random::<f32>() * std::f32::consts::TAU),
            vec3(
                rand::thread_rng().gen_range(0.0..width),
                rand::thread_rng().gen_range(0.0..height),
                0.,
            ),
        );
    e2::BatchDraw {
        color,
        src_rect: e2::Rect::ONE,
        transform,
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("e2 example")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_resizable(true)
        .build(&event_loop)?;

    let cx = e2::Context::new(&window, wgpu::Backends::PRIMARY)?;
    cx.configure_surface(WIDTH, HEIGHT, wgpu::PresentMode::Mailbox);

    let mesh_pipe = e2::MeshRenderPipeline::new(
        &cx,
        1,
        cx.surface.get_preferred_format(&cx.adapter).unwrap(),
        Some(wgpu::BlendState::ALPHA_BLENDING),
        None,
    );
    let batch_pipe = e2::BatchRenderPipeline::new(
        &cx,
        1,
        cx.surface.get_preferred_format(&cx.adapter).unwrap(),
        None,
        None,
    );

    let mut mesh = e2::MeshRenderer::new(&cx, &mesh_pipe);
    let mut batch = e2::BatchRenderer::new(&batch_pipe);

    let sprite = e2::ImageTexture::from_image(
        &image::load_from_memory(include_bytes!("doge.png"))?.to_rgba8(),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    )
    .create(&cx);
    let rect = e2::Mesh::new(
        &cx,
        &[
            e2::Vertex {
                pos: [0., 0.],
                uv: [0., 0.],
            },
            e2::Vertex {
                pos: [1., 0.],
                uv: [1., 0.],
            },
            e2::Vertex {
                pos: [0., 1.],
                uv: [0., 1.],
            },
            e2::Vertex {
                pos: [1., 1.],
                uv: [1., 1.],
            },
        ],
        &[0, 2, 1, 2, 3, 1],
    );
    let sampler = e2::SimpleSampler::linear_clamp().create(&cx);
    let mut ortho = Mat4::orthographic_rh(0., WIDTH as _, HEIGHT as _, 0., 0., 1.);
    let mut draws = e2::DrawArray::new(
        &cx,
        &(0..10000)
            .map(|_| random_draw(ortho, WIDTH as _, HEIGHT as _))
            .collect::<Vec<_>>(),
    );

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let size = window.inner_size();
                if size.width * size.height == 0 {
                    return;
                }

                let swapchain = cx.next_frame().unwrap();
                let view = swapchain.texture.create_view(&Default::default());

                let mut frame = e2::Frame::new(&cx);
                mesh.free();
                batch.free();

                {
                    let mut pass = e2::SimpleRenderPass {
                        target: &view,
                        resolve: None,
                        clear: Some(e2::Color::BLACK),
                        depth_stencil: None,
                    }
                    .begin(&mut frame);

                    mesh_pipe.bind(&mut pass, &mut mesh);
                    mesh.bind_sampler(&cx, &mut pass, &sampler);
                    mesh.draw(
                        &cx,
                        &mut pass,
                        e2::MeshDraw {
                            mesh: &rect,
                            texture: &sprite,
                            color: e2::Color::WHITE,
                            src_rect: e2::Rect::ONE,
                            transform: ortho
                                * Mat4::from_scale_rotation_translation(
                                    vec3(size.width as _, size.height as _, 1.),
                                    Quat::IDENTITY,
                                    vec3(0., 0., 0.),
                                ),
                        },
                    );

                    batch_pipe.bind(&mut pass, &mut batch);
                    batch.bind_sampler(&cx, &mut pass, &sampler);
                    batch.draw_array(&cx, &mut pass, &rect, &sprite, &draws);
                }

                frame.submit(&cx);
                swapchain.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    if size.width * size.height > 0 {
                        cx.configure_surface(size.width, size.height, wgpu::PresentMode::Mailbox);
                        ortho = glam::Mat4::orthographic_rh(
                            0.,
                            size.width as _,
                            size.height as _,
                            0.,
                            0.,
                            1.,
                        );
                        draws.update(
                            &cx,
                            &(0..10000)
                                .map(|_| random_draw(ortho, size.width as _, size.height as _))
                                .collect::<Vec<_>>(),
                        );
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}
