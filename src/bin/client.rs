extern crate ted;

use ted::editor::Editor;

fn main() {
    // Run our client editor
    Editor::from_server("104.131.129.181:3910").unwrap().run();
}
