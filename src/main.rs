#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod bar;
mod buddy;
mod brightness_wmi;
mod cal;
mod cast;
mod pixdata;
mod scene;
mod shelf;
mod gfx;
mod i18n;
mod pop;
mod media;
mod note_edit;
mod radio;
mod store;
mod sys;
mod ui;
mod gfx_win;

include!("win_main.rs");
