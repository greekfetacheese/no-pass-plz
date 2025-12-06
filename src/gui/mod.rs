pub mod app;
pub mod auth;
pub mod home;
pub mod misc;

use app::AppCtx;

use eframe::egui::{
   Align2, Button, Context, MenuBar, OpenUrl, RichText, ScrollArea, Ui, Window, vec2,
};
use egui_commonmark::{CommonMarkCache, CommonMarkViewer};
use lazy_static::lazy_static;
use std::sync::{Arc, RwLock};
use zeus_theme::{Theme, ThemeKind};

use super::gui::{auth::*, home::Home, misc::*};

lazy_static! {
   pub static ref SHARED_GUI: SharedGUI = SharedGUI::default();
}

#[derive(Clone)]
pub struct SharedGUI(Arc<RwLock<GUI>>);

impl SharedGUI {
   /// Shared access to the [GUI]
   pub fn _read<R>(&self, reader: impl FnOnce(&GUI) -> R) -> R {
      reader(&self.0.read().unwrap())
   }

   /// Exclusive mutable access to the [GUI]
   pub fn write<R>(&self, writer: impl FnOnce(&mut GUI) -> R) -> R {
      writer(&mut self.0.write().unwrap())
   }
}

impl Default for SharedGUI {
   fn default() -> Self {
      Self(Arc::new(RwLock::new(GUI::default())))
   }
}

pub struct GUI {
   pub egui_ctx: Context,
   pub theme: Theme,
   pub top_menu: TopMenu,
   pub home: Home,
   pub auth: Auth,
   pub msg_window: MsgWindow,
   pub loading_window: LoadingWindow,
}

impl Default for GUI {
   fn default() -> Self {
      Self {
         egui_ctx: Context::default(),
         theme: Theme::new(ThemeKind::Dark),
         top_menu: TopMenu::new(),
         home: Home::new(),
         auth: Auth::new(),
         msg_window: MsgWindow::new(),
         loading_window: LoadingWindow::default(),
      }
   }
}

impl GUI {
   pub fn request_repaint(&self) {
      self.egui_ctx.request_repaint();
   }

   pub fn show_central_panel(&mut self, app: AppCtx, ui: &mut Ui) {
      let theme = &self.theme;

      self.msg_window.show(theme, ui);
      self.loading_window.show(theme, ui);
      self.top_menu.show_how_it_works(theme, ui);
      self.top_menu.show_about(theme, ui);

      self.auth.show(app.clone(), theme, ui);
      self.home.show(app, theme, ui);
   }

   pub fn show_top_panel(&mut self, ui: &mut Ui) {
      let theme = &self.theme;
      self.top_menu.show(theme, ui);
   }
}

pub struct TopMenu {
   how_it_works_open: bool,
   about_open: bool,
}

impl TopMenu {
   pub fn new() -> Self {
      Self {
         how_it_works_open: false,
         about_open: false,
      }
   }

   pub fn open_about(&mut self) {
      self.about_open = true;
   }

   pub fn show(&mut self, theme: &Theme, ui: &mut Ui) {
      MenuBar::new().ui(ui, |ui| {
         ui.spacing_mut().button_padding = vec2(8.0, 8.0);

         let text = RichText::new("Help").size(theme.text_sizes.normal);

         ui.menu_button(text, |ui| {
            ui.spacing_mut().button_padding = vec2(4.0, 4.0);

            let text = RichText::new("How it works").size(theme.text_sizes.normal);
            if ui.button(text).clicked() {
               self.how_it_works_open = true;
            }

            let text = RichText::new("About").size(theme.text_sizes.normal);
            if ui.button(text).clicked() {
               self.open_about();
            }
         });
      });
   }

   pub fn show_about(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.about_open {
         return;
      }

      Window::new("About")
         .title_bar(false)
         .resizable(false)
         .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
         .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
               ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
               ui.spacing_mut().button_padding = vec2(8.0, 8.0);

               let text = RichText::new("NoPassPlz").size(theme.text_sizes.heading);
               ui.label(text);

               let text = RichText::new("Version 1.0.0").size(theme.text_sizes.normal);
               ui.label(text);

               let repo_link = "https://github.com/greekfetacheese/no-pass-plz";
               let text = RichText::new("View on GitHub").size(theme.text_sizes.normal);
               let res = ui.add(Button::new(text).min_size(vec2(100.0, 25.0)));

               if res.clicked() {
                  let url = OpenUrl::new_tab(repo_link);
                  ui.ctx().open_url(url);
               }

               let text = RichText::new("Close").size(theme.text_sizes.normal);
               let button = Button::new(text).min_size(vec2(100.0, 25.0));
               if ui.add(button).clicked() {
                  self.about_open = false;
               }
            });
         });
   }

   pub fn show_how_it_works(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.how_it_works_open {
         return;
      }

      Window::new("How it works")
         .title_bar(false)
         .resizable(false)
         .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
         .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
               ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
               ui.spacing_mut().button_padding = vec2(8.0, 8.0);

               ui.set_width(400.0);
               ui.set_height(450.0);

               let readme = include_str!("../../readme.md");

               ScrollArea::vertical().show(ui, |ui| {
                  let mut cache = CommonMarkCache::default();
                  CommonMarkViewer::new().show(ui, &mut cache, readme);
               });

               let text = RichText::new("Close").size(theme.text_sizes.normal);
               let button = Button::new(text).min_size(vec2(100.0, 25.0));
               if ui.add(button).clicked() {
                  self.how_it_works_open = false;
               }
            });
         });
   }
}
