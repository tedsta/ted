extern crate ted;

use ted::editor::Editor;

fn main() {
    // Run our client editor
    Editor::from_server("104.181.129.131:3910").unwrap().run();
}
