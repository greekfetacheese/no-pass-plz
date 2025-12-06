#![allow(dead_code)]

use argon2_rs::Argon2;
use eframe::egui::{Button, FontId, Margin, RichText, Sense, Ui, vec2};
use passwd_derive::{PasswordDeriver, fast, normal, slow, very_slow};
use secure_types::SecureString;
use zeus_theme::{Theme, utils::frame_it};
use zeus_widgets::SecureTextEdit;

use super::{AppCtx, SHARED_GUI};

pub struct CredentialsForm {
   open: bool,
   with_confirm_password: bool,
   username: SecureString,
   password: SecureString,
   confirm_password: SecureString,
}

impl CredentialsForm {
   pub fn new() -> Self {
      Self {
         open: true,
         with_confirm_password: true,
         username: SecureString::new_with_capacity(50).unwrap(),
         password: SecureString::new_with_capacity(50).unwrap(),
         confirm_password: SecureString::new_with_capacity(50).unwrap(),
      }
   }

   pub fn is_open(&self) -> bool {
      self.open
   }

   pub fn open(&mut self) {
      self.open = true;
   }

   pub fn erase(&mut self) {
      self.username.erase();
      self.password.erase();
      self.confirm_password.erase();
   }

   pub fn show(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.open {
         return;
      }

      ui.vertical_centered(|ui| {
         ui.spacing_mut().item_spacing = vec2(10.0, 15.0);

         let ui_width = ui.available_width();
         let text_edit_size = vec2(ui_width * 0.6, 20.0);

         // Username Field
         ui.label(RichText::new("Username").size(theme.text_sizes.large));
         self.username.unlock_mut(|username| {
            let text_edit = SecureTextEdit::singleline(username)
               .min_size(text_edit_size)
               .margin(Margin::same(10))
               .password(false)
               .font(FontId::proportional(theme.text_sizes.normal));
            ui.add(text_edit);
         });

         // Password Field
         ui.label(RichText::new("Password").size(theme.text_sizes.large));
         self.password.unlock_mut(|password| {
            let text_edit = SecureTextEdit::singleline(password)
               .min_size(text_edit_size)
               .margin(Margin::same(10))
               .font(FontId::proportional(theme.text_sizes.normal))
               .password(true);
            ui.add(text_edit);
         });

         // Confirm Password Field
         if self.with_confirm_password {
            ui.label(RichText::new("Confirm Password").size(theme.text_sizes.large));
            self.confirm_password.unlock_mut(|confirm_password| {
               let text_edit = SecureTextEdit::singleline(confirm_password)
                  .min_size(text_edit_size)
                  .margin(Margin::same(10))
                  .font(FontId::proportional(theme.text_sizes.normal))
                  .password(self.with_confirm_password);
               ui.add(text_edit);
            });
         } else {
            self.password.unlock_str(|str| {
               self.confirm_password.erase();
               self.confirm_password.push_str(str);
            });
         }
      });
   }
}

pub struct Auth {
   open: bool,
   show_argon2_selection: bool,
   credentials_form: CredentialsForm,
   argon2: Argon2,
}

impl Auth {
   pub fn new() -> Self {
      Self {
         open: true,
         show_argon2_selection: true,
         credentials_form: CredentialsForm::new(),
         argon2: slow(),
      }
   }

   pub fn close(&mut self) {
      self.open = false;
   }

   pub fn erase(&mut self) {
      self.credentials_form.erase();
   }

   pub fn show(&mut self, app: AppCtx, theme: &Theme, ui: &mut Ui) {
      if !self.open {
         return;
      }

      self.show_credentials_input(app, theme, ui);
   }

   pub fn _show_argon2_selection(&mut self, theme: &Theme, ui: &mut Ui) {
      if !self.show_argon2_selection {
         return;
      }

      let text = RichText::new("Select Argon2 parameters").size(theme.text_sizes.large);
      ui.label(text);

      ui.horizontal(|ui| {
         ui.add_space(250.0);

         ui.vertical_centered(|ui| {
            ui.spacing_mut().item_spacing = vec2(5.0, 3.0);
            ui.spacing_mut().button_padding = vec2(8.0, 8.0);

            let params = vec![fast(), normal(), slow(), very_slow()];
            let est_times = vec!["17 secs", "35 secs", "1:11 min", "2:17 mins"];

            let mut frame = theme.frame2;
            let visuals = theme.frame2_visuals;

            for (i, param) in params.iter().enumerate() {
               let est_time = est_times[i];
               let res = frame_it(&mut frame, Some(visuals), ui, |ui| {
                  self._show_param(param, est_time, theme, ui);
               });

               if res.interact(Sense::click()).clicked() {
                  self.argon2 = param.clone();
                  self.show_argon2_selection = false;
                  self.credentials_form.open();
               }
            }
         });
      });
   }

   fn _show_param(&mut self, param: &Argon2, est_time: &str, theme: &Theme, ui: &mut Ui) {
      ui.horizontal(|ui| {
         let text = RichText::new("Memory cost:").size(theme.text_sizes.normal);
         ui.label(text);

         let memory = _to_gigabytes(param.m_cost);
         let text = RichText::new(format!("{:.2} GB", memory)).size(theme.text_sizes.normal);
         ui.label(text);
      });

      ui.horizontal(|ui| {
         let text = RichText::new("Time cost:").size(theme.text_sizes.normal);
         ui.label(text);

         let text = RichText::new(param.t_cost.to_string()).size(theme.text_sizes.normal);
         ui.label(text);
      });

      ui.horizontal(|ui| {
         let text = RichText::new("Parallelism:").size(theme.text_sizes.normal);
         ui.label(text);

         let text = RichText::new(param.p_cost.to_string()).size(theme.text_sizes.normal);
         ui.label(text);
      });

      ui.horizontal(|ui| {
         let text = RichText::new("Estimated time:").size(theme.text_sizes.normal);
         ui.label(text);

         let text = RichText::new(est_time).size(theme.text_sizes.normal);
         ui.label(text);
      });
   }

   pub fn show_credentials_input(&mut self, app: AppCtx, theme: &Theme, ui: &mut Ui) {
      if !self.credentials_form.is_open() {
         return;
      }

      ui.vertical_centered(|ui| {
         ui.spacing_mut().item_spacing = vec2(10.0, 15.0);
         ui.spacing_mut().button_padding = vec2(8.0, 8.0);

         self.credentials_form.show(theme, ui);

         let text = RichText::new("OK").size(theme.text_sizes.normal);
         let button = Button::new(text).min_size(vec2(100.0, 25.0));

         if ui.add(button).clicked() {
            self.init_deriver(app.clone());
         }

         #[cfg(feature = "dev")]
         {
            let text = RichText::new("DEV").size(theme.text_sizes.normal);
            let button = Button::new(text).min_size(vec2(100.0, 25.0));
            if ui.add(button).clicked() {
               let username = SecureString::from("dev");
               let password = SecureString::from("dev");
               let confirm_password = SecureString::from("dev");
               let argon2 = Argon2::new(16_000, 1, 1);
               self.credentials_form.username = username;
               self.credentials_form.password = password;
               self.credentials_form.confirm_password = confirm_password;
               self.argon2 = argon2;
               self.init_deriver(app);
            }
         }
      });
   }

   pub fn init_deriver(&self, app: AppCtx) {
      let username = self.credentials_form.username.clone();
      let password = self.credentials_form.password.clone();
      let confirm_password = self.credentials_form.confirm_password.clone();
      let argon2 = self.argon2.clone();

      std::thread::spawn(move || {
         SHARED_GUI.write(|gui| {
            gui.loading_window.open("Please wait... this may take a minute or two");
         });

         let deriver = match PasswordDeriver::new(username, password, confirm_password, argon2) {
            Ok(deriver) => deriver,
            Err(err) => {
               SHARED_GUI.write(|gui| {
                  gui.msg_window.open("Error", err.to_string());
                  gui.loading_window.reset();
               });
               return;
            }
         };

         SHARED_GUI.write(|gui| {
            gui.loading_window.reset();
            gui.auth.close();
            gui.auth.erase();
            gui.home.open();
            gui.request_repaint();
         });

         app.write(|app| {
            app.passwd_derive = Some(deriver);
         });
      });
   }
}

fn _to_gigabytes(kibi: u32) -> f64 {
   let bytes = kibi as u64 * 1024;
   bytes as f64 / 1_000_000_000.0
}
