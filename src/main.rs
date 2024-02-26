use std::io::Cursor;
use std::time::{Instant, Duration};

use eframe::egui::{Color32, ImageSource, RichText, Sense, Vec2};
use eframe::{egui, egui::CentralPanel};
use eframe::{run_native, App, NativeOptions};
use rodio::{Decoder, OutputStream, Sink, Source};

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

const DEBUG_MINE: ImageSource<'_> = egui::include_image!("../assets/debug_mine.png");
const FLAGGED_MINE: ImageSource<'_> = egui::include_image!("../assets/flagged_mine.png");
const UNFLAGGED_MINE: ImageSource<'_> = egui::include_image!("../assets/unflagged_mine.png");
const REVEALED_MINE: ImageSource<'_> = egui::include_image!("../assets/revealed_mine.png");
const CURSOR: ImageSource<'_> = egui::include_image!("../assets/cursor.png");

const REVEALED_MINE_1: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_1.png");
const REVEALED_MINE_2: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_2.png");
const REVEALED_MINE_3: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_3.png");
const REVEALED_MINE_4: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_4.png");
const REVEALED_MINE_5: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_5.png");
const REVEALED_MINE_6: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_6.png");
const REVEALED_MINE_7: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_7.png");
const REVEALED_MINE_8: ImageSource<'_> = egui::include_image!("../assets/revealed_mine_8.png");

const CELL_IMAGES: [ImageSource<'_>; 13] = [DEBUG_MINE, FLAGGED_MINE, UNFLAGGED_MINE, REVEALED_MINE, CURSOR, REVEALED_MINE_1, REVEALED_MINE_2, REVEALED_MINE_3, REVEALED_MINE_4, REVEALED_MINE_5, REVEALED_MINE_6, REVEALED_MINE_7, REVEALED_MINE_8];

const MOULDY_CHEESE_REGULAR: &[u8; 112116] = include_bytes!("../assets/MouldyCheeseRegular-WyMWG.ttf");

const AWAKE10_MEGA_WALL: &[u8; 2026231] = include_bytes!("../assets/awake10_megaWall.mp3");

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
    was_winner: bool,
    should_die: bool,
    game_instant: Option<Instant>,
    game_duration: Option<Duration>,
    field: Field,
    current_selection: usize,
    inputs: [(bool, u8); 8],
}

impl Game {
    fn new_with_context(cc: &eframe::CreationContext<'_>) -> Game {
        let mut fonts = egui::FontDefinitions::default();

        let font_key = String::from("mouldy_cheese_regular");

        fonts.font_data.insert(font_key.clone(), egui::FontData::from_static(MOULDY_CHEESE_REGULAR));

        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, font_key.clone());

        fonts.families.entry(egui::FontFamily::Monospace).or_default().push(font_key);

        cc.egui_ctx.set_fonts(fonts);

        egui_extras::install_image_loaders(&cc.egui_ctx);

        Game::new()
    }

    fn new() -> Game {
        Game {
            was_winner: false,
            should_die: false,
            game_instant: None,
            game_duration: None,
            field: Field::new(),
            current_selection: 0,
            inputs: [(false, 0); 8],
        }
    }

    fn update(&mut self) {
        for (i, (is_down, count)) in self.inputs[0..4].iter_mut().enumerate() {
            if *is_down {
                if *count == 0 {
                    *count = 15;
                    match i {
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
                        _ => {},
                    }
                } else {
                    *count -= 1;
                }
            }
        }
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
            self.inputs[input] = (true, 15);
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
    }

    fn reveal_from_index(&mut self, index: usize) {
        if self.game_instant.is_none() {
            self.game_instant = Some(Instant::now())
        }
        let cell = &mut self.field.cells[index];
        if !cell.revealed {
            if !cell.has_mine {
                cell.revealed = true;
                if cell.flagged {
                    self.field.flags_left += 1;
                    cell.flagged = false;
                }
                if cell.mines_around == 0 {
                    self.field.reveal_surrounding_mines_from_index(index);
                }
            } else {
                self.should_die = true;
                if let Some(game_instant) = self.game_instant {
                    self.game_duration = Some(game_instant.elapsed());
                }
            }
        }
    }

    fn flag_from_index(&mut self, index: usize) {
        if self.game_instant.is_none() {
            self.game_instant = Some(Instant::now())
        }
        let cell = &mut self.field.cells[index];
        if !cell.revealed {
            if cell.flagged {
                self.field.flags_left += 1;
                cell.flagged = false;
            } else if self.field.flags_left > 0 {
                self.field.flags_left -= 1;
                cell.flagged = true;
                if self.field.flags_left == 0 {
                    for cell in self.field.cells {
                        if cell.has_mine && !cell.flagged {
                            return;
                        }
                    }
                    self.should_die = true;
                    self.was_winner = true;
                    if let Some(game_instant) = self.game_instant {
                        self.game_duration = Some(game_instant.elapsed());
                    }
                }
            }
            }
        }
}

impl App for Game {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        if !self.should_die {
            CentralPanel::default().show(ctx, |ui| {
                ui.label(format!("Time: {}", if let Some(game_instant) = self.game_instant { game_instant.elapsed().as_secs() } else { 0 }));
                ui.label(format!("Flags: {}", self.field.flags_left));
                ui.label(RichText::new("Palaster").color(Color32::RED));
                ui.style_mut().spacing.item_spacing = Vec2::ZERO;
                for i in 0..NUMBER_OF_ROWS_AND_COLUMNS {
                    ui.horizontal(|ui| {
                        for j in 0..NUMBER_OF_ROWS_AND_COLUMNS {
                            let cell_index = two_d_to_one_d(j, i);
                            let cell = self.field.cells[cell_index];
                            let image = if cell.revealed {
                                match cell.mines_around {
                                    1..=8 => {
                                        CELL_IMAGES[4 + (cell.mines_around as usize)].clone()
                                    },
                                    _ => {
                                        CELL_IMAGES[3].clone()
                                    },
                                }
                            } else if cell.flagged {
                                CELL_IMAGES[1].clone()
                            } else {
                                CELL_IMAGES[2].clone()
                            };
                            let response = ui.add(egui::widgets::ImageButton::new(image).frame(false));
                            if response.clicked() {
                                self.reveal_from_index(cell_index);
                            }
                            if response.secondary_clicked() {
                                self.flag_from_index(cell_index);
                            }
                        }
                    });
                }
                ui.reset_style();
            });
        } else {
            let inner_response = CentralPanel::default().show(ctx, |ui| {
                ui.label(if self.was_winner { "Winner" } else { "Loser" });
                ui.label(format!("Time: {}", if let Some(game_duration) = self.game_duration { game_duration.as_secs() } else { 0 }));
                if !self.was_winner {
                    let mut flagged_mine_counter = 0;
                    for cell in self.field.cells {
                        if cell.flagged && cell.has_mine {
                            flagged_mine_counter += 1;
                        }
                    }
                    ui.label(format!("Correctly flagged mines: {}", flagged_mine_counter));
                }
                ui.label(RichText::new("Palaster").color(Color32::RED));
            });

            let response = inner_response.response.interact(Sense::click());
            if response.clicked() {
                self.was_winner = false;
                self.should_die = false;
                self.game_instant = None;
                self.game_duration = None;
                self.field = Field::new();
                self.current_selection = 0;
                self.inputs = [(false, 0); 8];
            }
        }
    }
}

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().expect("Couldn't get default output stream");
    let sink = Sink::try_new(&stream_handle).expect("Couldn't create new sink from stream_handle");
    let decoder = Decoder::new(Cursor::new(AWAKE10_MEGA_WALL)).expect("Couldn't create decoder");
    sink.set_volume(1.0);
    sink.append(decoder.repeat_infinite());
    sink.play();

    if run_native("Minesweeper", NativeOptions::default(), Box::new(|cc| Box::new(Game::new_with_context(cc)))).is_err() {}
    /*
    'running: loop {
        game.update();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => {
                    break 'running
                },
                Event::MouseButtonUp { mouse_btn, x, y, .. } => {
                    if game.scene == 1 {
                        game.scene = 0;
                        game.game_instant = None;
                        game.game_duration = None;
                        game.was_winner = false;
                        game.field = Field::new();
                        game.current_selection = 0;
                        game.inputs = [(false, 0); 8];
                        continue;
                    }
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
                    if game.scene == 1 {
                        game.scene = 0;
                        game.game_instant = None;
                        game.game_duration = None;
                        game.was_winner = false;
                        game.field = Field::new();
                        game.current_selection = 0;
                        game.inputs = [(false, 0); 8];
                        continue;
                    }
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
                    if let Ok(key_code) = usize::try_from(key_code) {
                        game.update_input(true, key_code);
                    }
                },
                Event::ControllerButtonUp { button, .. } => {
                    if game.scene == 1 {
                        game.scene = 0;
                        game.game_instant = None;
                        game.game_duration = None;
                        game.was_winner = false;
                        game.field = Field::new();
                        game.current_selection = 0;
                        game.inputs = [(false, 0); 8];
                        continue;
                    }
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
                        game.update_input(false, key_code);
                    }
                },
                _ => (),
            }
        }
    }
     */
}
