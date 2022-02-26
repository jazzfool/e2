// Classic snake game implemented in e2.
// This also demonstrates a possible higher-level abstraction for drawing; see Canvas.

use e2::{glam::*, wgpu};
use rand::Rng;
use std::time::Instant;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const GRID_COLS: u32 = 10;
const GRID_ROWS: u32 = 10;
const GRID_SIZE: u32 = 50;
const WIDTH: u32 = GRID_COLS * GRID_SIZE;
const HEIGHT: u32 = GRID_ROWS * GRID_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Right,
    Left,
}

impl Direction {
    fn opposite(&self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Right => Direction::Left,
            Direction::Left => Direction::Right,
        }
    }

    fn offset(&self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Right => (1, 0),
            Direction::Left => (-1, 0),
        }
    }
}

struct Snake {
    body: Vec<(u32, u32)>,
    length: u32,
    dir: Direction,
    timer: Option<Instant>,
}

impl Snake {
    fn new() -> Self {
        Snake {
            body: vec![(GRID_COLS / 2, GRID_ROWS / 2)],
            length: 1,
            dir: Direction::Left,
            timer: Some(Instant::now()),
        }
    }

    fn head(&self) -> (u32, u32) {
        *self.body.last().unwrap()
    }

    fn input(&mut self, key: VirtualKeyCode) {
        let dir = match key {
            VirtualKeyCode::Up => Some(Direction::Up),
            VirtualKeyCode::Down => Some(Direction::Down),
            VirtualKeyCode::Right => Some(Direction::Right),
            VirtualKeyCode::Left => Some(Direction::Left),
            _ => None,
        };
        if let Some(dir) = dir {
            if dir == self.dir.opposite() && self.length > 1 {
                // do nothing; we can't start moving into ourselves
                return;
            }
            self.dir = dir;
        }
    }

    fn update(&mut self) {
        let movement = if let Some(timer) = self.timer {
            (Instant::now() - timer).as_millis() > 300
        } else {
            true
        };

        if movement {
            self.timer = Some(Instant::now());
            let head = self.head();
            let (mut x, mut y) = (head.0 as i32, head.1 as i32);
            x += self.dir.offset().0;
            y += self.dir.offset().1;
            if x < 0 || x > GRID_COLS as _ || y < 0 || y > GRID_ROWS as _ {
                // snake went out of bounds
                // end game
                println!("game over!");
                std::process::exit(0);
            }

            self.body.push((x as _, y as _)); // push head
            if self.body.len() > self.length as _ {
                self.body.remove(0); // pop tail
            }
        }

        // check if head has collided with any part of the body
        let head = self.head();
        for &body in &self.body[..self.body.len() - 1] {
            if body == head {
                println!("game over!");
                std::process::exit(0);
            }
        }
    }

    fn draw(&self) -> Vec<e2::SpriteBatchColorDraw> {
        self.body
            .iter()
            .rev()
            .enumerate()
            .map(|(i, &(x, y))| {
                let deflate = ((i + 2) as f32 * 2.0).min(15.);
                e2::SpriteBatchColorDraw {
                    color: e2::Color::new(1.0, 0.58, 0.4, 1.0),
                    rect: e2::Rect::new(
                        x as f32 * GRID_SIZE as f32,
                        y as f32 * GRID_SIZE as f32,
                        GRID_SIZE as f32,
                        GRID_SIZE as f32,
                    )
                    .deflate(deflate, deflate),
                    rotation: 0.0,
                }
            })
            .collect()
    }
}

struct Food {
    at: (u32, u32),
}

impl Food {
    fn random() -> Self {
        Food {
            at: (
                rand::thread_rng().gen_range(0..GRID_COLS),
                rand::thread_rng().gen_range(0..GRID_ROWS),
            ),
        }
    }

    fn draw(&self) -> e2::SpriteBatchColorDraw {
        e2::SpriteBatchColorDraw {
            color: e2::Color::new(0.58, 0.89, 0.62, 1.0),
            rect: e2::Rect::new(
                self.at.0 as f32 * GRID_SIZE as f32,
                self.at.1 as f32 * GRID_SIZE as f32,
                GRID_SIZE as f32,
                GRID_SIZE as f32,
            ),
            rotation: 0.0,
        }
    }
}

struct Game {
    snake: Snake,
    food: Food,
}

impl Game {
    fn new() -> Self {
        Game {
            snake: Snake::new(),
            food: Food::random(),
        }
    }

    fn input(&mut self, key: VirtualKeyCode) {
        self.snake.input(key);
    }

    fn update(&mut self) {
        if self.snake.head() == self.food.at {
            // eat the food
            self.food = Food::random();
            self.snake.length += 1;
        }

        self.snake.update();
    }

    fn draw(&self) -> Vec<e2::SpriteBatchColorDraw> {
        let mut out = vec![];
        out.append(&mut self.snake.draw());
        out.push(self.food.draw());
        out
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("e2 snake")
        .with_inner_size(PhysicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false)
        .build(&event_loop)?;

    let cx = e2::Context::new(&window, wgpu::Backends::PRIMARY)?;
    cx.configure_surface(WIDTH, HEIGHT, wgpu::PresentMode::Mailbox);

    let mut game = Game::new();

    let batch_pipe = e2::BatchRenderPipeline::new(
        &cx,
        1,
        cx.surface.get_preferred_format(&cx.adapter).unwrap(),
        None,
        None,
    );
    let mut renderer = e2::SpriteBatchRenderer::new(&cx, &batch_pipe);
    let sampler = e2::SimpleSampler::linear_clamp().create(&cx);
    let ortho = Mat4::orthographic_rh(0., WIDTH as _, HEIGHT as _, 0., 0., 1.);

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::RedrawRequested(_) => {
                let swapchain = cx.next_frame().unwrap();
                let view = swapchain.texture.create_view(&Default::default());

                let mut frame = e2::Frame::new(&cx);

                {
                    let mut pass = e2::SimpleRenderPass {
                        target: &view,
                        resolve: None,
                        clear: Some(e2::Color::BLACK),
                        depth_stencil: None,
                    }
                    .begin(&mut frame);

                    game.update();

                    batch_pipe.bind(&mut pass, &mut renderer);
                    renderer.set_matrix(ortho);
                    renderer.bind_sampler(&cx, &mut pass, &sampler);
                    renderer.draw(&cx, &mut pass, &game.draw()[..]);
                }

                frame.submit(&cx);
                swapchain.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    if input.state == ElementState::Pressed {
                        game.input(input.virtual_keycode.unwrap());
                    }
                }
                _ => {}
            },
            _ => {}
        }
    });
}
