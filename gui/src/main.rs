use eframe::egui;
use internal::Lattice;
use std::time::Duration;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([640.0, 640.0])
            .with_resizable(true),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "R-Ising Model",
        options,
        Box::new(|_| Ok(Box::<App>::default())),
    )
}

struct App {
    lattice: Lattice,
    delay: usize,
    is_paused: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            lattice: Lattice::new(25, 1000.0, 1000.0),
            delay: 1000,
            is_paused: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("R-Ising Model");

            egui::containers::Frame::canvas(ui.style()).show(ui, |ui| {
                let up = egui::RichText::new(" ^ ").color(egui::Color32::RED);
                let down = egui::RichText::new(" v ").color(egui::Color32::YELLOW);

                ui.ctx()
                    .request_repaint_after(Duration::from_millis(self.delay as u64));
                for y_text in &self.lattice.value {
                    ui.horizontal(|ui| {
                        for x in y_text {
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
                let (x_rand, y_rand) = self.lattice.pick_random_point();
                self.lattice.metropolis_algo_calculation(x_rand, y_rand);
            });

            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.lattice.size += 1;
                }
                if ui.button("-").clicked() {
                    self.lattice.size -= 1;
                }
                ui.add(egui::Slider::new(&mut self.lattice.size, 0..=1000).text("size"));
            });
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.lattice.temperature += 1.0;
                }
                if ui.button("-").clicked() {
                    self.lattice.temperature -= 1.0;
                }
                ui.add(
                    egui::Slider::new(&mut self.lattice.temperature, 0.0..=10_000.0)
                        .text("temperature (K)"),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.lattice.interactivity += 1.0;
                }
                if ui.button("-").clicked() {
                    self.lattice.interactivity -= 1.0;
                }
                ui.add(
                    egui::Slider::new(&mut self.lattice.interactivity, -10_000.0..=10_000.0)
                        .text("interactivity"),
                );
            });
            ui.horizontal(|ui| {
                if ui.button("+").clicked() {
                    self.delay += 1;
                }
                if ui.button("-").clicked() {
                    self.delay -= 1;
                }
                ui.add(egui::Slider::new(&mut self.delay, 0..=10_000).text("delay (ms)"));
            });
            if ui.button("Pause").clicked() {
                self.is_paused = true;
            }
        });
    }
}
