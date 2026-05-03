use clap::Parser;
use piston_window::wgpu_graphics::{Texture, TextureSettings};
use piston_window::{
    Button, PistonWindow, PressEvent, ReleaseEvent, UpdateEvent, WindowSettings, graphics::clear,
};
use sdl2::mixer;

mod active;
mod tetris;
mod tetromino;

// Embedded assets (included in the binary)
const BLOCK_PNG: &[u8] = include_bytes!("../bin/assets/block.png");
// gravitationalWaves by airtone (c)
// copyright 2016 Licensed under a Creative Commons Attribution Noncommercial  (3.0) license.
// http://dig.ccmixter.org/files/airtone/55021
const WAVES_OGG: &[u8] = include_bytes!("../bin/assets/airtone-gravitationalWaves.ogg");

#[derive(Parser)]
#[command(version, about, long_about = Some("A simple Tetris clone written in Rust"))]
struct CliArgs {
    /// Deteremines the number of lines to be filled randomly
    #[arg(short = 'i', long, default_value_t = 0)]
    initial_stack_size: usize,

    /// Turns off the music
    #[arg(short = 'o', long)]
    music_off: bool,

    /// Minified rendering for screens < 600x800
    #[arg(short = 'm', long)]
    mini: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let CliArgs {
        initial_stack_size,
        music_off,
        mini,
    } = CliArgs::parse();
    let (width, height) = (tetris::WINDOW_WIDTH, tetris::WINDOW_HEIGHT);
    let (width, height) = if mini {
        (width / 2, height / 2)
    } else {
        (width, height)
    };
    let mut window: PistonWindow = WindowSettings::new("Rusty Tetris", [width, height])
        .exit_on_esc(true)
        .build()?;

    let basic_block = {
        let img = image::load_from_memory(BLOCK_PNG)?.to_rgba8();
        Texture::from_image(
            &mut window.create_texture_context(),
            &img,
            &TextureSettings::new(),
        )
        .map_err(|e| e.to_string())?
    };

    let mut game = tetris::Tetris::new(
        if mini { 0.5 } else { 1.0 },
        basic_block,
        initial_stack_size,
    );

    let sdl;
    let _audio;
    let _mixer;
    let waves;
    if !music_off {
        sdl = sdl2::init()?;
        _audio = sdl.audio()?;
        _mixer = mixer::init(mixer::InitFlag::OGG);
        mixer::open_audio(
            mixer::DEFAULT_FREQUENCY,
            mixer::DEFAULT_FORMAT,
            mixer::DEFAULT_CHANNELS,
            1024,
        )?;
        mixer::allocate_channels(16);
        waves = mixer::Music::from_static_bytes(WAVES_OGG)?;
        let vol = (0.4_f64 * mixer::MAX_VOLUME as f64) as i32;
        mixer::Music::set_volume(vol);
        waves.play(-1)?;
    }

    while let Some(e) = window.next() {
        window.draw_2d(&e, |c, gl, _| {
            clear([1.0; 4], gl);
            game.render(&c, gl);
        });

        if let Some(uargs) = e.update_args() {
            game.update(&uargs);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            game.key_press(&key);
        }

        if let Some(Button::Keyboard(key)) = e.release_args() {
            game.key_release(&key);
        }
    }

    // Avoid potential destructor-time crashes in some EGL/driver stacks by
    // exiting immediately (bypasses running global destructors that can hit
    // driver bugs during cleanup). This is safer for a short-lived app.
    std::process::exit(0);
}
