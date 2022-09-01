// 유전자 알고리즘 -- 재산 분배 문제
// 2022-09-01
// 
// YeJun Jung (yejun614@naver.com)

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::thread;
use std::sync::mpsc;

use eframe::egui;
use egui::plot::{Line, Plot, PlotPoints};
use egui::{FontId, TextStyle};
use egui::FontFamily::Proportional;

use super::load_dataset;
use super::model::{GAModelTracker, GAModel, Gene};

pub struct GeneApp {
    pub model: GAModel,
    pub recv: mpsc::Receiver<GAModelTracker>,
    pub generation: usize,
    pub divide_file_path: String,
    pub properties_file_path: String,
    pub control_window: bool,
    pub logs_window: bool,
    pub plot_window: bool,
    pub fit_results_window: bool,
}

impl Default for GeneApp {
    fn default() -> Self {
        Self {
            model: GAModel::default(),
            recv: mpsc::channel().1,
            generation: 5000,
            divide_file_path: "./property/divide10.txt".to_string(),
            properties_file_path: "./property/properties100.txt".to_string(),
            control_window: true,
            logs_window: false,
            plot_window: false,
            fit_results_window: false,
          }
    }
}

impl eframe::App for GeneApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
          (TextStyle::Heading, FontId::new(30.0, Proportional)),
          (TextStyle::Body, FontId::new(18.0, Proportional)),
          (TextStyle::Monospace, FontId::new(14.0, Proportional)),
          (TextStyle::Button, FontId::new(14.0, Proportional)),
          (TextStyle::Small, FontId::new(10.0, Proportional)),
        ].into();

        ctx.set_style(style);

        egui::Window::new("Control")
          .default_size(egui::Vec2::new(1000.0, 300.0))
          .open(&mut self.control_window)
          .show(ctx, |ui| {
              // Title
              ui.heading("Genegtic Algorithm Example");
              ui.label("Property distribution problem");
              ui.add_space(15.0);

              ui.collapsing("Dataset", |ui| {
                  // Dataset
                  ui.label("Divide File Path");
                  ui.text_edit_singleline(&mut self.divide_file_path);
                  ui.add_space(10.0);

                  ui.label("Properties File Path");
                  ui.text_edit_singleline(&mut self.properties_file_path);
                  ui.add_space(10.0);

                  if ui.button("Load datasets").clicked() {
                    let (divide, properties) = load_dataset(&self.divide_file_path, &self.properties_file_path);

                    self.model.divide = divide;
                    self.model.properties = properties;
                  }
              });

              ui.collapsing("Parameters", |ui| {
                  egui::Grid::new("parameters_grid").show(ui, |ui| {
                      ui.label("Generation");
                      ui.add(egui::Slider::new(&mut self.generation, 0..=100000));
                      ui.add_space(10.0);
                      ui.end_row();

                      ui.label("Gene Len");
                      ui.add(egui::Slider::new(&mut self.model.gene_len, 0..=100000));
                      ui.add_space(10.0);
                      ui.end_row();

                      ui.label("Mutation Probability");
                      ui.add(egui::Slider::new(&mut self.model.mutation_probability, 0.0..=1.0));
                      ui.add_space(10.0);
                      ui.end_row();

                      ui.label("Mutation Gene Data Len");
                      ui.add(egui::Slider::new(&mut self.model.mutation_gene_data_len, 0..=10000));
                      ui.add_space(10.0);
                      ui.end_row();

                      ui.label("Elite Conservation Probability");
                      ui.add(egui::Slider::new(&mut self.model.elite_conservation_probability, 0.0..=1.0));
                      ui.add_space(10.0);
                      ui.end_row();

                      ui.label("Total Conservation Probability");
                      ui.add(egui::Slider::new(&mut self.model.total_conservation_probability, 0.0..=1.0));
                      ui.add_space(10.0);
                      ui.end_row();
                  });
              });

              if ui.button("Fit start").clicked() {
                  self.plot_window = true;
                  self.fit_results_window = true;

                  let mut model = self.model.clone();
                  let generation = self.generation;

                  let (handler, rx) = model.fit_back(generation);
                  self.recv = rx;
              }
        });

        egui::Window::new("Logs")
          .default_size(egui::Vec2::new(1000.0, 500.0))
          .open(&mut self.logs_window)
          .show(ctx, |ui| {
              egui::ScrollArea::vertical().show(ui, |ui| {
                  ui.label("hello");
              });
          });

        egui::Window::new("Plot")
          .default_size(egui::Vec2::new(1000.0, 300.0))
          .open(&mut self.plot_window)
          .show(ctx, |ui| {
              match self.recv.try_recv() {
                Ok(tracker) => { self.model.tracker = tracker; }
                Err(err) => {}
              }

              let best_fitness_changes: PlotPoints = (0..self.model.tracker.total_generation).map(|i| {
                  let x = i as f64;
                  [x, self.model.tracker.best_fitness_changes[i as usize]]
              }).collect();

              let average_fitness_changes: PlotPoints = (0..self.model.tracker.total_generation).map(|i| {
                  let x = i as f64;
                  [x, self.model.tracker.average_fitness_changes[i as usize]]
              }).collect();

              let average_diff_changes: PlotPoints = (0..self.model.tracker.total_generation).map(|i| {
                  let x = i as f64;
                  [x, self.model.tracker.average_diff_changes[i as usize]]
              }).collect();

              Plot::new("fitness changes")
                  .view_aspect(2.0)
                  .show(ui, |plot_ui| {
                      plot_ui.line(Line::new(best_fitness_changes));
                      plot_ui.line(Line::new(average_fitness_changes));
                      plot_ui.line(Line::new(average_diff_changes));
                  });
          });

        // egui::Window::new("Fit Result")
        //   .default_size(egui::Vec2::new(1000.0, 300.0))
        //   .open(&mut self.fit_results_window)
        //   .show(ctx, |ui| {
        //       ui.heading("Best Gene");

        //       ui.label("Gene Data: ");
        //       ui.label(format!("{:?}", self.model.tracker.best_gene.data));

        //       ui.label("Fitness: ");
        //       ui.label(format!("{}", self.model.tracker.best_gene.fitness));
        //   });

        ctx.request_repaint();
    }
}

impl GeneApp {
    pub fn run_native(self) {
        let options = eframe::NativeOptions::default();

        eframe::run_native(
            "Genetic Algorithm Example",
            options,
            Box::new(|_cc| Box::new(self)),
        );
    }
}
