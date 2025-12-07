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
               ui.set_height(400.0);

               ScrollArea::vertical().show(ui, |ui| {
                  let mut cache = CommonMarkCache::default();
                  CommonMarkViewer::new().show(ui, &mut cache, MARKDOWN);
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


const MARKDOWN: &str =
r"NoPassPlz is a deterministic password generator. Unlike traditional password managers like Bitwarden, your passwords are never stored in the cloud or even locally, they are always derived on-the-fly from your master username and password. Think of it as generating high-entropy passwords from a single set of master credentials.

## This is still WIP I may introduce breaking changes in the future.

## How It Works

Given your master username and password, we first compute a seed using the Argon2id key derivation function (KDF) with these default parameters:
- **Salt**: SHA3-512 hash of the username
- **Memory cost**: 8192 MB
- **Iterations**: 8
- **Parallelism**: 1
- **Output length**: 64 bytes


As of 2025 the estimated computation time for these parameters is about 1 min and 11 seconds. (For most consumer hardware, give or take a couple of seconds)

For each password, we then derive it using HMAC-SHA3-512:
- The Argon2id output is used as the HMAC key.
- A user-selected index (a simple integer, like 0, 1, 2...) is the message.
- The result is a 512-bit (64-byte) hash, encoded as a 128-character hexadecimal string.

This allows to derive an unlimited number of unique, high-entropy passwords from the same master credentials, all without storing any secrets.

# FAQ

### Can I replace an existing password manager with NoPassPlz?

I wouldn't recommend it for most people, while you can deterministically generate as many passwords as you want because of its stateless nature,
it cannot really function very well as a password manager, but you can use it as a password generator for your most important services.

For example password managers like Bitwarden require a `master password` to unlock your vault, and if you not know it already there is no
way to recover it if you forget it you are locked out of your account. With `NoPassPlz` you can generate a high-entropy password for such use cases and be able to always recover it assuming you don't forget your master `Credentials`.

### Does NoPassPlz store any data?

The only data that is stored locally is the a map of `Index Data` which contains metadata about a password entry on a given index.
This data doesn't expose any secrets and is stored in a file called `NoPassPlz.json` in the same directory as the executable and is completely safe to make backups of it even unecrypted.

### What metadata is stored?
Each entry includes:
- **Title** (string, required): Where the password is used (e.g., Google Account).
- **Description** (string, optional): Additional notes (e.g., Main email login).
- **Exposed** (bool, optional): Flag if the password has ever been compromised (e.g., via a breach).

### What if I lose the `NoPassPlz.json` file?

You can still regenerate passwords by manually entering the correct index in the app. You'll just need to remember or rediscover which indexes correspond to which accounts.

### What if I forget my master credentials?

If you forget your master credentials you are never be able to recover your passwords.
It is important to create a username and a password in your mind that not only it's stored anywhere but also
not easy to guess, so you can always recover your passwords.

Some tips:

- Username doesn't necessarily need to be something secret or hard to guess, the most important thing is your password.
- Use a long enough password, at least 20+ characters.
- Do not store your password in any digital or physical form, it's should only be stored in your mind.
- Your master password doesn't have to be a very long sequence of nosense words, you could possible
create your own seed phrase based on something you know but is not publicly available information or easy to guess.
";