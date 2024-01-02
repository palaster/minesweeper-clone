use std::thread;
use std::time::{Instant, Duration};

use sdl2::audio::{AudioSpecDesired, AudioQueue};
use sdl2::controller::Button;
use sdl2::event::Event;
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::BlendMode;

const NUMBER_OF_ROWS_AND_COLUMNS: usize = 24;
const NUMBER_OF_CELLS: usize = NUMBER_OF_ROWS_AND_COLUMNS * NUMBER_OF_ROWS_AND_COLUMNS;

const HEIGHT_PLAY_AREA_START: u16 = 64;
const WIDTH: u16 = 32 * (NUMBER_OF_ROWS_AND_COLUMNS as u16);
const HEIGHT: u16 = HEIGHT_PLAY_AREA_START + (32 * NUMBER_OF_ROWS_AND_COLUMNS as u16);

const fn one_d_to_two_d(coord: usize) -> (usize, usize) {
    (coord % NUMBER_OF_ROWS_AND_COLUMNS, coord / NUMBER_OF_ROWS_AND_COLUMNS)
}

/*
const fn one_d_to_two_d_x(coord: usize) -> usize {
    coord % NUMBER_OF_ROWS_AND_COLUMNS
}

const fn one_d_to_two_d_y(coord: usize) -> usize {
    coord / NUMBER_OF_ROWS_AND_COLUMNS
}
*/

const fn two_d_to_one_d(x: usize, y: usize) -> usize {
    y * NUMBER_OF_ROWS_AND_COLUMNS + x
}

const SAMPLE_RATE: u16 = 44_100;

const DEBUG_MINE: &[u8; 210] = include_bytes!("../assets/debug_mine.png");
const FLAGGED_MINE: &[u8; 211] = include_bytes!("../assets/flagged_mine.png");
const UNFLAGGED_MINE: &[u8; 128] = include_bytes!("../assets/unflagged_mine.png");
const REVEALED_MINE: &[u8; 128] = include_bytes!("../assets/revealed_mine.png");
const CURSOR: &[u8; 129] = include_bytes!("../assets/cursor.png");

const REVEALED_MINE_1: &[u8; 253] = include_bytes!("../assets/revealed_mine_1.png");
const REVEALED_MINE_2: &[u8; 583] = include_bytes!("../assets/revealed_mine_2.png");
const REVEALED_MINE_3: &[u8; 639] = include_bytes!("../assets/revealed_mine_3.png");
const REVEALED_MINE_4: &[u8; 410] = include_bytes!("../assets/revealed_mine_4.png");
const REVEALED_MINE_5: &[u8; 509] = include_bytes!("../assets/revealed_mine_5.png");
const REVEALED_MINE_6: &[u8; 736] = include_bytes!("../assets/revealed_mine_6.png");
const REVEALED_MINE_7: &[u8; 427] = include_bytes!("../assets/revealed_mine_7.png");
const REVEALED_MINE_8: &[u8; 792] = include_bytes!("../assets/revealed_mine_8.png");

#[derive(Clone, Copy)]
struct Cell {
    revealed: bool,
    flagged: bool,
    has_mine: bool,
    mines_around: u8,
}

impl Cell {
    fn new() -> Cell {
        Cell {
            revealed: false,
            flagged: false,
            has_mine: false,
            mines_around: 0,
        }
    }

    fn should_reveal(&self) -> bool {
        !self.has_mine && !self.revealed
    }

    fn reveal<F, G>(&mut self, flagged: F, revealed_mine: G)
    where
        F: FnOnce(),
        G: FnOnce()
    {
        self.revealed = true;
        if self.flagged {
            flagged();
            self.flagged = false;
        }
        if self.has_mine {
            revealed_mine();
        }
    }
}

#[derive(Clone, Copy)]
struct Field {
    cells: [Cell; NUMBER_OF_CELLS],
    flags_left: u8,
}

impl Field {
    fn new() -> Field {
        let mut cells = [Cell::new(); NUMBER_OF_CELLS];
        let mut mines_to_place = 99;

        // Place Mines
        while mines_to_place > 0 {
            for i in 0..cells.len() {
                if rand::random() {
                    continue;
                }
                if let Some(i_minus) = i.checked_sub(1) {
                    if cells[i_minus].has_mine {
                        continue;
                    }
                }
                if let Some(i_minus) = i.checked_sub(NUMBER_OF_ROWS_AND_COLUMNS) {
                    if cells[i_minus].has_mine {
                        continue;
                    }
                }
                if i + 1 < cells.len() && cells[i + 1].has_mine {
                    continue;
                }
                if i + NUMBER_OF_ROWS_AND_COLUMNS < cells.len() && cells[i + NUMBER_OF_ROWS_AND_COLUMNS].has_mine {
                    continue;
                }
                let cell = &mut cells[i];
                if rand::random() && !cell.has_mine && mines_to_place > 0 {
                    cell.has_mine = true;
                    mines_to_place -= 1;
                }
            }
        }

        // Increase mines_around
        for cell_index in 0..cells.len() {
            let (x, y) = one_d_to_two_d(cell_index);
            let mut cell = cells[cell_index];
            if !cell.has_mine {
                let mut number_of_mines = 0;
                let mut cells_check = Vec::<(usize, usize)>::new();

                // Left Side
                if let Some(x_value) = x.checked_sub(1) {
                    // 0 Cell
                    if let Some(y_value) = y.checked_sub(1) {
                        cells_check.push((x_value, y_value));
                    }
                    // 3 Cell
                    cells_check.push((x_value, y));
                    // 6 Cell
                    if y < 23 {
                        cells_check.push((x_value, y + 1));
                    }
                }

                // Middle
                // 1 Cell
                if let Some(y_value) = y.checked_sub(1) {
                    cells_check.push((x, y_value));
                }
                // 7 Cell
                if y < 23 {
                    cells_check.push((x, y + 1));
                }

                // Right Side
                if x < 23 {
                    // 2 Cell
                    if let Some(y_value) = y.checked_sub(1) {
                        cells_check.push((x + 1, y_value));
                    }
                    // 5 Cell
                    cells_check.push((x + 1, y));
                    // 8 Cell
                    if y < 23 {
                        cells_check.push((x + 1, y + 1));
                    }
                }
                for (x, y) in cells_check {
                    if cells[two_d_to_one_d(x, y)].has_mine {
                        number_of_mines += 1;
                    }
                }
                cell.mines_around = number_of_mines;
            }
            cells[cell_index] = cell;
        }

        Field {
            cells,
            flags_left: 99,
        }
    }

    fn reveal_surrounding_mines_from_index(&mut self, index: usize) {
        let (x, y) = one_d_to_two_d(index);
        self.reveal_surrounding_mines_from_x_y(x, y);
    }

    fn reveal_surrounding_mines_from_x_y(&mut self, x: usize, y: usize) {
        let mut cells = Vec::<(usize, usize)>::new();

        if let Some(x_value) = x.checked_sub(1) {
            // 0 Cell
            if let Some(y_value) = y.checked_sub(1) {
                cells.push((x_value, y_value));
            }
            // 3 Cell
            cells.push((x_value, y));
            // 6 Cell
            if y < 23 {
                cells.push((x_value, y + 1));
            }
        }

        // 1 Cell
        if let Some(y_value) = y.checked_sub(1) {
            cells.push((x, y_value));
        }
        // 7 Cell
        if y < 23 {
            cells.push((x, y + 1));
        }

        if x < 23 {
            // 2 Cell
            if let Some(y_value) = y.checked_sub(1) {
                cells.push((x + 1, y_value));
            }
            // 5 Cell
            cells.push((x + 1, y));
            // 8 Cell
            if y < 23 {
                cells.push((x + 1, y + 1));
            }
        }
        for (x, y) in cells {
            let cell = &mut self.cells[two_d_to_one_d(x, y)];
            if cell.should_reveal() && !cell.has_mine {
                cell.reveal(|| self.flags_left += 1, || {});
                if cell.mines_around == 0 {
                    self.reveal_surrounding_mines_from_x_y(x, y);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Game {
    should_die: bool,
    field: Field,
    current_selection: usize,
    inputs: [(bool, u8); 8],
}

impl Game {
    fn new() -> Game {
        Game {
            should_die: false,
            field: Field::new(),
            current_selection: 0,
            inputs: [(false, 0); 8],
        }
    }

    fn update(&mut self) {
        /*
        for (i, (is_down, count)) in self.inputs[0..4].iter_mut().enumerate() {
            println!("{} - {}", i, is_down);
            if *is_down {
                match i {
                    0 => {
                        let new_selction = self.current_selection + 1;
                        self.current_selection = if new_selction % 24 == 0 {
                            new_selction - 24
                        } else {
                            new_selction
                        }
                    },
                    1 => {
                        self.current_selection = if self.current_selection % 24 == 0 {
                            self.current_selection + 23
                        } else {
                            self.current_selection - 1
                        }
                    },
                    2 => {
                        self.current_selection = match self.current_selection.checked_sub(24) {
                            Some(t) => t,
                            None => (self.current_selection % 24) + 552,
                        };
                    },
                    3 => {
                        self.current_selection = if (self.current_selection + 24) > 576 {
                            self.current_selection % 24
                        } else {
                            self.current_selection + 24
                        }
                    },
                    _ => {},
                }
            }
        }
        println!("Current Selection: {}", self.current_selection);
        */
    }

    fn update_input(&mut self, is_down: bool, input: usize) {
        /*
            2 == UP
            1 == LEFT
            3 == DOWN
            0 == RIGHT
            5 == B
            4 == A
            6 == SELECT
            7 == START
        */
        let mut cause_event = false;
        if is_down {
            self.inputs[input] = (true, 4);
        } else if self.inputs[input].0 {
            cause_event = true;
            self.inputs[input] = (false, 0);
        }
        if !cause_event {
            return;
        }
        match input {
            0 => {
                let new_selction = self.current_selection + 1;
                self.current_selection = if new_selction % NUMBER_OF_ROWS_AND_COLUMNS == 0 {
                    new_selction - NUMBER_OF_ROWS_AND_COLUMNS
                } else {
                    new_selction
                }
            },
            1 => {
                self.current_selection = if self.current_selection % NUMBER_OF_ROWS_AND_COLUMNS == 0 {
                    self.current_selection + NUMBER_OF_ROWS_AND_COLUMNS - 1
                } else {
                    self.current_selection - 1
                }
            },
            2 => {
                self.current_selection = match self.current_selection.checked_sub(NUMBER_OF_ROWS_AND_COLUMNS) {
                    Some(t) => t,
                    None => (self.current_selection % NUMBER_OF_ROWS_AND_COLUMNS) + (NUMBER_OF_CELLS - NUMBER_OF_ROWS_AND_COLUMNS),
                };
            },
            3 => {
                self.current_selection = if (self.current_selection + NUMBER_OF_ROWS_AND_COLUMNS) > NUMBER_OF_CELLS {
                    self.current_selection % NUMBER_OF_ROWS_AND_COLUMNS
                } else {
                    self.current_selection + NUMBER_OF_ROWS_AND_COLUMNS
                }
            },
            4 => {
                if !is_down {
                    // Reveal Cell
                    self.reveal_from_index(self.current_selection);
                }
            }
            5 => {
                if !is_down {
                    // Flag Cell
                    self.flag_from_index(self.current_selection);
                }
            }
            _ => {},
        }
        println!("Current Selection: {}", self.current_selection);
    }

    fn reveal_from_index(&mut self, index: usize) {
        let cell = &mut self.field.cells[index];
        if !cell.revealed {
            if !cell.has_mine {
                cell.revealed = true;
                self.field.flags_left += 1;
                cell.flagged = false;
                if cell.mines_around == 0 {
                    self.field.reveal_surrounding_mines_from_index(index);
                }
            } else {
                self.should_die = true;
            }
        }
    }

    fn flag_from_index(&mut self, index: usize) {
        let cell = &mut self.field.cells[index];
        if !cell.revealed {
            if cell.flagged {
                self.field.flags_left += 1;
                cell.flagged = false;
            } else if self.field.flags_left > 0 {
                self.field.flags_left -= 1;
                cell.flagged = true;
            }
        }
    }
}

fn main() {
    let sdl_context = sdl2::init().expect("Couldn't init sdl");
    let video_subsystem = sdl_context.video().expect("Couldn't init sdl video");
    let audio_subsystem = sdl_context.audio().expect("Couldn't init sdl audio");
    let game_controller_subsystem = sdl_context.game_controller().expect("Couldn't init sdl game_controller");

    let window = video_subsystem.window("Minesweeper", WIDTH.into(), HEIGHT.into())
        .position_centered()
        .build()
        .expect("Couldn't create window from video");

    let mut canvas = window.into_canvas()
        .accelerated()
        .build()
        .expect("Couldn't create canvas from window");

    let texture_creator = canvas.texture_creator();

    let debug_mine_texture = texture_creator.load_texture_bytes(DEBUG_MINE).expect("Couldn't create texture from DEBUG_MINE");
    let flagged_mine_texture = texture_creator.load_texture_bytes(FLAGGED_MINE).expect("Couldn't create texture from FLAGGED_MINE");
    let unflagged_mine_texture = texture_creator.load_texture_bytes(UNFLAGGED_MINE).expect("Couldn't create texture from UNFLAGGED_MINE");
    let revealed_mine_texture = texture_creator.load_texture_bytes(REVEALED_MINE).expect("Couldn't create texture from REVEALED_MINE");
    let cursor_texture = texture_creator.load_texture_bytes(CURSOR).expect("Couldn't create texture from CURSOR");

    let revealed_mine_1_texture = texture_creator.load_texture_bytes(REVEALED_MINE_1).expect("Couldn't create texture from REVEALED_MINE_1");
    let revealed_mine_2_texture = texture_creator.load_texture_bytes(REVEALED_MINE_2).expect("Couldn't create texture from REVEALED_MINE_2");
    let revealed_mine_3_texture = texture_creator.load_texture_bytes(REVEALED_MINE_3).expect("Couldn't create texture from REVEALED_MINE_3");
    let revealed_mine_4_texture = texture_creator.load_texture_bytes(REVEALED_MINE_4).expect("Couldn't create texture from REVEALED_MINE_4");
    let revealed_mine_5_texture = texture_creator.load_texture_bytes(REVEALED_MINE_5).expect("Couldn't create texture from REVEALED_MINE_5");
    let revealed_mine_6_texture = texture_creator.load_texture_bytes(REVEALED_MINE_6).expect("Couldn't create texture from REVEALED_MINE_6");
    let revealed_mine_7_texture = texture_creator.load_texture_bytes(REVEALED_MINE_7).expect("Couldn't create texture from REVEALED_MINE_7");
    let revealed_mine_8_texture = texture_creator.load_texture_bytes(REVEALED_MINE_8).expect("Couldn't create texture from REVEALED_MINE_8");

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(2),
        samples: None,
    };

    let device: AudioQueue<f32> = audio_subsystem.open_queue(None, &desired_spec).expect("Couldn't get a desired audio device");
    device.resume();

    let number_of_joystics = game_controller_subsystem.num_joysticks().expect("Couldn't find any joysticks");
    let _controller = (0..number_of_joystics)
        .find_map(|id| {
            if !game_controller_subsystem.is_game_controller(id) {
                return None;
            }
            game_controller_subsystem.open(id).ok()
        });

    let mut event_pump = sdl_context.event_pump().expect("Couldn't get event_pump from sdl_context");

    let mut game = Game::new();

    let mut previous_instant: Instant = Instant::now();
    let mut current_instant: Instant;
    
    let frame_per_second: Duration = Duration::from_secs_f64(1.0/60.0);

    'running: loop {
        game.update();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event:: MouseButtonDown { mouse_btn, x, y, .. } => {
                    match mouse_btn {
                        MouseButton::Left => {
                            println!("Left Button Down - x: {} - y: {}", x, y);
                        },
                        MouseButton::Right => {
                            println!("Right Button Down - x: {} - y: {}", x, y);
                        },
                        _ => {},
                    }
                    // TODO: Mouse Button Down
                },
                Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    let y = y - HEIGHT_PLAY_AREA_START as i32;
                    if y >= 0 {
                        let pixel_to_2d_y = y / 32;
                        let pixel_to_2d_x = x / 32;
                        let cell_number = pixel_to_2d_y * (NUMBER_OF_ROWS_AND_COLUMNS as i32) + pixel_to_2d_x;
                        if let Ok(cell_number) = usize::try_from(cell_number) {
                            match mouse_btn {
                                MouseButton::Left => {
                                    game.reveal_from_index(cell_number);
                                },
                                MouseButton::Right => {
                                    game.flag_from_index(cell_number);
                                },
                                _ => {},
                            }
                        }
                    }
                    // TODO: Mouse Button Up
                },
                Event::KeyDown { keycode: Some(key_down), repeat: false, .. } => {
                    let key_code: i8 = match key_down {
                        Keycode::W => 2, // UP
                        Keycode::A => 1, // LEFT
                        Keycode::S => 3, // DOWN
                        Keycode::D => 0, // RIGHT
                        Keycode::H => 5, // B
                        Keycode::U => 4, // A
                        Keycode::B => 6, // SELECT
                        Keycode::N => 7, // START
                        _ => -1,
                    };
                    if let Ok(key_code) = usize::try_from(key_code) {
                        game.update_input(true, key_code);
                    }
                },
                Event::KeyUp { keycode: Some(key_up), repeat: false, .. } => {
                    let key_code: i8 = match key_up {
                        Keycode::W => 2, // UP
                        Keycode::A => 1, // LEFT
                        Keycode::S => 3, // DOWN
                        Keycode::D => 0, // RIGHT
                        Keycode::H => 5, // B
                        Keycode::U => 4, // A
                        Keycode::B => 6, // SELECT
                        Keycode::N => 7, // START
                        _ => -1,
                    };
                    if let Ok(key_code) = usize::try_from(key_code) {
                        game.update_input(true, key_code);
                    }
                },
                Event::ControllerButtonDown { button, .. } => {
                    let key_code: i8 = match button {
                        Button::DPadUp => 2, // UP
                        Button::DPadLeft => 1, // LEFT
                        Button::DPadDown => 3, // DOWN
                        Button::DPadRight => 0, // RIGHT
                        Button::A => 5, // B
                        Button::B => 4, // A
                        Button::Back => 6, // SELECT
                        Button::Start => 7, // START
                        _ => -1,
                    };
                    if let Ok(key_code) = usize::try_from(key_code) {
                        game.update_input(true, key_code);
                    }
                },
                Event::ControllerButtonUp { button, .. } => {
                    let key_code: i8 = match button {
                        Button::DPadUp => 2, // UP
                        Button::DPadLeft => 1, // LEFT
                        Button::DPadDown => 3, // DOWN
                        Button::DPadRight => 0, // RIGHT
                        Button::A => 5, // B
                        Button::B => 4, // A
                        Button::Back => 6, // SELECT
                        Button::Start => 7, // START
                        _ => -1,
                    };
                    if let Ok(key_code) = usize::try_from(key_code) {
                        game.update_input(true, key_code);
                    }
                },
                _ => (),
            }
        }

        if game.should_die {
            break 'running;
        }

        canvas.clear();
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        let _ = canvas.draw_rect(Rect::new(0, 0, WIDTH as u32, HEIGHT_PLAY_AREA_START as u32));
        for (i, cell) in game.field.cells.iter().enumerate() {
            let (x, y) = one_d_to_two_d(i);
            let texture = if cell.revealed {
                match cell.mines_around {
                    1 => {
                        &revealed_mine_1_texture
                    },
                    2 => {
                        &revealed_mine_2_texture
                    },
                    3 => {
                        &revealed_mine_3_texture
                    },
                    4 => {
                        &revealed_mine_4_texture
                    },
                    5 => {
                        &revealed_mine_5_texture
                    },
                    6 => {
                        &revealed_mine_6_texture
                    },
                    7 => {
                        &revealed_mine_7_texture
                    },
                    8 => {
                        &revealed_mine_8_texture
                    },
                    _ => {
                        &revealed_mine_texture
                    },
                }
            } else if cell.flagged {
                &flagged_mine_texture
            } else if cell.has_mine {
                &debug_mine_texture
            } else {
                &unflagged_mine_texture
            };
            canvas.copy(texture, None, Some(Rect::new(32 * (x as i32), HEIGHT_PLAY_AREA_START as i32 + (y as i32) * 32, 32, 32))).expect("Couldn't copy canvas");
        }
        canvas.set_blend_mode(BlendMode::None);
        if game.current_selection == 0 {
            canvas.copy(&cursor_texture, None, Some(Rect::new(0, HEIGHT_PLAY_AREA_START as i32, 32, 32))).expect("Couldn't copy canvas");
        } else {
            let (x, y) = one_d_to_two_d(game.current_selection);
            canvas.copy(&cursor_texture, None, Some(Rect::new(32 * (x as i32), HEIGHT_PLAY_AREA_START as i32 + (y as i32) * 32, 32, 32))).expect("Couldn't copy canvas");
        }
        canvas.present();

        //let _ = device.queue_audio(&spu.audio_data);
        //spu.audio_data.clear();

        current_instant = Instant::now();
        let elapsed = current_instant - previous_instant;
        previous_instant = current_instant;
        if elapsed <= frame_per_second {
            thread::sleep(frame_per_second - elapsed);
        }
    }
}
