# **e2**

## Lightweight 2D rendering toolbox for WGPU

*What does it do?*

e2 makes your life easier when doing 2D game rendering with WGPU.
It provides utilities such as simplified resource creation and loading,
text rendering, batched rendering, sprite rendering, and more.

*What if I only want to use X feature, and nothing else?*

e2's API is designed to support that. Almost every type in e2 can be constructed
from existing WGPU handles. Everything has an escape hatch.

*Does it handle physics, audio, etc?*

No, and we don't plan supporting these things. e2 is exclusively for rendering.

*Can it do 3D?*

Not with the built-in renderers, but there's nothing stopping you from
creating your own 3D renderer using e2.

--

To give you a better picture of what exactly e2 does, look at the examples.
But in short:

```rs
let cx = e2::Context::new(&window, wgpu::Backends::PRIMARY);
cx.configure_surface(width, height, wgpu::PresentMode::Mailbox);

let mesh_pipe = e2::MeshRenderPipeline::new();
let mut renderer = e2::SpriteRenderer::new(&cx, &mesh_pipe);

let sampler = e2::SimleSampler::linear_clamp().create(&cx);
let tile = e2::ImageTexture::from_path("tile.png", true)?.create(&cx);

loop {
	let swapchain = cx.next_frame().unwrap();
	let view = swapchain.texture.create_view(&Default::default());

	let mut frame = e2::Frame::new(&cx);
	renderer.free();

	{
		let mut pass = e2::SimpleRenderPass {
			target: &view,
			resolve: None,
			clear: Some(e2::Color::BLACK),
			depth_stencil: None,
		}
		.begin(&mut frame);

		mesh_pipe.bind(&mut pass, &mut renderer);
		renderer.set_matrix(glam::Mat4::orthographic_rh(0., width as _, height as _, 0., 0., 1.));
		renderer.bind_sampler(&cx, &mut pass, &sampler);
		renderer.draw(&cx, &mut pass, &tile, e2::Rect::new(10., 10., 50., 50.), 0.);
	}

	frame.submit(&cx);
	swapchain.present();
}
```
