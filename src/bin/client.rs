extern crate ted;

use ted::editor::Editor;

fn main() {
    // Run our client editor
    Editor::from_server("127.0.0.1:3910").unwrap().run();
}
