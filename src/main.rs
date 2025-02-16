// بسم الله الرحمن الرحيم

use slint::{ ModelRc, SharedString, Color };
use std::fs;

slint::slint! {
    import { VerticalBox } from "std-widgets.slint";

    struct EditorState {
        command-mode: bool,
        saving-mode: bool,
    }

    export component Vuno inherits Window {
        in property <string> status-message: "";
        in property <string> editor-content: "";
        in-out property <EditorState> state: {
            command-mode: false,
            saving-mode: false,
        };

        callback save-file(string);
        callback handle-command(string);

        VerticalBox {
            background: #000000;
            Text {
                height: 5%;
                color: white;
                text: root.status-message;
                horizontal-alignment: center;
                visible: root.status-message != "";
            }

            TextEdit {
                text: root.editor-content;
                height: root.state.command-mode ? 95% : 100%;
                has-focus: !root.state.command-mode;
                wrap: word-wrap;
                font-size: 14px;
                color: white;

                edited(text) => {
                    root.editor-content = text;
                }

                key-pressed(event) => {
                    if (event.text == Key.Escape) {
                        root.state.command-mode = true;
                    }
                }
            }

            Rectangle {
                background: #000000;
                height: 5%;
                visible: root.state.command-mode;

                TextInput {
                    height: 100%;
                    width: 100%;
                    horizontal-alignment: left;
                    placeholder-text: root.state.saving-mode ?
                        "Enter filename to save..." :
                        "";
                    accepted(text) => {
                        if (root.state.saving-mode) {
                            root.save-file(text);
                            root.state.saving-mode = false;
                            root.state.command-mode = false;
                        } else {
                            root.handle-command(text);
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
    
    // Handle file saving
    app.on_save_file(move |filename| {
        let content = app.get_editor_content();
        match fs::write(&filename, content) {
            Ok(_) => {
                app.set_status_message("File saved successfully!".into());
            }
            Err(err) => {
                app.set_status_message(format!("Error saving file: {}", err).into());
            }
        }
    });

    // Handle commands
    app.on_handle_command(move |cmd| {
        match cmd.as_str() {
            "quit" => {
                std::process::exit(0);
            }
            "save" => {
                app.set_state(slint::private_unstable_api::re_export::Struct1 {
                    command_mode: true,
                    saving_mode: true,
                });
            }
            _ => {
                app.set_status_message(format!("Unknown command: {}", cmd).into());
            }
        }
    });

    app.run().unwrap();
}