use eframe::egui::{Label, Order, RichText, Spinner, Ui, vec2};

use zeus_theme::Theme;
use zeus_widgets::{Button, Modal};

pub struct LoadingWindow {
   open: bool,
   pub msg: String,
   pub size: (f32, f32),
}

impl Default for LoadingWindow {
   fn default() -> Self {
      Self::new()
   }
}

impl LoadingWindow {
   pub fn new() -> Self {
      Self {
         open: false,
         msg: String::new(),
         size: (250.0, 150.0),
      }
   }

   pub fn open(&mut self, msg: impl Into<String>) {
      self.open = true;
      self.msg = msg.into();
   }

   pub fn reset(&mut self) {
      *self = Self::new();
   }

   pub fn show(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.open {
         return;
      }

      Modal::new(self.msg.clone(), &mut self.open)
         .backdrop_order(Order::Tooltip)
         .content_order(Order::Debug)
         .close_on_backdrop(false)
         .close_on_escape(false)
         .show(ui.ctx(), |ui| {
            ui.set_width(self.size.0);
            ui.set_max_height(self.size.1);

            ui.vertical_centered(|ui| {
               ui.add(Spinner::new().size(25.0).color(theme.colors.text));
               ui.label(RichText::new(&self.msg).size(17.0));
            });
         });
   }
}

#[derive(Default)]
pub struct MsgWindow {
   pub open: bool,
   pub title: String,
   pub message: String,
   pub size: (f32, f32),
}

impl MsgWindow {
   pub fn new() -> Self {
      Self {
         open: false,
         title: String::new(),
         message: String::new(),
         size: (250.0, 150.0),
      }
   }

   /// Open the window with this title and message
   pub fn open(&mut self, title: impl Into<String>, msg: impl Into<String>) {
      self.open = true;
      self.title = title.into();
      self.message = msg.into();
   }

   pub fn show(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.open {
         return;
      }

      let button_visuals = theme.button_visuals();

      let title = RichText::new(self.title.clone()).size(theme.text_sizes.heading);
      let msg = RichText::new(&self.message).size(theme.text_sizes.normal);

      let mut open = self.open;

      Modal::new("msg_window", &mut open)
         .backdrop_order(Order::Tooltip)
         .content_order(Order::Debug)
         .close_on_backdrop(false)
         .close_on_escape(false)
         .show(ui.ctx(), |ui| {
            ui.set_width(self.size.0);
            ui.set_max_height(self.size.1);

            ui.vertical_centered(|ui| {
               ui.spacing_mut().item_spacing.y = 20.0;
               ui.spacing_mut().button_padding = vec2(10.0, 8.0);

               ui.label(title);

               let label = Label::new(msg).wrap();
               ui.add(label);

               let size = vec2(50.0, 20.0);
               let text = RichText::new("OK").size(theme.text_sizes.normal);
               let ok_button = Button::new(text).min_size(size).visuals(button_visuals);

               if ui.add(ok_button).clicked() {
                  self.open = false;
               }
            });
         });
   }
}
