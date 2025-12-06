use eframe::egui::{
   Align2, Button, Frame, Label, Order, RichText, Spinner, Ui, Vec2, Window, vec2,
};

use zeus_theme::Theme;

pub struct LoadingWindow {
   open: bool,
   pub msg: String,
   pub size: (f32, f32),
   pub anchor: (Align2, Vec2),
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
         size: (200.0, 100.0),
         anchor: (Align2::CENTER_CENTER, vec2(0.0, 0.0)),
      }
   }

   pub fn open(&mut self, msg: impl Into<String>) {
      self.open = true;
      self.msg = msg.into();
   }

   pub fn reset(&mut self) {
      self.open = false;
      self.msg = String::new();
      self.size = (200.0, 100.0);
   }

   pub fn show(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.open {
         return;
      }

      Window::new("Loading")
         .title_bar(false)
         .order(Order::Debug)
         .resizable(false)
         .anchor(self.anchor.0, self.anchor.1)
         .collapsible(false)
         .frame(Frame::window(ui.style()))
         .show(ui.ctx(), |ui| {
            ui.set_width(self.size.0);
            ui.set_height(self.size.1);
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
}

impl MsgWindow {
   pub fn new() -> Self {
      Self {
         open: false,
         title: String::new(),
         message: String::new(),
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

      let title = RichText::new(self.title.clone()).size(theme.text_sizes.heading);
      let msg = RichText::new(&self.message).size(theme.text_sizes.normal);

      Window::new("msg_window")
         .title_bar(false)
         .resizable(false)
         .order(Order::Debug)
         .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
         .collapsible(false)
         .frame(Frame::window(ui.style()))
         .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
               ui.spacing_mut().item_spacing.y = 10.0;
               ui.spacing_mut().button_padding = vec2(10.0, 8.0);

               ui.label(title);

               let label = Label::new(msg).wrap();
               ui.add(label);

               ui.add_space(10.0);

               let size = vec2(ui.available_width() * 0.2, 25.0);
               let ok_button =
                  Button::new(RichText::new("OK").size(theme.text_sizes.normal)).min_size(size);
               if ui.add(ok_button).clicked() {
                  self.open = false;
               }
            });
         });
   }
}
