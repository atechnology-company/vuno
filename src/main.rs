// بسم الله الرحمن الرحيم

use slint::{self, ComponentHandle, Weak};
use std::fs;

slint::slint! {
    import { VerticalBox, TextEdit, LineEdit } from "std-widgets.slint";
    import "src/fonts/r.ttf";
    import "src/fonts/i.ttf";
    import "src/fonts/b.ttf";
    import "src/fonts/bi.ttf";


    struct EditorState {
        command-mode: bool,
        saving-mode: bool,
    }

    export component Vuno inherits Window {
        in-out property <string> editor-content: "";
        in-out property <EditorState> state: {
            command-mode: false,
            saving-mode: false,
        };

        callback save-file(string);
        callback handle-command(string);

        callback focus-cl();
        callback focus-e();

        focus-e => {e.focus();}
        focus-cl => {cl.focus();}

        background: #000000;
        preferred-height: 500px;
        preferred-width: 800px;
        default-font-family: "JetBrains Mono";

        VerticalBox {
            spacing: 0px;
            padding: 0px;

            e := TextEdit {
                text: root.editor-content;
                height: root.state.command-mode ? 95% : 100%;
                read-only: root.state.command-mode;
                wrap: word-wrap;
                font-size: 14px;

                edited => {
                    root.editor-content = self.text;
                }

                key-pressed(event) => {
                    if (event.text == Key.Escape) {
                        root.state.command-mode = true;
                        cl.focus();
                        return accept;
                    }
                    return reject;
                }
            }

            Rectangle {
                height: root.state.command-mode ? 5% : 0%;
                background: #000000;

                cl := LineEdit {
                    height: 100%;
                    width: 100%;
                    horizontal-alignment: left;
                    text: "";
                    font-size: 14px;

                    enabled: root.state.command-mode;

                    key-pressed(event) => {
                        if (event.text == Key.Escape) {
                            root.state.saving-mode = false;
                            root.state.command-mode = false;
                            e.focus();
                            return accept;
                        }
                        return reject;
                    }

                    accepted => {
                        if (root.state.saving-mode) {
                            root.save-file(self.text);
                            root.state.saving-mode = false;
                            root.state.command-mode = false;
                        } else {
                            root.handle-command(self.text);
                        }
                        self.text = "";
                    }
                }
            }

        }
    }
}

fn main() {
    let app = Vuno::new().unwrap();
    let weak = app.as_weak();
    
    app.on_save_file(move |filename| {
        let app = weak.unwrap();
        let content = app.get_editor_content();
        fs::write(&filename, content);
    });

    let weak = app.as_weak();

    app.on_handle_command(move |cmd| {
        let app = weak.unwrap();
        match cmd.as_str() {
            "quit" => {
                std::process::exit(0);
            }
            "save" => {
                let mut state = app.get_state();
                state.command_mode = true;
                state.saving_mode = true;
                app.set_state(state);
            }
            _ => {
                app.invoke_focus_e();
            }
        }
    });

    app.invoke_focus_e();

    app.run().unwrap();
}