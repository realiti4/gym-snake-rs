use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, ButtonArgs, ButtonEvent, ButtonState, Key, RenderArgs, RenderEvent, UpdateArgs,
    UpdateEvent,
};
use piston::window::WindowSettings;
use rand::prelude::*;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(PartialEq, Clone, Copy)]
pub struct Segment {
    pub x: i32,
    pub y: i32,
}

pub struct Game {
    gl: GlGraphics,
    segments: Vec<Segment>,
    direction: Direction,
    apple: Segment,
    size: i32,
    pub score: u32,
    pub game_over: bool,
}

impl Game {
    pub fn new(gl: GlGraphics) -> Game {
        let size = 30;

        let segments = vec![
            Segment { x: 5 * size, y: 3 * size },
            Segment { x: 4 * size, y: 3 * size },
            Segment { x: 3 * size, y: 3 * size },
        ];

        let apple = Segment { x: 10 * size, y: 10 * size };

        Game {
            gl,
            segments,
            direction: Direction::Right,
            apple,
            size: size,
            score: 0,
            game_over: false,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
        const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let mut square_segments: Vec<[f64; 4]> = Vec::new();
        for i in &self.segments {
            let x = i.x as f64;
            let y = i.y as f64;

            square_segments.push(rectangle::square(x, y, self.size as f64));
        }

        let apple = rectangle::square(self.apple.x as f64, self.apple.y as f64, self.size as f64);

        self.gl.draw(args.viewport(), |c, gl| {
            clear(WHITE, gl);
            let transform = c.transform.trans(0.0, 0.0).rot_deg(0.0);
            for i in square_segments {
                rectangle(BLUE, i, transform, gl);
            }
            rectangle(RED, apple, transform, gl);
        });
    }

    pub fn update(&mut self, _args: &UpdateArgs, windowx: &u32, windowy: &u32) {
        if self.game_over {
            return;
        }
        if matches!(self.direction, Direction::Up) {
            self.segments.insert(
                0,
                Segment {
                    x: self.segments[0].x,
                    y: self.segments[0].y - self.size,
                },
            );
        }
        if matches!(self.direction, Direction::Down) {
            self.segments.insert(
                0,
                Segment {
                    x: self.segments[0].x,
                    y: self.segments[0].y + self.size,
                },
            );
        }
        if matches!(self.direction, Direction::Left) {
            self.segments.insert(
                0,
                Segment {
                    x: self.segments[0].x - self.size,
                    y: self.segments[0].y,
                },
            );
        }
        if matches!(self.direction, Direction::Right) {
            self.segments.insert(
                0,
                Segment {
                    x: self.segments[0].x + self.size,
                    y: self.segments[0].y,
                },
            );
        }
        if self.check_if_collision(&windowx, &windowy) {
            self.game_over = true;
            return;
        }
        if self.segments[0].x == self.apple.x && self.segments[0].y == self.apple.y {
            self.gen_apple_coords(&windowx, &windowy);
            self.score += 1;
        } else {
            self.segments.pop();
        }
    }

    fn check_if_collision(&self, windowx: &u32, windowy: &u32) -> bool {
        let head = &self.segments[0];
    
        // Check boundary collision
        if head.x < 0 || head.y < 0 || head.x as u32 >= *windowx || head.y as u32 >= *windowy {
            return true;
        }
        
        // Check self-collision (head touching any body segment)
        self.segments[1..].contains(&head)
    }

    fn gen_apple_coords(&mut self, windowx: &u32, windowy: &u32) {
        let mut rng = rand::rng();
        let grid_width = *windowx / self.size as u32;
        let grid_height = *windowy / self.size as u32;

        loop {
            let x = rng.random_range(0..grid_width) as i32;
            let y = rng.random_range(0..grid_height) as i32;

            let candidate = Segment {
                x: x * self.size,
                y: y * self.size
            };

            if !self.segments.contains(&candidate) && &candidate != &self.apple {
                self.apple.x = candidate.x;
                self.apple.y = candidate.y;
                break;
            }
        }
    }

    pub fn change_directions(&mut self, args: &ButtonArgs){
        if args.state == ButtonState::Press {
            if args.button == Button::Keyboard(Key::Up) && check_directions(&self.direction, Direction::Up) {
                self.direction = Direction::Up;
            }
            if args.button == Button::Keyboard(Key::Down) && check_directions(&self.direction, Direction::Down) {
                self.direction = Direction::Down;
            }
            if args.button == Button::Keyboard(Key::Left) && check_directions(&self.direction, Direction::Left) {
                self.direction = Direction::Left;
            }
            if args.button == Button::Keyboard(Key::Right) && check_directions(&self.direction, Direction::Right) {
                self.direction = Direction::Right;
            }
        }
    }
}

fn check_directions(dir1: &Direction, dir2: Direction) -> bool{
    if (matches!(dir1, Direction::Down) && matches!(dir2, Direction::Up)) || (matches!(dir1 ,Direction::Up) && matches!(dir2 ,Direction::Down)) || (matches!(dir1, Direction::Left) && matches!(dir2, Direction::Right)) || (matches!(dir1, Direction::Right) && matches!(dir2, Direction::Left)) {
        return false;
    }
    return true;
}