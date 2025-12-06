#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::gui::app::App;
use eframe::{
    egui,
    egui_wgpu::{WgpuConfiguration, WgpuSetup, WgpuSetupCreateNew},
    wgpu::{self, MemoryHints, Trace},
};
use std::sync::Arc;

mod gui;

fn main() -> eframe::Result {
    let wgpu_setup = WgpuSetup::CreateNew(WgpuSetupCreateNew {
        device_descriptor: Arc::new(|_adapter| wgpu::DeviceDescriptor {
            memory_hints: MemoryHints::MemoryUsage,
            trace: Trace::Off,
            ..Default::default()
        }),
        ..Default::default()
    });

    let wgpu_config = WgpuConfiguration {
        wgpu_setup,
        ..Default::default()
    };

    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_config,
        viewport: egui::ViewportBuilder::default()
            .with_decorations(true)
            .with_inner_size([780.0, 600.0])
            .with_min_inner_size([780.0, 600.0])
            .with_transparent(true)
            .with_resizable(true),

        ..Default::default()
    };

    eframe::run_native(
        "NoPassPlz",
        options,
        Box::new(|cc| {
            let app = App::new(cc);

            Ok(Box::new(app))
        }),
    )
}
