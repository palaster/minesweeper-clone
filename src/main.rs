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

const HEIGHT_OFFSET: u16 = 64;
const WIDTH: u16 = 768; // 32 * 24
const HEIGHT: u16 = HEIGHT_OFFSET + 768; // 32 * 24

const SAMPLE_RATE: u16 = 44_100;

const FLAGGED_MINE: &[u8; 211] = include_bytes!("../assets/flagged_mine.png");
const UNFLAGGED_MINE: &[u8; 128] = include_bytes!("../assets/unflagged_mine.png");
const REVEALED_MINE: &[u8; 128] = include_bytes!("../assets/revealed_mine.png");
const CURSOR: &[u8; 129] = include_bytes!("../assets/cursor.png");

#[derive(Clone, Copy)]
struct Mine {
    revealed: bool,
    flagged: bool,
    has_mine: bool,
    mines_around: u8,
}

impl Mine {
    fn new() -> Mine {
        Mine {
            revealed: false,
            flagged: false,
            has_mine: false,
            mines_around: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Field {
    mines: [Mine; 576_usize], // 24 x 24 Grid
    flags_left: u8,
}

impl Field {
    fn new() -> Field {
        Field {
            mines: [Mine::new(); 576_usize],
            flags_left: 99,
        }
    }
}

#[derive(Clone, Copy)]
struct Game {
    field: Field,
    current_selection: u16,
    inputs: [(bool, u8); 8],
}

impl Game {
    fn new() -> Game {
        Game {
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

    fn update_input(&mut self, is_down: bool, input: i8) {
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
        let input = u8::try_from(input).expect("Couldn't try_from input");
        let mut cause_event = false;
        if is_down {
            self.inputs[input as usize] = (true, 4);
        } else if self.inputs[input as usize].0 {
            cause_event = true;
            self.inputs[input as usize] = (false, 0);
        }
        if !cause_event {
            return;
        }
        match input {
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
            4 => {
                if !is_down {
                    // Reveal Mine
                    let mine = &mut self.field.mines[self.current_selection as usize];
                    if !mine.revealed {
                        if !mine.has_mine {
                            mine.revealed = true;
                            self.field.flags_left += 1;
                            mine.flagged = false;
                        } else {
                            // Game Over
                        }
                    }
                }
            }
            5 => {
                if !is_down {
                    // Flag Mine
                    let mine = &mut self.field.mines[self.current_selection as usize];
                    if !mine.revealed {
                        if mine.flagged {
                            self.field.flags_left += 1;
                            mine.flagged = false;
                        } else if self.field.flags_left > 0 {
                            self.field.flags_left -= 1;
                            mine.flagged = true;
                        }
                    }
                }
            }
            _ => {},
        }
        println!("Current Selection: {}", self.current_selection);
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

    let flagged_mine_texture = texture_creator.load_texture_bytes(FLAGGED_MINE).expect("Couldn't create texture from FLAGGED_MINE");
    let unflagged_mine_texture = texture_creator.load_texture_bytes(UNFLAGGED_MINE).expect("Couldn't create texture from UNFLAGGED_MINE");
    let revealed_mine_texture = texture_creator.load_texture_bytes(REVEALED_MINE).expect("Couldn't create texture from REVEALED_MINE");
    let cursor_texture = texture_creator.load_texture_bytes(CURSOR).expect("Couldn't create texture from CURSOR");

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
                    match mouse_btn {
                        MouseButton::Left => {
                            println!("Left Button Up - x: {} - y: {}", x, y);
                        },
                        MouseButton::Right => {
                            println!("Right Button Up - x: {} - y: {}", x, y);
                        },
                        _ => {},
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
                    if key_code >= 0 {
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
                    if key_code >= 0 {
                        game.update_input(false, key_code);
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
                    if key_code >= 0 {
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
                    if key_code >= 0 {
                        game.update_input(false, key_code);
                    }
                },
                _ => (),
            }
        }


        canvas.clear();
        canvas.set_blend_mode(BlendMode::Blend);
        canvas.set_draw_color(Color::RGB(128, 128, 128));
        let _ = canvas.draw_rect(Rect::new(0, 0, WIDTH as u32, HEIGHT_OFFSET as u32));
        for (i, mine) in game.field.mines.iter().enumerate() {
            let x: i32 = i as i32 % 24;
            let y = i as i32 / 24;
            let texture = if mine.revealed {
                &revealed_mine_texture
            } else if mine.flagged {
                &flagged_mine_texture
            } else {
                &unflagged_mine_texture
            };
            canvas.copy(texture, None, Some(Rect::new(32 * x, HEIGHT_OFFSET as i32 + y * 32, 32, 32))).expect("Couldn't copy canvas");
        }
        canvas.set_blend_mode(BlendMode::None);
        if game.current_selection == 0 {
            canvas.copy(&cursor_texture, None, Some(Rect::new(0, HEIGHT_OFFSET as i32, 32, 32))).expect("Couldn't copy canvas");
        } else {
            let x = game.current_selection as i32 % 24;
            let y = game.current_selection as i32 / 24;
            canvas.copy(&cursor_texture, None, Some(Rect::new(32 * x, HEIGHT_OFFSET as i32 + y * 32, 32, 32))).expect("Couldn't copy canvas");
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
