use eframe::egui::{self, Pos2, Rect};
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
        // Set to always use dark mode
        cc.egui_ctx.set_theme(egui::Theme::Dark);

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
        let side_panel_width = 150.0;
        let top_bottom_panel_height = 50.0;

        egui::TopBottomPanel::top("top_panel")
            .resizable(true)
            .default_height(top_bottom_panel_height)
            .height_range(25.0..=75.0)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("R-Ising Model");
                    ui.label("Ising Model simulation built with Rust and egui.");
                });
            });

        egui::SidePanel::left("left_panel")
            .default_width(side_panel_width)
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
                ui.label("");

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

                ui.vertical(|ui| {
                    ui.label("");
                    ui.label("Legends:");
                    ui.label(egui::RichText::new("Spin up (+)").color(egui::Color32::DARK_RED));
                    ui.label(egui::RichText::new("Spin down (-)").color(egui::Color32::LIGHT_BLUE));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::containers::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.label("Hover on a tile to see the detail");

                // To create a 2D grid we need these data
                // - Display size
                // - Number of spins in a row
                let ui_size = ui.available_size();
                let (tile_size, offset) = if ui_size.x > ui_size.y {
                    (
                        ui_size.y / (1.5 * self.lattice.size as f32),
                        (ui_size.x - ui_size.y) / 2.,
                    )
                } else {
                    (
                        ui_size.x / (1.5 * self.lattice.size as f32),
                        (ui_size.y - ui_size.x) / 2.,
                    )
                };

                // Render lattice
                for x in 0..self.lattice.size {
                    for y in 0..self.lattice.size {
                        let (xp, yp) = if ui_size.x > ui_size.y {
                            (
                                x as f32 * tile_size + offset + 1.5 * side_panel_width,
                                y as f32 * tile_size + top_bottom_panel_height,
                            )
                        } else {
                            (
                                x as f32 * tile_size + 1.5 * side_panel_width,
                                y as f32 * tile_size + offset + top_bottom_panel_height,
                            )
                        };
                        let tile = Rect::from_two_pos(
                            Pos2::new(xp, yp),
                            Pos2::new(xp + tile_size, yp + tile_size),
                        );
                        if ui.rect_contains_pointer(tile) {
                            let h_energy = self.lattice.calculate_hamiltonian(x, y);
                            let delta_h = self.lattice.calculate_delta_h(x, y);
                            let acceptence_criteria =
                                self.lattice.calculate_acceptence_criteria(delta_h);
                            let is_flipped = delta_h < 0.0 || acceptence_criteria > 0.5;

                            if self.lattice.value[y].value[x] == 1 {
                                ui.label(
                                    egui::RichText::new(format!("x: {x}, y: {y} Spin up (+)"))
                                        .color(egui::Color32::DARK_RED),
                                );
                            } else {
                                ui.label(
                                    egui::RichText::new(format!("x: {x}, y: {y} Spin down (-)"))
                                        .color(egui::Color32::LIGHT_BLUE),
                                );
                            }
                            ui.label(format!("Hamiltonian Energy: {h_energy} | Diff: {delta_h}"));
                            ui.label(format!("Acceptance Criteria: {acceptence_criteria} | Will be flipped? {is_flipped}"));
                        }
                        let fil_color = if self.lattice.value[y].value[x] == 1 {
                            egui::Color32::DARK_RED
                        } else {
                            egui::Color32::LIGHT_BLUE
                        };
                        ui.painter().rect_filled(tile, 0.0, fil_color);
                    }
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
            .default_height(top_bottom_panel_height)
            .height_range(25.0..=75.0)
            .show(ctx, |ui| {
                ui.label("Made by Husni smoll brain");
            });
    }
}
