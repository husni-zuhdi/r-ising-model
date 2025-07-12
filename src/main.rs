use core::{f64, time};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{
        canvas::{Canvas, Points},
        Block, Paragraph, Widget,
    },
    DefaultTerminal, Frame,
};
use std::io;
use std::thread::sleep;

const UP: &str = "+";
const DOWN: &str = "-";
const J: f64 = 10.0; // Interactivity. J > 0 ferromagnetic. J = 0 non-interactive. J < 0
                     // anti-ferromagnetic
const TEMP: f64 = 10000.0; // In K
const KB: f64 = 1.380649e-23; // Boltzmann Constant in J K^-1

#[derive(Debug, Default)]
struct App {
    lattice: Lattice,
    exit: bool,
}

impl App {
    /// Run app until user quit
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Init lattice and run the app loop
        self.lattice = Lattice::new(10, J, TEMP);
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
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
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("The r-ising model".bold());
        let instructions = Line::from(vec![" Quit ".into(), "<Q> ".red().bold()]);

        let block = Block::bordered()
            .title(title.centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);

        // let body = Text::from(vec![
        //     Line::from(vec!["Test: ".into(), " True ".blue().bold().into()]),
        //     Line::from(vec!["Tast: ".into(), " True ".red().bold().into()]),
        // ]);

        let lattice = self.lattice.clone().to_string();
        let y_text: String = lattice[0].iter().flat_map(|x| x.chars()).collect();
        let reduced_lattice: Text = Text::from(y_text.as_str());

        Paragraph::new(reduced_lattice)
            .centered()
            .block(block)
            .render(area, buf);
    }
}

#[derive(Clone, Debug, Default)]
struct Lattice {
    value: Vec<Vec<i32>>,
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

    fn show(&self) {
        for y_index in 0..self.size {
            for x_index in 0..self.size {
                let spin = if self.value[y_index][x_index] == 1 {
                    UP
                } else {
                    DOWN
                };
                if x_index == self.size - 1 {
                    // Add a new line
                    println!(" {} ", spin)
                } else {
                    // Only print spin
                    print!(" {} ", spin)
                }
            }
        }
    }

    fn pick_random_point(&self) -> (usize, usize) {
        (
            rand::random_range(0..self.size),
            rand::random_range(0..self.size),
        )
    }

    fn calculate_hamiltonian(&self, x_rand: usize, y_rand: usize) -> f64 {
        let random_spin = f64::from(self.value[y_rand][x_rand]);
        println!("Random Point at {}, {}: {}", x_rand, y_rand, random_spin);

        let left = if x_rand != 0 {
            self.value[y_rand][x_rand - 1]
        } else {
            self.value[y_rand][x_rand]
        };
        let right = if x_rand != self.size - 1 {
            self.value[y_rand][x_rand + 1]
        } else {
            self.value[y_rand][x_rand]
        };
        let down = if y_rand != 0 {
            self.value[y_rand - 1][x_rand]
        } else {
            self.value[y_rand][x_rand]
        };
        let up = if y_rand != self.size - 1 {
            self.value[y_rand + 1][x_rand]
        } else {
            self.value[y_rand][x_rand]
        };
        println!(
            "Left: {}, Right: {}, Down: {}, Up: {}.",
            left, right, up, down
        );
        // Hamiltonian Formula
        // H = -J * sum_over_nearest_neighbors(spin_i, spin_j)
        // H = -J * current_spin * sum_of_all_neighbors
        -1.0 * self.interactivity * random_spin * f64::from(left + right + down + up)
    }

    fn metropolis_algo_calculation(&mut self, x_rand: usize, y_rand: usize) {
        // Delta_H = H_new - H_current
        // Beta = 1 / ( k_B * T)
        // If Delta_H < 0; take the new flip. It's mean the atom transition to a lower energy state
        // If Delta_H > 0;
        // If P(Delta_H) > e^(-Beta * Delta_H); take the new flip. It's mean the atom try to escape
        // a local minima.
        // Else keep the old spin
        let current_hamiltonian_energy = self.calculate_hamiltonian(x_rand, y_rand);
        let new_hamiltonian_energy = -1.0 * current_hamiltonian_energy;
        println!(
            "New and Current H: {}, {}",
            new_hamiltonian_energy, current_hamiltonian_energy
        );

        let delta_h = new_hamiltonian_energy - current_hamiltonian_energy;
        let acceptence_criteria = f64::consts::E.powf(delta_h * (-1.0 / KB * self.temperature));
        println!("Delta H: {}", delta_h);
        println!("A: {}", acceptence_criteria);

        // Flip only in these two condition
        if delta_h < 0.0 || acceptence_criteria > 0.5 {
            self.value[y_rand][x_rand] = self.value[y_rand][x_rand] * -1;
            println!("FLIPPED!")
        }
    }
}

// fn main() {
//     let mut lattice = Lattice::new(5, J, TEMP);
//
//     for _ in 0..50 {
//         lattice.show();
//         let (x_rand, y_rand) = lattice.pick_random_point();
//         lattice.metropolis_algo_calculation(x_rand, y_rand);
//         sleep(time::Duration::from_secs(1))
//     }
// }

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
