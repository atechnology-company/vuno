// بسم الله الرحمن الرحيم

use slint::{self, ComponentHandle, Weak, SharedString};
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
        opening-mode: bool,
    }

    export component Vuno inherits Window {
        in-out property <string> window-title: "vuno";
        in-out property <string> current-file: "";
        in-out property <string> editor-content: "";
        in-out property <string> command-line-text: "";
        in-out property <EditorState> state: {
            command-mode: false,
            saving-mode: false,
            opening-mode: false,
        };

        title: window-title;

        callback save-file(string);
        callback open-file(string);
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
                    if event.text == Key.Escape {
                        root.state.command-mode = true;
                        root.focus-cl();
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
                    text <=> root.command-line-text;  // Changed to two-way binding
                    font-size: 14px;

                    enabled: root.state.command-mode;

                    key-pressed(event) => {
                        if event.text == Key.Escape {
                            root.state.saving-mode = false;
                            root.state.opening-mode = false;
                            root.state.command-mode = false;
                            root.command-line-text = "";
                            root.focus-e();
                            return accept;
                        }
                        return reject;
                    }

                    accepted => {
                        if root.state.saving-mode {
                            if self.text != "" {
                                root.save-file(self.text);
                            } else if root.current-file != "" {
                                root.save-file(root.current-file);
                            }
                            root.state.saving-mode = false;
                            root.state.command-mode = false;
                            root.command-line-text = "";
                            root.focus-e();
                        } else if root.state.opening-mode {
                            if self.text != "" {
                                root.open-file(self.text);
                            }
                            root.command-line-text = "";
                        } else {
                            root.handle-command(self.text);
                            root.command-line-text = "";
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let app = Vuno::new().unwrap();

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let filename = &args[1];
        match fs::read_to_string(filename) {
            Ok(content) => {
                app.set_editor_content(SharedString::from(content));
                app.set_current_file(SharedString::from(filename.to_string()));
                app.set_window_title(SharedString::from(format!("vuno - {}", filename)));
            },
            Err(e) => {
                eprintln!("Error opening file: {}", e);
            }
        }
    }

    let weak = app.as_weak();
    
    app.on_save_file(move |filename| {
        let app = weak.unwrap();
        let content = app.get_editor_content();
        if fs::write(&filename, content).is_ok() {
            app.set_current_file(SharedString::from(filename.to_string()));
            app.set_window_title(SharedString::from(format!("vuno - {}", filename)));

            let mut state = app.get_state();
            state.command_mode = false;
            state.saving_mode = false;
            app.set_state(state);
            app.set_command_line_text(SharedString::from(""));
            app.invoke_focus_e();
        }
    });

    let weak = app.as_weak();

    app.on_open_file(move |filename| {
        let app = weak.unwrap();
        if !filename.trim().is_empty() {
            match fs::read_to_string(&filename) {
                Ok(content) => {
                    app.set_editor_content(SharedString::from(content));
                    app.set_current_file(SharedString::from(filename.to_string()));
                    app.set_window_title(SharedString::from(format!("vuno - {}", filename)));

                    let mut state = app.get_state();
                    state.command_mode = false;
                    state.opening_mode = false;
                    app.set_state(state);
                    app.set_command_line_text(SharedString::from(""));
                    app.invoke_focus_e();
                },
                Err(e) => {
                    eprintln!("Error opening file: {}", e);
                    // Keep command mode active on error
                    let mut state = app.get_state();
                    state.command_mode = true;
                    state.opening_mode = true;
                    app.set_state(state);
                    app.set_command_line_text(SharedString::from(""));
                }
            }
        }
    });

    let weak = app.as_weak();

    app.on_handle_command(move |cmd| {
        let app = weak.unwrap();
        if cmd.starts_with("save ") {
            let filename = cmd.trim_start_matches("save ").trim();
            app.invoke_save_file(filename.into());
        } else if cmd.starts_with("open ") {
            let filename = cmd.trim_start_matches("open ").trim();
            if !filename.is_empty() {
                app.invoke_open_file(filename.into());
            }
        } else {
            match cmd.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "save" => {
                    if !app.get_current_file().is_empty() {
                        app.invoke_save_file(app.get_current_file());
                    } else {
                        let mut state = app.get_state();
                        state.command_mode = true;
                        state.saving_mode = true;
                        app.set_state(state);
                    }
                }
                "open" => {
                    let mut state = app.get_state();
                    state.command_mode = true;
                    state.opening_mode = true;
                    app.set_state(state);
                }
                _ => {
                    let mut state = app.get_state();
                    state.command_mode = false;
                    app.set_state(state);
                    app.invoke_focus_e();
                }
            }
        }
    });

    app.invoke_focus_e();
    app.run().unwrap();
}