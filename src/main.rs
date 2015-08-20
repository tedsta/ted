#![feature(convert)]

extern crate rustbox;

use editor::Editor;

mod buffer;
mod command;
mod editor;
mod view;

fn main() {
    Editor::new().run();
}
