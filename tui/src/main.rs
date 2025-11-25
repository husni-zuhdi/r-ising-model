use core::f64;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use internal::Lattice;
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
        let lattice = self.lattice.clone().convert_to_string();

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
        self.lattice.interactivity += self.increment
    }

    fn increase_temperature(&mut self) {
        self.lattice.temperature += self.increment
    }

    fn increase_increment(&mut self) {
        self.increment += 10.0
    }

    fn increase_delay(&mut self) {
        self.delay += Duration::from_millis(10)
    }

    fn decrease_interactivity(&mut self) {
        self.lattice.interactivity -= self.increment
    }

    fn decrease_temperature(&mut self) {
        if self.lattice.temperature == 0.0 {
            self.lattice.temperature = 0.0;
            return;
        }
        self.lattice.temperature -= self.increment
    }

    fn decrease_increment(&mut self) {
        self.increment -= 10.0
    }

    fn decrease_delay(&mut self) {
        if self.delay == Duration::from_millis(0) {
            self.delay = Duration::from_millis(0);
            return;
        }
        self.delay -= Duration::from_millis(10)
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
            format!(" = {interactivity:.2}").yellow().bold(),
            " Temperature".into(),
            format!(" = {temperature:.2} K").blue().bold(),
            " Variable Increment".into(),
            format!(" = {increment:.2}").red(),
        ]);

        let block = Block::bordered()
            .title(title.centered())
            .title(Line::from(" Quit <q/Q> ").red().bold().left_aligned())
            .title(Line::from(" Delay ").gray().right_aligned())
            .title(Line::from(format!(" {delay:.2}ms ")).red().right_aligned())
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

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}
