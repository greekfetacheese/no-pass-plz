use super::{AppCtx, SHARED_GUI, app::IndexData};
use eframe::egui::{FontId, Order, RichText, ScrollArea, Spinner, Stroke, Ui, vec2};
use egui_elements::{Button, Label, Modal, MultiLabel, QrImage, SecureTextEdit, Theme};
use secure_types::Zeroize;

/// Main Ui
pub struct Home {
   open: bool,
   show_edit_window: bool,
   show_qr_code: bool,
   qr_loading: bool,
   qr_image: QrImage,
   index_to_edit: u32,
   edited_index: IndexData,
   current_page: u32,
   items_per_page: u32,
}

impl Home {
   pub fn new() -> Self {
      Self {
         open: false,
         show_edit_window: false,
         show_qr_code: false,
         qr_loading: false,
         qr_image: QrImage::empty_with_error("No QR code available".to_string()),
         index_to_edit: 0,
         edited_index: IndexData::default(),
         current_page: 0,
         items_per_page: 10,
      }
   }

   pub fn open(&mut self) {
      self.open = true;
   }

   pub fn show(&mut self, app: AppCtx, theme: &Theme, ui: &mut Ui) {
      if !self.open {
         return;
      }

      let button_visuals = theme.button_visuals();

      self.show_edit_window(app.clone(), theme, ui);
      self.show_qr_code(theme, ui);

      ui.vertical_centered(|ui| {
         ui.add_space(10.0);
         ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
         ui.spacing_mut().button_padding = vec2(6.0, 6.0);

         let items_per_page = self.items_per_page;
         let start = self.current_page * items_per_page;
         let end = start + items_per_page;

         ui.horizontal(|ui| {
            ui.add_space(135.0);
            ui.spacing_mut().item_spacing = vec2(10.0, 0.0);
            ui.spacing_mut().button_padding = vec2(4.0, 4.0);

            let current_page_text = format!("Showing {}-{} entries", start, end);
            let current_page_text = RichText::new(current_page_text).size(theme.typography.large);
            ui.label(current_page_text);

            let text = RichText::new("Prev").size(theme.typography.normal);
            let button = Button::new(text).visuals(button_visuals);

            if ui.add_enabled(self.current_page > 0, button).clicked() {
               self.current_page -= 1;
            }

            let text = RichText::new("Next").size(theme.typography.normal);
            let button = Button::new(text).visuals(button_visuals);

            if ui.add(button).clicked() {
               self.current_page += 1;
            }
         });

         ScrollArea::vertical().show(ui, |ui| {
            ui.set_width(ui.available_width());
            for i in start..end {
               let index_data = app.get_index(i);
               self.show_item(app.clone(), i, index_data, theme, ui);
            }
         });
      });
   }

   pub fn show_item(
      &mut self,
      app: AppCtx,
      index: u32,
      data: Option<IndexData>,
      theme: &Theme,
      ui: &mut Ui,
   ) {
      let frame_width = ui.available_width() * 0.6;
      let frame_height = 60.0;

      let error = theme.colors.error;
      let warning = theme.colors.warning;
      let success = theme.colors.success;

      let exists = data.is_some();
      let index_data = if exists {
         data.as_ref().unwrap()
      } else {
         &IndexData::default()
      };

      let stroke = match index_data.exposed {
         true => Stroke::new(1.0, error),
         false => Stroke::NONE,
      };

      let title_color = match index_data.exposed {
         true => error,
         false => success,
      };

      let button_visuals = theme.button_visuals();
      let frame = theme.frame1.stroke(stroke).outer_margin(0);

      let no_entry_text =
         RichText::new("No entry found").size(theme.typography.normal).color(warning);

      let title_text = if exists {
         let title = index_data.title.clone();
         let final_text = match index_data.exposed {
            true => format!("{} (EXPOSED)", title),
            false => title,
         };
         RichText::new(final_text).size(theme.typography.normal).color(title_color)
      } else {
         no_entry_text
      };

      let title_label = Label::new(title_text, None);

      frame.show(ui, |ui| {
         ui.set_width(frame_width);
         ui.set_height(frame_height);

         ui.horizontal(|ui| {
            let text = format!("{}.", index);
            let text = RichText::new(text).size(theme.typography.normal);
            let index_label = Label::new(text, None);
            let multi_label = MultiLabel::new(vec![index_label, title_label]).inter_spacing(10.0);
            ui.add(multi_label);
         });

         ui.horizontal(|ui| {
            let text = RichText::new("Copy Password").size(theme.typography.small);
            let button = Button::new(text).visuals(button_visuals);
            if ui.add(button).clicked() {
               let password = app.derive_at(index).expect("Deriver instance not found");
               let pass_str = password.unlock_str(|s| String::from(s));
               ui.ctx().copy_text(pass_str);
            }

            let text = RichText::new("QR Code").size(theme.typography.small);
            let button = Button::new(text).visuals(button_visuals);
            if ui.add(button).clicked() {
               self.show_qr_code = true;
               self.encode_qr(app.clone(), index);
            }

            let text = RichText::new("Edit").size(theme.typography.small);
            let button = Button::new(text).visuals(button_visuals);

            if ui.add(button).clicked() {
               self.show_edit_window = true;
               self.index_to_edit = index;
               self.edited_index = index_data.clone();
            }
         });
      });
   }

   fn encode_qr(&mut self, app: AppCtx, index: u32) {
      self.qr_loading = true;

      std::thread::spawn(move || {
         let password = app.derive_at(index).expect("Deriver instance not found");
         let mut data = password.unlock_str(|s| String::from(s));
         let uri = format!("password:{}", index);

         let qr_image = QrImage::new(&data, uri);
         data.zeroize();

         SHARED_GUI.write(|gui| {
            gui.home.qr_image = qr_image;
            gui.home.qr_loading = false;
         });
      });
   }

   fn show_qr_code(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.show_qr_code {
         return;
      }

      let button_visuals = theme.button_visuals();
      let mut open = self.show_qr_code;

      Modal::new("qr_code", &mut open)
         .backdrop_order(Order::Middle)
         .content_order(Order::Foreground)
         .max_width(400.0)
         .close_on_backdrop(false)
         .close_on_escape(false)
         .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
               ui.spacing_mut().item_spacing = vec2(10.0, 25.0);
               ui.spacing_mut().button_padding = vec2(8.0, 8.0);

               if self.qr_loading {
                  ui.add(Spinner::new().size(20.0).color(theme.colors.text));
                  return;
               }

               if let Some(err) = self.qr_image.error() {
                  let text = RichText::new(err.to_string()).size(theme.typography.normal);
                  ui.label(text);
                  return;
               }

               let text = RichText::new("QR Code").size(theme.typography.large);
               ui.label(text);

               let image = self.qr_image.image();
               let size = vec2(250.0, 250.0);
               ui.add(image.fit_to_exact_size(size));

               let text = RichText::new("Close").size(theme.typography.normal);
               let button = Button::new(text).min_size(vec2(100.0, 25.0)).visuals(button_visuals);
               if ui.add(button).clicked() {
                  self.show_qr_code = false;
                  let erased = self.qr_image.clear(ui.ctx());
                  debug_assert!(erased);
               }
            });
         });
   }

   fn show_edit_window(&mut self, app: AppCtx, theme: &Theme, ui: &mut Ui) {
      if !self.show_edit_window {
         return;
      }

      let button_visuals = theme.button_visuals();
      let mut open = self.show_edit_window;

      Modal::new("entry_edit", &mut open)
         .backdrop_order(Order::Middle)
         .content_order(Order::Foreground)
         .max_width(400.0)
         .close_on_backdrop(false)
         .close_on_escape(false)
         .show(ui.ctx(), |ui| {
            ui.vertical_centered(|ui| {
               ui.spacing_mut().item_spacing = vec2(10.0, 10.0);
               ui.spacing_mut().button_padding = vec2(8.0, 8.0);

               let text = RichText::new("Title").size(theme.typography.large);
               ui.label(text);

               let text_edit = SecureTextEdit::singleline(&mut self.edited_index.title)
                  .font(FontId::proportional(theme.typography.normal))
                  .desired_width(ui.available_width() * 0.6)
                  .hint_text("Title");
               ui.add(text_edit);

               let text = RichText::new("Description").size(theme.typography.large);
               ui.label(text);

               let hint_text = RichText::new("Don't write sensitive information here")
                  .size(theme.typography.normal)
                  .color(theme.colors.text_muted);

               let text_edit = SecureTextEdit::multiline(&mut self.edited_index.description)
                  .font(FontId::proportional(theme.typography.normal))
                  .desired_width(ui.available_width() * 0.9)
                  .hint_text(hint_text);
               ui.add(text_edit);

               let text = RichText::new("Exposed").size(theme.typography.normal);
               ui.checkbox(&mut self.edited_index.exposed, text);

               let text = RichText::new("OK").size(theme.typography.normal);
               let button = Button::new(text).min_size(vec2(100.0, 25.0)).visuals(button_visuals);

               if ui.add(button).clicked() {
                  let new_data = self.edited_index.clone();
                  let index = self.index_to_edit;
                  std::thread::spawn(move || {
                     validate_and_save(app, index, new_data);
                  });
               }

               let text = RichText::new("Cancel").size(theme.typography.normal);
               let button = Button::new(text).min_size(vec2(100.0, 25.0)).visuals(button_visuals);

               if ui.add(button).clicked() {
                  self.show_edit_window = false;
               }
            });
         });
   }
}

fn validate_and_save(app: AppCtx, index: u32, data: IndexData) {
   if data.title.is_empty() {
      SHARED_GUI.write(|gui| {
         gui.msg_window.open("Error", "Title cannot be empty");
      });
      return;
   }

   app.set_index(index, data);

   match app.save_index_map_to_file() {
      Ok(_) => {
         SHARED_GUI.write(|gui| {
            gui.home.show_edit_window = false;
            gui.msg_window.open("Success", "Entry saved");
         });
      }
      Err(err) => {
         app.remove_index(index);
         SHARED_GUI.write(|gui| {
            gui.home.show_edit_window = false;
            gui.msg_window.open("Error", err.to_string());
         });
      }
   }
}
