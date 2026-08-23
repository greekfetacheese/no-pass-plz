use argon2_rs::Argon2;
use eframe::egui::{RichText, Ui, vec2};
use passwd_derive::{PasswordDeriver, default_argon2};
use egui_elements::{Theme, Button, CredentialsForm};

use super::{AppCtx, SHARED_GUI};

pub struct Auth {
   open: bool,
   credentials_form: CredentialsForm,
   argon2: Argon2,
}

impl Auth {
   pub fn new() -> Self {
      let form = CredentialsForm::new().with_open(true).with_confirm_password(true);

      Self {
         open: true,
         credentials_form: form,
         argon2: default_argon2(),
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

   pub fn show_credentials_input(&mut self, app: AppCtx, theme: &Theme, ui: &mut Ui) {
      if !self.credentials_form.is_open() {
         return;
      }

      let button_visuals = theme.button_visuals();

      ui.vertical_centered(|ui| {
         ui.spacing_mut().item_spacing = vec2(10.0, 15.0);
         ui.spacing_mut().button_padding = vec2(10.0, 8.0);

         let form_size = vec2(ui.available_width() * 0.5, 20.0);
         self.credentials_form.set_min_size(form_size);
         self.credentials_form.set_icon_size(vec2(20.0, 20.0));

         ui.scope(|ui| {
            ui.spacing_mut().button_padding = vec2(4.0, 4.0);
            self.credentials_form.show(ui);
         });

         let text = RichText::new("OK").size(theme.typography.normal);
         let button = Button::new(text).min_size(vec2(100.0, 25.0)).visuals(button_visuals);

         if ui.add(button).clicked() {
            self.init_deriver(app.clone());
         }

         #[cfg(feature = "dev")]
         {
            use secure_types::SecureString;
            let text = RichText::new("DEV").size(theme.typography.normal);
            let button = Button::new(text).min_size(vec2(100.0, 25.0)).visuals(button_visuals);
            if ui.add(button).clicked() {
               let username = SecureString::from("dev");
               let password = SecureString::from("dev");
               let confirm_password = SecureString::from("dev");
               let argon2 = Argon2::new(16_000, 1, 1);
               self.credentials_form.set_username_text(username);
               self.credentials_form.set_password_text(password);
               self.credentials_form.set_confirm_password_text(confirm_password);

               self.argon2 = argon2;
               self.init_deriver(app);
            }
         }
      });
   }

   pub fn init_deriver(&self, app: AppCtx) {
      let username = self.credentials_form.username();
      let password = self.credentials_form.password();
      let confirm_password = self.credentials_form.confirm_password();
      let argon2 = self.argon2.clone();

      std::thread::spawn(move || {
         SHARED_GUI.write(|gui| {
            gui.loading_window.open("Please wait... this may take 2-3 minutes");
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
