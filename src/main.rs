use core::f64;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, BorderType, Paragraph, Widget},
    DefaultTerminal, Frame,
};
use std::time::Instant;
use std::{io, time::Duration};

const KB: f64 = 1.380649e-23; // Boltzmann Constant in J K^-1

#[derive(Debug, Default)]
struct App {
    lattice: Lattice,
    increment: f64,
    delay: Duration,
    exit: bool,
}

impl App {
    /// Run app until user quit
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Init lattice and values
        let init_interactivity = 10_000.0;
        let init_temperature = 10_000.0;
        self.increment = 1000.0;
        self.delay = Duration::from_millis(10);
        let mut last_tick = Instant::now();

        self.lattice = Lattice::new(25, init_interactivity, init_temperature);

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            // Start event pooling
            let timeout = self.delay.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                self.handle_events()?
            }

            // Update lattice after delay
            if last_tick.elapsed() >= self.delay {
                self.on_tick();
                last_tick = Instant::now()
            }
        }
        Ok(())
    }

    /// Draw in terminal
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// Update app state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => self.exit(),
            KeyCode::Char('+') => self.increase_increment(),
            KeyCode::Char('-') => self.decrease_increment(),
            KeyCode::Char('i') => self.increase_interactivity(),
            KeyCode::Char('t') => self.increase_temperature(),
            KeyCode::Char('d') => self.increase_delay(),
            KeyCode::Char('I') => self.decrease_interactivity(),
            KeyCode::Char('T') => self.decrease_temperature(),
            KeyCode::Char('D') => self.decrease_delay(),
            _ => {}
        }
    }

    // Render a lattice into Lines
    fn render_lattice(&self) -> Vec<Line> {
        let mut lattice_line = vec![];
        let lattice = self.lattice.clone().to_string();

        let up = " ^ ".fg(Color::Yellow).bg(Color::Red);
        let down = " v ".fg(Color::Yellow).bg(Color::White);
        for y_text in lattice {
            let mut x_row = vec![];

            for x in y_text {
                match x.as_str() {
                    "-1" => {
                        x_row.push(down.clone());
                    }
                    "1" => {
                        x_row.push(up.clone());
                    }
                    _ => {
                        continue;
                    }
                }
            }
            lattice_line.push(Line::from_iter(x_row));
        }
        lattice_line
    }

    // Run Metropolis Algorithm after delay second
    fn on_tick(&mut self) {
        let (x_rand, y_rand) = self.lattice.pick_random_point();
        self.lattice.metropolis_algo_calculation(x_rand, y_rand);
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn increase_interactivity(&mut self) {
        self.lattice.interactivity = self.lattice.interactivity + self.increment
    }

    fn increase_temperature(&mut self) {
        self.lattice.temperature = self.lattice.temperature + self.increment
    }

    fn increase_increment(&mut self) {
        self.increment = self.increment + 10.0
    }

    fn increase_delay(&mut self) {
        self.delay = self.delay + Duration::from_millis(10)
    }

    fn decrease_interactivity(&mut self) {
        self.lattice.interactivity = self.lattice.interactivity - self.increment
    }

    fn decrease_temperature(&mut self) {
        self.lattice.temperature = self.lattice.temperature - self.increment
    }

    fn decrease_increment(&mut self) {
        self.increment = self.increment - 10.0
    }

    fn decrease_delay(&mut self) {
        if self.delay != Duration::from_millis(0) {
            self.delay = self.delay - Duration::from_millis(10)
        } else {
            self.delay = Duration::from_millis(0)
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("The r-ising model".bold());
        let interactivity = self.lattice.interactivity;
        let temperature = self.lattice.temperature;
        let increment = self.increment;
        let delay = self.delay.as_millis();

        let instructions = Line::from(vec![
            " Interactivity".into(),
            format!(" = {:.2}", interactivity).yellow().bold(),
            " Temperature".into(),
            format!(" = {:.2} K", temperature).blue().bold(),
            " Variable Increment".into(),
            format!(" = {:.2}", increment).red(),
            " Delay ".into(),
            format!(" = {:.2} ms", delay).red(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title(Line::from(" Quit <q/Q> ").red().bold().left_aligned())
            .title_bottom(instructions.centered())
            .border_set(border::THICK)
            .border_type(BorderType::Rounded);

        let lattice_line = self.render_lattice();
        Paragraph::new(lattice_line)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[derive(Clone, Debug, Default)]
struct Lattice {
    // the 3d lattice
    value: Vec<Vec<i32>>,
    // lattice size
    size: usize,
    interactivity: f64,
    temperature: f64,
}

impl Lattice {
    fn new(size: usize, interactivity: f64, temperature: f64) -> Lattice {
        let mut lattice: Vec<Vec<i32>> = Vec::new();
        for _ in 0..size {
            let y_vector = (0..size)
                .map(|_| rand::random_range(0..=1))
                // Alter 0 to -1 (negative spin)
                .map(|s| if &s == &0 { -1 } else { 1 })
                .collect();
            lattice.push(y_vector)
        }
        Lattice {
            value: lattice,
            size,
            interactivity,
            temperature,
        }
    }

    // convert 1 and 0 to String
    // TODO: I don't think we need to do this. Can we cast int -> Line ?
    fn to_string(self) -> Vec<Vec<String>> {
        let mut lattice: Vec<Vec<String>> = Vec::new();

        for y in 0..self.size {
            let mut y_vector: Vec<String> = Vec::new();
            for x in 0..self.size {
                y_vector.push(self.value[y][x].to_string());
            }
            lattice.push(y_vector);
        }

        lattice
    }

    // pick randomg x and y point to be sampled
    fn pick_random_point(&self) -> (usize, usize) {
        (
            rand::random_range(0..self.size),
            rand::random_range(0..self.size),
        )
    }

    // Hamiltonian Formula
    // H = -J * sum_over_nearest_neighbors(spin_i, spin_j)
    // H = -J * current_spin * sum_of_all_neighbors
    fn calculate_hamiltonian(&self, x_rand: usize, y_rand: usize) -> f64 {
        let current_spin = f64::from(self.value[y_rand][x_rand]);
        let (left, right, down, up) = self.find_neighbours(x_rand, y_rand);

        -1.0 * self.interactivity * current_spin * f64::from(left + right + down + up)
    }

    // Gather nearest neighbours
    fn find_neighbours(&self, x_rand: usize, y_rand: usize) -> (i32, i32, i32, i32) {
        let current_spin = self.value[y_rand][x_rand];
        let is_not_most_left = x_rand != 0;
        let is_not_most_right = x_rand != self.size - 1;
        let is_not_bottom = y_rand != 0;
        let is_not_top = y_rand != self.size - 1;

        let (mut left, mut right, mut down, mut up) =
            (current_spin, current_spin, current_spin, current_spin);

        if is_not_most_left {
            left = self.value[y_rand][x_rand - 1]
        };
        if is_not_most_right {
            right = self.value[y_rand][x_rand + 1]
        };
        if is_not_bottom {
            down = self.value[y_rand - 1][x_rand]
        };
        if is_not_top {
            up = self.value[y_rand + 1][x_rand]
        };

        (left, right, down, up)
    }

    // Delta_H = H_new - H_current
    // Beta = 1 / ( k_B * T)
    // If Delta_H < 0; take the new flip. It's mean the atom transition to a lower energy state
    // If Delta_H > 0;
    // If P(Delta_H) > e^(-Beta * Delta_H); take the new flip. It's mean the atom try to escape
    // a local minima.
    // Else keep the old spin
    fn metropolis_algo_calculation(&mut self, x_rand: usize, y_rand: usize) {
        let current_hamiltonian_energy = self.calculate_hamiltonian(x_rand, y_rand);
        let flipped_hamiltonian_energy = -1.0 * current_hamiltonian_energy;

        let delta_h = flipped_hamiltonian_energy - current_hamiltonian_energy;
        let minus_beta = -1.0 / (KB * self.temperature);
        let acceptence_criteria = f64::consts::E.powf(minus_beta * delta_h);

        // Flip only when delta H is lower than 0 and acceptence_criteria is higher than half
        // Half represent the threshold to flip or not
        let is_flipped = delta_h < 0.0 || acceptence_criteria > 0.5;
        if is_flipped {
            self.value[y_rand][x_rand] = self.value[y_rand][x_rand] * -1;
        }
    }
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
