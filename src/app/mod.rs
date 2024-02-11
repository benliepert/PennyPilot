use crate::category::{CategoryManager, CategoryName};
use crate::csvadapter::read_entries_from_file;
use crate::datamanager::DataManager;
use crate::entry::Entry;

mod components;
mod egui_app;

use components::{AddEntry, Entries, Graph};
use egui::{vec2, Ui, Window};
use strum_macros::EnumIter;

use std::collections::HashSet;
#[cfg(target_arch = "wasm32")]
use std::error::Error;
use std::path::PathBuf;
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

pub const SCREENSHOT_PATH: &str = "data/screenshots";

#[derive(Debug, EnumIter, PartialEq, Eq, Copy, Clone)]
pub enum SidePanelSelection {
    Graph,
    Entries,
}

impl std::fmt::Display for SidePanelSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SidePanelSelection::Graph => write!(f, "Graph"),
            SidePanelSelection::Entries => write!(f, "Entries"),
        }
    }
}

/// Track whether various windows are open
pub struct WindowState {
    pub entry_open: bool,
    pub spending_limits_open: bool,
    pub graph_settings_open: bool,
    pub category_editor_open: bool,

    #[cfg(target_arch = "wasm32")]
    pub web_notice_open: bool,
}

// implementing by hand so we always show the notice window on wasm
// allow this warning because clippy doesn't know we need this for wasm
// could conditionally derive Default when not on wasm, and use the block
// below only on wasm, but I think this is more clear.
#[allow(clippy::derivable_impls)]
impl Default for WindowState {
    fn default() -> Self {
        Self {
            entry_open: false,
            spending_limits_open: false,
            graph_settings_open: false,
            category_editor_open: true,

            #[cfg(target_arch = "wasm32")]
            web_notice_open: true,
        }
    }
}

/// On wasm, a user can asynchronously pick a file. Use this message to communicate what they picked
/// across threads so that we can load the file contents
#[cfg(target_arch = "wasm32")]
pub enum FileResponse {
    NoFile,
    FileData(Vec<Entry>),
    Error(Box<dyn Error>),
}

pub struct App {
    pub data_mgr: DataManager,
    pub window_state: WindowState,

    pub cat_mgr: CategoryManager,
    pub entry_view: Entries,
    pub add_entry_view: AddEntry,
    pub graph: Graph,

    #[cfg(target_arch = "wasm32")]
    // Handle asynchronous file import on wasm
    pub file_pick: Arc<Mutex<Option<FileResponse>>>,
}

impl Default for App {
    fn default() -> Self {
        let backend = DataManager::default();

        // backend might deserialize the entry view. Make sure we use it in the app so they're in sync
        // there's probably a better design for this
        debug!("Using sort by = '{}' to init entries...", backend.sort_by);
        let entry_view = Entries {
            sort_by: backend.sort_by,
            ..Default::default()
        };

        Self {
            data_mgr: backend,
            graph: Graph::new(),
            add_entry_view: AddEntry::default(),
            window_state: WindowState::default(),
            entry_view,
            cat_mgr: CategoryManager::default(),
            #[cfg(target_arch = "wasm32")]
            file_pick: Arc::new(Mutex::new(None)),
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     panic!("if you want to add this back, make sure you're happy with the app state that's saved! default WONT be called!");
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        // if you want to register fonts, do this:
        // let mut style = (*_cc.egui_ctx.style()).clone();
        // register_fonts(&mut style);

        Default::default()
    }

    /// Load a file from the user on native
    ///
    /// For wasm handling, see MenuBar::import_button()
    fn import_entries_file(&mut self, file_path: PathBuf) {
        let entries = match read_entries_from_file(&file_path) {
            Ok(entries) => entries,
            Err(e) => {
                error!("Error reading entries from file: {}", e);
                // TODO: show a message to the user
                return;
            }
        };
        if self.data_mgr.active_file != Some(file_path.clone()) {
            self.data_mgr.active_file = Some(file_path);
            // self.serialize_backend();
        }
        self.import_entry_vec(entries);
    }

    /// Import a vector of entries. This doesn't preserve any existing entries.
    fn import_entry_vec(&mut self, entries: Vec<Entry>) {
        info!("Importing entry vector");
        let unique_categories = Self::create_unique_category_set(&entries);
        self.data_mgr.set_entries(entries);

        info!("Appended categories: {unique_categories:?}");
        match self.cat_mgr.append_categories(unique_categories) {
            Ok(_) => (),
            Err(e) => {
                error!("Error appending categories: {}", e);
            }
        }

        // why is this bool part of the data mgr lol?
        // now that we oversee this at the app level, check if this bool is used anywhere else, and just move the flag
        // to the graph since the app knows about it.
        self.data_mgr.plot_reset_next_frame = true;
    }

    fn create_unique_category_set(entries: &[Entry]) -> HashSet<CategoryName> {
        entries.iter().map(|entry| entry.category.clone()).collect()
    }

    /// Display various windows based on window state
    fn show_windows(&mut self, ui: &mut Ui) {
        // Open the entry view window if the button was clicked
        Window::new("Entry View")
            .open(&mut self.window_state.entry_open)
            .default_size(vec2(200.0, 200.0))
            .vscroll(false)
            .show(ui.ctx(), |ui| {
                self.entry_view.ui(ui, &mut self.data_mgr);
            });

        Window::new("Spending Limits")
            .open(&mut self.window_state.spending_limits_open)
            .default_size(vec2(200.0, 400.0))
            .vscroll(false)
            .show(ui.ctx(), |ui| {
                self.cat_mgr.limits_ui(ui, &self.data_mgr);
            });

        Window::new("Graph Settings")
            .open(&mut self.window_state.graph_settings_open)
            .default_size(vec2(200.0, 400.0))
            .vscroll(false)
            .show(ui.ctx(), |ui| {
                self.graph.settings.ui(ui, &mut self.cat_mgr);
            });

        Window::new("Category Editor")
            .open(&mut self.window_state.category_editor_open)
            .default_size(vec2(200.0, 400.0))
            .vscroll(false)
            .show(ui.ctx(), |ui| {
                self.cat_mgr.editor_ui(ui);
            });

        #[cfg(target_arch = "wasm32")]
        egui::Window::new("Web Notice")
            .open(&mut self.window_state.web_notice_open)
            .default_size(egui::vec2(200.0, 200.0))
            .vscroll(false)
            .show(ui.ctx(), |ui| {
                ui.label("Hey! Thanks for using PennyPilot on the web.\nNote: You MUST export data via file -> export in order for it to be saved!");
            });
    }
}
