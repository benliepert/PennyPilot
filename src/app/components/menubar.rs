#[cfg(target_arch = "wasm32")]
use crate::app::FileResponse;

use crate::app::App;
use egui::Ui;

// this is necessary because otherwise wasm will try to find this and can't
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

#[cfg(target_arch = "wasm32")]
use rfd::AsyncFileDialog;

pub struct MenuBar {}

/// The menu bar shown at the top of PennyPilot
impl MenuBar {
    pub fn add(app: &mut App, ui: &mut Ui, frame: &mut eframe::Frame) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                Self::import_button(ui, app);
                Self::export_button(ui, app);

                if ui.button("View Entries").clicked() {
                    app.window_state.entry_open = true;
                }

                ui.menu_button("Settings", |ui| {
                    if ui
                        .add_enabled(
                            !app.window_state.graph_settings_open,
                            egui::Button::new("Graph"),
                        )
                        .clicked()
                    {
                        app.window_state.graph_settings_open = true;
                    }
                    if ui
                        .add_enabled(
                            !app.window_state.spending_limits_open,
                            egui::Button::new("Spending Limits"),
                        )
                        .clicked()
                    {
                        app.window_state.spending_limits_open = true;
                    }
                    if ui
                        .add_enabled(
                            !app.window_state.category_editor_open,
                            egui::Button::new("Categories"),
                        )
                        .clicked()
                    {
                        app.window_state.category_editor_open = true;
                    }
                });

                #[cfg(not(target_arch = "wasm32"))] // not supported on wasm
                if ui.button("Quit").clicked() {
                    frame.close();
                }
            });
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn import_button(ui: &mut Ui, app: &mut App) {
        if ui.button("Import").clicked() {
            let file = FileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .pick_file();

            if let Some(file_path) = file {
                app.import_entries_file(file_path);
            }
        }
    }

    // TODO: worth restricting this even further?
    // make a trait to encapsulate "write_entries_to_csv()" functionality, accept an object that implements it here
    // this makes this code 1. more modular / less coupled 2. safer (it can't just mutate the entire data mgr)
    // NOTE: if you do this you need to fix/change the wasm interface, which also requires a pick_file mutex to communicate
    // across threads.
    #[cfg(not(target_arch = "wasm32"))]
    fn export_button(ui: &mut Ui, app: &mut App) {
        if ui.button("Export").clicked() {
            let file = FileDialog::new()
                .add_filter("CSV Files", &["csv"])
                .pick_file();

            app.data_mgr.write_entries_to_csv(file);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn import_button(ui: &mut Ui, app: &mut App) {
        // TODO: add this button as enabled only when the file pick channel is None
        // TODO: genericize the interface to get the path and load the file for native + wasm
        //      impl a new trait for PathBuf and rfd::FilePath so we can read each from a csv
        if ui.button("Import").clicked() {
            let file_pick_clone = app.file_pick.clone();

            // Spawn a thread that does the work of getting the picked file and reading the contents
            // into a vector. The vector is what we send back to the main thread so it can update its
            // data model.
            wasm_bindgen_futures::spawn_local(async move {
                let file = AsyncFileDialog::new().set_directory(".").pick_file().await;
                let mut response = file_pick_clone.lock().unwrap();
                *response = match file {
                    None => {
                        debug!("No file selected. Notifying main app.");
                        Some(FileResponse::NoFile)
                    }
                    Some(handle) => {
                        use crate::csvadapter::read_entries_from_vec;
                        let data = handle.read().await;
                        match read_entries_from_vec(data) {
                            Ok(entry_vec) => Some(FileResponse::FileData(entry_vec)),
                            Err(e) => Some(FileResponse::Error(e)),
                        }
                    }
                };
            });
        }

        let mut data_opt = None;
        if let Ok(mut response) = app.file_pick.try_lock() {
            if let Some(pick) = response.take() {
                debug!("Processing data from async file dialog...");
                match pick {
                    FileResponse::NoFile => debug!("Main thread registered: no file picked"),
                    FileResponse::FileData(data) => {
                        debug!("Main thread registered: data: {data:?}");
                        // app.import_entry_vec(data);
                        data_opt = Some(data);
                    }
                    FileResponse::Error(e) => error!("Error from async file dialog: {e}"),
                }
                // we've consumed the response, we don't need this anymore
                *response = None;
            }
        }
        if let Some(data) = data_opt {
            app.import_entry_vec(data);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn export_button(ui: &mut Ui, app: &mut App) {
        use crate::app;

        if ui.button("Export").clicked() {
            use std::io::Write;
            // collect our entry data in a buffer that we'll be writing out
            let mut buf = vec![];
            for entry in app.data_mgr.get_entries_iter(false) {
                writeln!(buf, "{}", entry.to_csv_string());
            }
            wasm_bindgen_futures::spawn_local(async move {
                // NOTE: this doesn't open a user prompt - it gets us a handle that, when written
                // to, will open up a prompt to choose file
                let handle = AsyncFileDialog::new().set_directory(".").save_file().await;
                match handle {
                    None => {
                        error!("export: didn't get the save file handle");
                    }
                    Some(handle) => {
                        debug!("Got file handle from the browser");
                        // we need the data as a Vec<u8> to pass in;
                        // the browser will ask us where we want to write to now.
                        // unfortunately, there doesn't seem to be a way to set the filename without js interop
                        match handle.write(buf.as_slice()).await {
                            Ok(_) => debug!("Successfully exported data!"),
                            Err(e) => error!("Error while exporting data: {e}"),
                        }
                    }
                }
            });
        }
    }
}
