use std::collections::VecDeque;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{
    Button, ButtonArgs, ButtonEvent, ButtonState, Key, RenderArgs, RenderEvent, UpdateArgs,
    UpdateEvent,
};
use piston::window::WindowSettings;
use rand::prelude::*;

use crate::game;

#[derive(PartialEq, Clone, Copy, Debug)]
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

pub struct GameSettings {
    pub progressive_speed: bool,
    pub allow_teleport: bool,
}

pub struct Game {
    gl: GlGraphics,
    segments: Vec<Segment>,
    direction: Direction,
    next_direction: Option<Direction>,
    input_buffer: VecDeque<Direction>,
    max_buffer_size: usize,
    apple: Segment,
    size: i32,
    pub score: u32,
    pub game_over: bool,
    pub game_settings: GameSettings,
    // Timing control for progressive speed
    last_update_time: f64,
    update_interval: f64,
    base_speed: f64,
}

impl Game {
    pub fn new(gl: GlGraphics) -> Game {
        let size = 30;

        let segments = vec![
            Segment {
                x: 5 * size,
                y: 3 * size,
            },
            Segment {
                x: 4 * size,
                y: 3 * size,
            },
            Segment {
                x: 3 * size,
                y: 3 * size,
            },
        ];

        let apple = Segment {
            x: 10 * size,
            y: 10 * size,
        };
        let game_settings = GameSettings {
            progressive_speed: true, // Enable progressive speed by default
            allow_teleport: false,
        };

        Game {
            gl,
            segments,
            direction: Direction::Right,
            next_direction: None,
            input_buffer: VecDeque::new(),
            max_buffer_size: 2,
            apple,
            size: size,
            score: 0,
            game_over: false,
            game_settings: game_settings,
            // Initialize timing fields
            last_update_time: 0.0,
            update_interval: 1.0 / 15.0, // Start slow (8 updates per second)
            base_speed: 1.0 / 8.0,      // Base speed for reference
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

            for (index, segment_rect) in square_segments.iter().enumerate() {
                // Calculate gradient: head (index 0) is darkest, tail is lightest
                let gradient_factor = 1.0 - (index as f32 * 0.7 / self.segments.len() as f32);
                let segment_color = [
                    BLUE[0],
                    BLUE[1],
                    BLUE[2],
                    gradient_factor.max(0.7), // Minimum opacity of 0.3 so tail is still visible
                ];
                rectangle(segment_color, *segment_rect, transform, gl);
            }

            rectangle(RED, apple, transform, gl);
        });
    }
    pub fn update(&mut self, args: &UpdateArgs, windowx: &u32, windowy: &u32) {
        if self.game_over {
            return;
        }

        // Update timing
        self.last_update_time += args.dt;

        // Calculate current speed based on game settings
        let current_interval = if self.game_settings.progressive_speed {
            // Progressive speed: get faster as snake grows
            // Start slow and increase speed based on score/length
            let speed_multiplier = 1.0 + (self.score as f64 * 0.05); // Increase speed by 10% per apple eaten
            let max_speed_multiplier = 3.0; // Cap at 4x base speed
            let capped_multiplier = speed_multiplier.min(max_speed_multiplier);
            self.base_speed / capped_multiplier
        } else {
            self.update_interval
        };

        // Only update game state if enough time has passed
        if self.last_update_time < current_interval {
            return;
        }

        // Reset timing for next update
        self.last_update_time = 0.0;

        // Process next input from buffer
        if let Some(new_dir) = self.input_buffer.pop_front() {
            if check_directions(&self.direction, new_dir) {
                self.direction = new_dir;
            }
        }

        // Move snake based on current direction
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

        // Check for collisions
        if self.check_if_collision(&windowx, &windowy) {
            self.game_over = true;
            return;
        }

        // Check if apple was eaten
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
                y: y * self.size,
            };

            if !self.segments.contains(&candidate) && &candidate != &self.apple {
                self.apple.x = candidate.x;
                self.apple.y = candidate.y;
                break;
            }
        }
    }

    pub fn change_directions(&mut self, args: &ButtonArgs) {
        if args.state == ButtonState::Press {
            let pressed_direction = match args.button {
                Button::Keyboard(Key::Up) => Some(Direction::Up),
                Button::Keyboard(Key::Down) => Some(Direction::Down),
                Button::Keyboard(Key::Left) => Some(Direction::Left),
                Button::Keyboard(Key::Right) => Some(Direction::Right),
                _ => None,
            };

            // Handle special keys
            match args.button {
                Button::Keyboard(Key::P) => {
                    self.toggle_progressive_speed();
                    return;
                }
                _ => {}
            }

            // Todo: decide best method
            // Method 1
            if let Some(p_dir) = pressed_direction {
                // Add to buffer if it's a valid direction change
                if self.input_buffer.is_empty() {
                    // Check against current direction
                    if check_directions(&self.direction, p_dir) {
                        self.input_buffer.push_back(p_dir);
                    }
                } else {
                    // Check against the last buffered direction
                    if let Some(&last_buffered) = self.input_buffer.back() {
                        if check_directions(&last_buffered, p_dir) {
                            self.input_buffer.push_back(p_dir);
                        }
                    }
                }

                // Prevent buffer overflow
                if self.input_buffer.len() > self.max_buffer_size {
                    self.input_buffer.pop_front();
                }
            }

            // // Method 2
            // if let Some(p_dir) = pressed_direction {
            //     // Determine the direction to validate against:
            //     // If queue is not empty, use the last queued direction.
            //     // Otherwise, use the snake's current actual direction.
            //     let validation_direction = self.input_buffer.back().copied().unwrap_or(self.direction);

            //     if check_directions(&validation_direction, p_dir) {
            //         // Limit queue size to prevent too many buffered moves (e.g., 2)
            //         if self.input_buffer.len() < self.max_buffer_size {
            //             self.input_buffer.push_back(p_dir);
            //         }
            //     }
            // }
        }
    }

    pub fn toggle_progressive_speed(&mut self) {
        self.game_settings.progressive_speed = !self.game_settings.progressive_speed;
    }

    pub fn get_current_speed_info(&self) -> (f64, bool) {
        let current_interval = if self.game_settings.progressive_speed {
            let speed_multiplier = 1.0 + (self.score as f64 * 0.1);
            let max_speed_multiplier = 4.0;
            let capped_multiplier = speed_multiplier.min(max_speed_multiplier);
            self.base_speed / capped_multiplier
        } else {
            self.update_interval
        };
        (1.0 / current_interval, self.game_settings.progressive_speed)
    }
}

fn check_directions(dir1: &Direction, dir2: Direction) -> bool {
    if (matches!(dir1, Direction::Down) && matches!(dir2, Direction::Up))
        || (matches!(dir1, Direction::Up) && matches!(dir2, Direction::Down))
        || (matches!(dir1, Direction::Left) && matches!(dir2, Direction::Right))
        || (matches!(dir1, Direction::Right) && matches!(dir2, Direction::Left))
    {
        return false;
    }
    return true;
}

fn round_to_nearest_10(n: i32) -> i32 {
    let a = (n / 10) * 10 as i32;
    let b = a + 10;
    if n - a > b - n {
        return b;
    }
    return a;
}
