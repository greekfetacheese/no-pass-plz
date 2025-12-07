use super::SHARED_GUI;
use eframe::{
   CreationContext,
   egui::{self, Frame},
};
use passwd_derive::PasswordDeriver;
use secure_types::SecureString;
use serde::{Deserialize, Serialize};
use std::{
   collections::HashMap,
   sync::{Arc, RwLock},
};
use zeus_theme::{Theme, ThemeKind};

#[derive(Clone, Default)]
pub struct AppCtx(Arc<RwLock<AppData>>);

impl AppCtx {
   pub fn read<R>(&self, reader: impl FnOnce(&AppData) -> R) -> R {
      reader(&self.0.read().unwrap())
   }

   pub fn write<R>(&self, writer: impl FnOnce(&mut AppData) -> R) -> R {
      writer(&mut self.0.write().unwrap())
   }

   pub fn load_index_map_from_file(&self) -> Result<(), Box<dyn std::error::Error>> {
      self.write(|app| app.load_index_map_from_file())
   }

   pub fn save_index_map_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
      self.read(|app| app.save_index_map_to_file())
   }

   pub fn get_index(&self, index: u32) -> Option<IndexData> {
      self.read(|app| app.index_map.get(&index).cloned())
   }

   pub fn set_index(&self, index: u32, data: IndexData) {
      self.write(|app| {
         app.index_map.insert(index, data);
      });
   }

   pub fn remove_index(&self, index: u32) {
      self.write(|app| {
         app.index_map.remove(&index);
      });
   }

   pub fn derive_at(&self, index: u32) -> Result<SecureString, Box<dyn std::error::Error>> {
      self.read(|app| {
         if let Some(deriver) = &app.passwd_derive {
            Ok(deriver.derive_at(index))
         } else {
            Err("No deriver instance found".into())
         }
      })
   }
}

#[derive(Default, Serialize, Deserialize)]
pub struct AppData {
   #[serde(skip)]
   pub passwd_derive: Option<PasswordDeriver>,
   pub index_map: HashMap<u32, IndexData>,
}

impl AppData {
   pub fn load_index_map_from_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
      let dir = std::env::current_dir()?;
      let path = dir.join("NoPassPlz.json");
      let data = std::fs::read(&path)?;
      let temp: AppData = serde_json::from_slice(&data)?;
      self.index_map = temp.index_map;
      Ok(())
   }

   pub fn save_index_map_to_file(&self) -> Result<(), Box<dyn std::error::Error>> {
      let dir = std::env::current_dir()?;
      let path = dir.join("NoPassPlz.json");
      let data = serde_json::to_string(self)?;
      std::fs::write(path, data)?;
      Ok(())
   }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct IndexData {
   pub exposed: bool,
   pub title: String,
   pub description: String,
}

pub struct App {
   pub style_has_been_set: bool,
   pub app_ctx: AppCtx,
}

impl App {
   pub fn new(cc: &CreationContext) -> Self {
      let egui_ctx = cc.egui_ctx.clone();
      let theme = Theme::new(ThemeKind::Dark);
      egui_ctx.set_style(theme.style.clone());

      SHARED_GUI.write(|gui| {
         gui.egui_ctx = egui_ctx;
      });

      let app_ctx = AppCtx::default();

      match app_ctx.load_index_map_from_file() {
         Ok(_) => {}
         Err(e) => {
            eprintln!("Failed to load app data {}", e);
         }
      };

      Self {
         style_has_been_set: false,
         app_ctx,
      }
   }

   fn on_shutdown(&mut self, ctx: &egui::Context) {
      if ctx.input(|i| i.viewport().close_requested()) {
         self.app_ctx.write(|app| {
            if let Some(mut deriver) = app.passwd_derive.take() {
               deriver.erase();
            }
         });
      }
   }
}

impl eframe::App for App {
   fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
      egui::Rgba::TRANSPARENT.to_array()
   }

   fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
      SHARED_GUI.write(|gui| {
         self.on_shutdown(ctx);

         // This is needed for Windows
         if !self.style_has_been_set {
            let style = gui.theme.style.clone();
            ctx.set_style(style);
            self.style_has_been_set = true;
         }

         let theme = gui.theme.clone();
         let bg_color = theme.colors.bg;
         let panel_frame = Frame::new().fill(bg_color);
         let top_frame = Frame::new().inner_margin(5).fill(bg_color);

         egui::TopBottomPanel::top("top_panel")
            .min_height(30.0)
            .max_height(50.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(top_frame)
            .show(ctx, |ui| {
               gui.show_top_panel(ui);
            });

         egui::CentralPanel::default().frame(panel_frame).show(ctx, |ui| {
            ui.vertical_centered(|ui| {
               gui.show_central_panel(self.app_ctx.clone(), ui);
            });
         });
      });
   }
}
