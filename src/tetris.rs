#![allow(clippy::identity_op)]
use piston_window::wgpu_graphics::Texture;
use piston_window::{
    Key, UpdateArgs,
    graphics::{Context, Graphics, Image, Transformed, rectangle},
};
use rand::Rng;
use std::default::Default;

use crate::active::ActiveTetromino;
use crate::tetris::State::*;
use crate::tetromino::{Color, Rotation, Tetromino, TetrominoBag};

pub const WINDOW_WIDTH: u32 = 600;
pub const WINDOW_HEIGHT: u32 = 800;
pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;
static TILE_SIZE: f64 = 40.0;

pub const UPDATE_TIME: f64 = 0.15;

#[derive(PartialEq, Copy, Clone)]
enum State {
    Playing,
    Dropping,
    Defeated,
}

#[derive(Debug, Copy, Clone)]
enum KeyStateType {
    Released,
    Pressed,
    PressedLongTime,
}

#[derive(Debug)]
struct KeyState {
    state_type: KeyStateType,
    press_count: i32,
}

impl KeyState {
    fn new() -> KeyState {
        KeyState {
            state_type: KeyStateType::Released,
            press_count: 0,
        }
    }

    fn update_on_press(&mut self) {
        if let KeyStateType::Released = self.state_type {
            self.state_type = KeyStateType::Pressed;
            self.press_count += 1;
        }
    }

    fn update_on_release(&mut self) {
        match self.state_type {
            KeyStateType::Pressed => {
                self.state_type = KeyStateType::Released;
            }
            KeyStateType::PressedLongTime => {
                self.reset();
            }
            _ => {}
        }
    }

    fn update_by_time(&mut self) {
        match (self.state_type, self.press_count) {
            (KeyStateType::Pressed, 1) => {
                self.state_type = KeyStateType::PressedLongTime;
            }
            (KeyStateType::PressedLongTime, _) => {}
            _ => {
                self.reset();
            }
        }
    }

    fn is_active(&self) -> bool {
        self.press_count > 0
    }

    fn reset(&mut self) {
        self.state_type = KeyStateType::Released;
        self.press_count = 0;
    }
}

pub struct ControlState {
    rotate_right: KeyState,
    rotate_left: KeyState,
    move_left: KeyState,
    move_right: KeyState,
}

pub struct Tetris {
    initial_stack_size: usize,
    gravity_accumulator: f64,
    gravity_factor: f64,
    tetromino_count: usize,
    line_count: usize,
    active_tetromino: ActiveTetromino,
    next_shape: &'static Tetromino,
    board: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT],
    state: State,
    control_state: ControlState,
    time: f64,
    block: Texture,
    paused: bool,
    scale: f64,
    bag: TetrominoBag,
}

impl Tetris {
    pub fn new(scale: f64, texture: Texture, initial_stack_size: usize) -> Tetris {
        let stack_size = if initial_stack_size < BOARD_HEIGHT {
            initial_stack_size
        } else {
            BOARD_HEIGHT - 1
        };
        let mut bag = TetrominoBag::new();
        Tetris {
            initial_stack_size: stack_size,
            gravity_accumulator: 0.0,
            gravity_factor: 0.5,
            tetromino_count: 0,
            line_count: 0,
            active_tetromino: ActiveTetromino::new(bag.next().unwrap()),
            next_shape: bag.next().unwrap(),
            board: Tetris::create_board(stack_size),
            state: Playing,
            control_state: ControlState {
                rotate_right: KeyState::new(),
                rotate_left: KeyState::new(),
                move_left: KeyState::new(),
                move_right: KeyState::new(),
            },
            time: UPDATE_TIME,
            block: texture,
            paused: false,
            scale,
            bag,
        }
    }

    pub fn create_board(initial_stack_size: usize) -> [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT] {
        let mut board = [[Default::default(); BOARD_WIDTH]; BOARD_HEIGHT];
        if initial_stack_size > 0 {
            for y in 0usize..initial_stack_size {
                // set random cells within a row
                for x in (0usize..BOARD_WIDTH).filter(|_| rand::rng().random()) {
                    board[(BOARD_HEIGHT - 1) - y][x] = Some(Color::Grey);
                }
            }
        }
        board
    }

    fn print_digit(&mut self, digit: usize, x_offset: usize) {
        match digit {
            0 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][0 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][0 + x_offset] = Some(Color::Grey);
                self.board[3][2 + x_offset] = Some(Color::Grey);
                self.board[4][0 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][0 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
                self.board[5][2 + x_offset] = Some(Color::Grey);
            }
            1 => {
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][1 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][2 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][2 + x_offset] = Some(Color::Grey);
            }
            2 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[4][0 + x_offset] = Some(Color::Grey);
                self.board[5][0 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
                self.board[5][2 + x_offset] = Some(Color::Grey);
            }
            3 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][2 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
                self.board[5][0 + x_offset] = Some(Color::Grey);
            }
            4 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[2][0 + x_offset] = Some(Color::Grey);
                self.board[3][0 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[3][2 + x_offset] = Some(Color::Grey);
                self.board[4][1 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
            }
            5 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][0 + x_offset] = Some(Color::Grey);
                self.board[3][0 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[3][2 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
                self.board[5][0 + x_offset] = Some(Color::Grey);
            }
            6 => {
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][0 + x_offset] = Some(Color::Grey);
                self.board[3][0 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[4][0 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][0 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
                self.board[5][2 + x_offset] = Some(Color::Grey);
            }
            7 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[4][1 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
            }
            8 => {
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[2][0 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[4][0 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
            }
            9 => {
                self.board[1][0 + x_offset] = Some(Color::Grey);
                self.board[1][1 + x_offset] = Some(Color::Grey);
                self.board[1][2 + x_offset] = Some(Color::Grey);
                self.board[2][0 + x_offset] = Some(Color::Grey);
                self.board[2][2 + x_offset] = Some(Color::Grey);
                self.board[3][0 + x_offset] = Some(Color::Grey);
                self.board[3][1 + x_offset] = Some(Color::Grey);
                self.board[3][2 + x_offset] = Some(Color::Grey);
                self.board[4][2 + x_offset] = Some(Color::Grey);
                self.board[5][0 + x_offset] = Some(Color::Grey);
                self.board[5][1 + x_offset] = Some(Color::Grey);
                self.board[5][2 + x_offset] = Some(Color::Grey);
            }
            _ => {}
        }
    }

    fn show_result(&mut self) {
        self.board = [[Default::default(); BOARD_WIDTH]; BOARD_HEIGHT];
        let first_digit = self.line_count % 10;
        self.print_digit(first_digit, 5);
        let second_digit = (self.line_count - first_digit) / 10;
        self.print_digit(second_digit, 1);
    }

    fn gravity(&mut self, amount: f64) {
        self.gravity_accumulator += amount * self.gravity_factor;
        if self.gravity_accumulator >= 0.35 {
            self.gravity_accumulator = 0.0;
            if !self.active_tetromino.try_move_down(&self.board) {
                for &(x, y) in self.active_tetromino.as_points().iter() {
                    if y < self.board.len() && x < self.board[y].len() {
                        self.board[y][x] = Some(self.active_tetromino.get_color());
                    } else {
                        self.state = Defeated;
                    }
                }
                if self.state == Playing || self.state == Dropping {
                    self.state = Playing;
                    let mut board: [[Option<Color>; BOARD_WIDTH]; BOARD_HEIGHT] =
                        [[None; BOARD_WIDTH]; BOARD_HEIGHT];
                    let mut full_line_count = BOARD_HEIGHT;
                    for (new, old) in board.iter_mut().rev().zip(
                        self.board
                            .iter()
                            .rev()
                            .filter(|row| row.iter().any(|color| color.is_none())),
                    ) {
                        *new = *old;
                        full_line_count -= 1;
                    }
                    self.board = board;
                    self.line_count += full_line_count;
                    self.active_tetromino = ActiveTetromino::new(self.next_shape);
                    self.next_shape = self.bag.next().unwrap();
                    self.tetromino_count += 1;
                    if self.tetromino_count >= 10 {
                        self.tetromino_count = 0;
                        self.gravity_factor *= 1.1;
                    }
                }
            }
        }
    }

    fn play_again(&mut self) {
        self.state = Playing;
        self.gravity_accumulator = 0.0;
        self.tetromino_count = 0;
        self.line_count = 0;
        self.gravity_factor = 0.5;
        self.board = Tetris::create_board(self.initial_stack_size);
        self.bag.clear();
        self.active_tetromino = ActiveTetromino::new(self.bag.next().unwrap());
        self.next_shape = self.bag.next().unwrap();
    }

    pub fn render<G: Graphics<Texture = piston_window::wgpu_graphics::Texture>>(
        &mut self,
        c: &Context,
        g: &mut G,
    ) {
        let c = c.zoom(self.scale);
        fn pos(n: usize) -> f64 {
            n as f64 * TILE_SIZE
        }
        // render the board
        for y in 0usize..BOARD_HEIGHT {
            for x in 0usize..BOARD_WIDTH {
                if let Some(e) = self.board[y][x].as_ref() {
                    Image::new_color(e.as_rgba()).draw(
                        &self.block,
                        &Default::default(),
                        c.trans(pos(x), pos(y)).transform,
                        g,
                    )
                };
            }
        }
        if self.state != Defeated {
            for &(x, y) in self.active_tetromino.as_points().iter() {
                Image::new_color(self.active_tetromino.get_color().as_rgba()).draw(
                    &self.block,
                    &Default::default(),
                    c.trans(pos(x), pos(y)).transform,
                    g,
                );
            }
        }
        // render the side bar
        rectangle(
            Color::Grey.as_rgba(),
            [
                0.0,
                0.0,
                WINDOW_WIDTH as f64 - pos(BOARD_WIDTH),
                WINDOW_HEIGHT as f64,
            ], // rectangle
            c.trans(pos(BOARD_WIDTH), 0.0).transform,
            g,
        );
        for &(x, y) in self.next_shape.points(Rotation::R0).iter() {
            Image::new_color(self.next_shape.get_color().as_rgba()).draw(
                &self.block,
                &Default::default(),
                c.trans(pos(BOARD_WIDTH) + pos(x + 1), pos(y)).transform,
                g,
            );
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        if self.paused {
            return;
        }

        self.time += args.dt;

        if self.time > UPDATE_TIME {
            if self.control_state.rotate_right.is_active() {
                for _ in 0..self.control_state.rotate_right.press_count {
                    self.active_tetromino.try_rotate_right(&self.board);
                }

                self.control_state.rotate_right.update_by_time();
            }

            if self.control_state.rotate_left.is_active() {
                for _ in 0..self.control_state.rotate_left.press_count {
                    self.active_tetromino.try_rotate_left(&self.board);
                }
                self.control_state.rotate_left.update_by_time();
            }

            if self.control_state.move_left.is_active() {
                for _ in 0..self.control_state.move_left.press_count {
                    self.active_tetromino.try_move_left(&self.board);
                }
                self.control_state.move_left.update_by_time();
            }

            if self.control_state.move_right.is_active() {
                for _ in 0..self.control_state.move_right.press_count {
                    self.active_tetromino.try_move_right(&self.board);
                }
                self.control_state.move_right.update_by_time();
            }

            self.time -= UPDATE_TIME;
        }

        match self.state {
            Playing => self.gravity(args.dt),
            Dropping => self.gravity(0.12 + args.dt),
            Defeated => self.show_result(),
        }
    }

    pub fn drop_fully(&mut self) {
        while self.active_tetromino.try_move_down(&self.board) {}
    }

    pub fn key_press(&mut self, key: &Key) {
        match (self.state, key) {
            (Defeated, _) => {
                if key == &Key::F1 {
                    self.play_again()
                }
            }
            (Playing, &Key::P) => self.paused = !self.paused,
            (_, &Key::F1) => self.play_again(),
            (_, &Key::E) if !self.paused => self.control_state.rotate_right.update_on_press(),
            (_, &Key::Space) if !self.paused => {
                self.state = Dropping;
                self.drop_fully()
            }
            (_, &Key::Up) | (_, &Key::Q) if !self.paused => {
                self.control_state.rotate_left.update_on_press()
            }
            (_, &Key::Left) | (_, &Key::A) if !self.paused => {
                self.control_state.move_left.update_on_press()
            }
            (_, &Key::Right) | (_, &Key::D) if !self.paused => {
                self.control_state.move_right.update_on_press()
            }
            (_, &Key::Down) | (_, &Key::S) if !self.paused => self.state = Dropping,
            _ => {}
        }
    }

    pub fn key_release(&mut self, key: &Key) {
        match (self.state, key) {
            (Dropping, &Key::Down) | (Dropping, &Key::S) if !self.paused => self.state = Playing,
            (_, &Key::E) if !self.paused => self.control_state.rotate_right.update_on_release(),
            (_, &Key::Up) | (_, &Key::Q) if !self.paused => {
                self.control_state.rotate_left.update_on_release()
            }
            (_, &Key::Left) | (_, &Key::A) if !self.paused => {
                self.control_state.move_left.update_on_release()
            }
            (_, &Key::Right) | (_, &Key::D) if !self.paused => {
                self.control_state.move_right.update_on_release()
            }
            _ => {}
        }
    }
}
