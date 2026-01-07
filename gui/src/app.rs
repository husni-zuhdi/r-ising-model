use eframe::egui;
use internal::Lattice;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct App {
    pub lattice: Lattice,
    pub is_paused: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            lattice: Lattice::new(15, 100.0, 100.0),
            is_paused: true,
        }
    }
}

impl App {
    /// Called new before egui render the frist frame
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for App {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .default_height(50.0)
            .height_range(25.0..=75.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("R-Ising Model");
                    ui.label("Ising Model simulation built with Rust and egui.");
                });
            });

        egui::SidePanel::left("left_panel")
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if self.is_paused {
                        if ui.button("Resume").clicked() {
                            println!("Resumed");
                            self.is_paused = false;
                        }
                    } else if ui.button("Pause").clicked() {
                        println!("Paused");
                        self.is_paused = true;
                    }

                    if ui.button("Reset").clicked() {
                        println!("Reset");
                        self.lattice = self.lattice.reset_value();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Lattice Size");
                    let response =
                        ui.add(egui::DragValue::new(&mut self.lattice.size).range(5.0..=25.0));
                    if response.changed() {
                        println!("Updating Lattice size to {}", self.lattice.size);
                        self.lattice.update_lattice();
                    }
                });

                ui.vertical(|ui| {
                    ui.label("Temperature (K)");
                    let response = ui.add(egui::Slider::new(
                        &mut self.lattice.temperature,
                        0.0..=10_000.0,
                    ));
                    if response.changed() {
                        println!("Updating temperature (K) to {}", self.lattice.temperature);
                    }
                });

                ui.vertical(|ui| {
                    ui.label("Interactivity");
                    let response = ui.add(egui::Slider::new(
                        &mut self.lattice.interactivity,
                        -10_000.0..=10_000.0,
                    ));
                    if response.changed() {
                        println!(
                            "Updating interactivity (K) to {}",
                            self.lattice.interactivity
                        );
                    }
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::containers::Frame::canvas(ui.style()).show(ui, |ui| {
                let up = egui::RichText::new(" ^ ").color(egui::Color32::RED);
                let down = egui::RichText::new(" v ").color(egui::Color32::YELLOW);

                // Render lattice
                for y_text in &self.lattice.value {
                    ui.horizontal(|ui| {
                        for x in &y_text.value {
                            match x {
                                -1 => {
                                    ui.label(down.clone());
                                }
                                1 => {
                                    ui.label(up.clone());
                                }
                                _ => continue,
                            }
                        }
                    });
                }

                // Only re-calculate and repaint if resumed
                if !self.is_paused {
                    let (x_rand, y_rand) = self.lattice.pick_random_point();
                    self.lattice.metropolis_algo_calculation(x_rand, y_rand);

                    ui.ctx().request_repaint();
                }
            });
        });

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(50.0)
            .height_range(25.0..=75.0)
            .show(ctx, |ui| {
                ui.label("Made by Husni smoll brain");
            });
    }
}
