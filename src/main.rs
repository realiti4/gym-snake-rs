extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use crate::piston::EventLoop;
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent, ButtonArgs, ButtonEvent, Button, ButtonState, Key};
use piston::window::WindowSettings;
use rand::Rng;

pub mod game;

use game::{Game, Segment};


fn main() {
    println!("Hello, world!");

    let opengl = OpenGL::V3_2;
    let windowx: u32 = 480;
    let windowy: u32 = 480;

    let mut window: Window = WindowSettings::new("Snake Game", [windowx, windowy])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    let gl = GlGraphics::new(opengl);
    
    let mut game = Game::new(gl);

    let event_settings = EventSettings::new().ups(15);
    let mut events = Events::new(event_settings);

    while let Some(event) = events.next(&mut window) {
        if let Some(args) = event.render_args() {
            game.render(&args);
        }

        if let Some(args) = event.update_args() {
            game.update(&args, &windowx, &windowy);
        }

        if game.game_over {
            println!("Game Over! Your score: {}", game.score);
            break;
        }

        if let Some(args) = event.button_args() {
            game.change_directions(&args);
        }
    }
}