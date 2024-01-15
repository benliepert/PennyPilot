use crate::category::CategoryName;
use crate::colors::*;
use crate::organize::*;

use crate::datamanager::*;
use egui::CollapsingHeader;
use egui::{
    plot::{Bar, BarChart, Legend, Plot, PlotPoint},
    Grid, Response, Ui,
};
use std::collections::HashMap;
use std::ops::RangeInclusive;
use strum::IntoEnumIterator;

/// In charge of plotting planner data. Stores its own settings, which can draw a ui to edit them.
pub struct Graph {
    // the graph settings window
    pub settings: GraphSettings,
}

const DATA_ASPECT: f32 = 0.5;
impl Default for Graph {
    fn default() -> Self {
        let group_by = GroupBy::Month;

        let (width, spacing) = get_width_spacing(group_by);

        let settings = GraphSettings {
            width,
            spacing,
            data_aspect: DATA_ASPECT,
            theme: Theme::Sunset,
            group_by,
            category_selector: CategorySelector::new(),
        };

        Self { settings }
    }
}

fn get_width_spacing(group_by: GroupBy) -> (f64, f64) {
    match group_by {
        GroupBy::Day => (0.7, 1.0),
        GroupBy::Month => (21.0, 30.0),
        GroupBy::Year => (252.0, 360.0),
    }
}

impl Graph {
    pub fn ui(&mut self, ui: &mut Ui, data_mgr: &mut DataManager) -> Response {
        let chart = self.build_chart(data_mgr);
        self.plot(ui, chart, &mut data_mgr.plot_reset_next_frame)
    }

    fn build_chart(&self, backend: &DataManager) -> Vec<BarChart> {
        let map = backend.cost_map(
            self.settings.group_by(),
            self.settings.selected_categories(),
        );

        // TODO: calculate this based on width as well since a wide bar will pass over the line x = 0
        // used to track spacing between bars
        let counter = self.settings.spacing() / 2.0;

        let colors = self.settings.theme().colors();
        let mut colors_idx = 0;

        let mut bar_charts: Vec<BarChart> = Vec::new();
        for (category, inner_map) in &map {
            // every category except the 'All' category (which shouldn't be in the data anyway) gets graphed
            let bars: Vec<_> = inner_map
                .iter()
                .enumerate()
                .map(|(idx, (date, cost))| {
                    Bar::new(counter + idx as f64 * self.settings.spacing(), *cost as f64)
                        .fill(colors[colors_idx])
                        .name(format!("{}: {}", date, category))
                })
                .collect();

            let refs: Vec<&BarChart> = bar_charts.iter().collect();
            let chart = BarChart::new(bars)
                .width(self.settings.width())
                .color(colors[colors_idx])
                .name(category.to_string())
                .stack_on(&refs[..]);

            bar_charts.push(chart);

            // use a different color per category
            colors_idx += 1;
        }

        bar_charts
    }

    /// Plot various bar charts on a ui
    fn plot(&self, ui: &mut Ui, charts: Vec<BarChart>, data_loaded: &mut bool) -> Response {
        // no x labels until (if) I can get custom labels working
        let x_fmt = |_x, _range: &RangeInclusive<f64>| String::new();

        // since we've removed x axis labels for now, just use the y value ($)
        let y_fmt = |y, _range: &RangeInclusive<f64>| format!("${}", y);

        // formatter used for the cursor label when floating on the graph
        let label_fmt = |_s: &str, val: &PlotPoint| format!("${:.2}", val.x);

        // Construct the base plot
        let mut plot = Plot::new("Bar Plot")
            .legend(Legend::default())
            .data_aspect(self.settings.data_aspect())
            .x_axis_formatter(x_fmt)
            .y_axis_formatter(y_fmt)
            .label_formatter(label_fmt);

        // Reset the plot if data was loaded
        if *data_loaded {
            debug!("Plot detected data was loaded!");
            *data_loaded = false;
            plot = plot.reset();
        }

        // Show the plot
        plot.show(ui, |plot_ui| {
            for chart in charts {
                plot_ui.bar_chart(chart);
            }
        })
        .response
    }
}

/// Store settings related to `Graph`. Can draw an egui UI that edits itself.
/// Only editable via the UI.
#[derive(Clone)]
pub struct GraphSettings {
    width: f64,
    spacing: f64,
    data_aspect: f32,

    theme: Theme,
    group_by: GroupBy,
    category_selector: CategorySelector,
}

impl GraphSettings {
    // TODO: is it OK for this not to return a response?
    pub fn ui(&mut self, ui: &mut Ui) {
        Grid::new("grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Theme:");
                egui::ComboBox::from_id_source("theme")
                    .selected_text(format!("{}", self.theme))
                    .show_ui(ui, |ui| {
                        for theme in Theme::iter() {
                            ui.selectable_value(&mut self.theme, theme, theme.to_string());
                        }
                    });
                ui.end_row();

                ui.label("Group by:");
                egui::ComboBox::from_id_source("group")
                    .selected_text(format!("{}", self.group_by))
                    .show_ui(ui, |ui| {
                        let prev_group = self.group_by;
                        for group in GroupBy::iter() {
                            ui.selectable_value(&mut self.group_by, group, group.to_string());
                        }
                        if prev_group != self.group_by {
                            // it changed, update our width/spacing to compensate
                            self.reset_bar_sizing();
                        }
                    });
                ui.end_row();
            });
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Bar Width:");
                // need to be less than the spacing below
                ui.add(egui::Slider::new(&mut self.width, 0.5..=self.spacing));

                ui.label("Bar Spacing:");
                // needs to be greater than the min width above
                ui.add(egui::Slider::new(&mut self.spacing, self.width..=400.0));

                ui.label("Data Aspect:");
                ui.add(egui::Slider::new(&mut self.data_aspect, 0.5..=10.0));

                if ui
                    .button("Reset")
                    .on_hover_text("Reset width, spacing, and aspect to defaults")
                    .clicked()
                {
                    debug!("Reset Bar Settings Clicked");
                    // change everything back to defaults
                    self.reset_bar_sizing();
                }
            });
        });
        self.category_selector.show(ui);
    }
}

/// Getters
impl GraphSettings {
    /// The only reason this returns a reference is so that Graph can call get() on it without having to store it in its
    /// own variable (it's dropped otherwise)
    fn theme(&self) -> &Theme {
        &self.theme
    }

    fn group_by(&self) -> GroupBy {
        self.group_by
    }

    fn width(&self) -> f64 {
        self.width
    }
    fn spacing(&self) -> f64 {
        self.spacing
    }
    fn data_aspect(&self) -> f32 {
        self.data_aspect
    }
    fn selected_categories(&self) -> Vec<CategoryName> {
        self.category_selector.selected_categories()
    }

    fn reset_bar_sizing(&mut self) {
        (self.width, self.spacing) = get_width_spacing(self.group_by);
        self.data_aspect = DATA_ASPECT;
    }
}

/// Track what `Category`s we'd like to graph
// TODO: move this to the category manager?
// Otherwise we need to duplicate all the categories here...
// and make sure they stay updated when someone adds new ones
#[derive(Default, Clone)]
struct CategorySelector {
    selections: HashMap<CategoryName, bool>,
}

impl CategorySelector {
    fn new() -> Self {
        let categories: Vec<CategoryName> = Vec::new();
        let selections = categories
            .iter()
            .map(|category| (category.clone(), true))
            .collect();
        CategorySelector { selections }
    }

    fn selected_categories(&self) -> Vec<CategoryName> {
        self.selections
            .iter()
            .filter(|(_, &value)| value)
            .map(|(&ref key, _)| key.clone())
            .collect()
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        CollapsingHeader::new("Displayed Categories:")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let all_true = self.selections.iter().all(|(_, &value)| value);
                    let all_false = self.selections.iter().all(|(_, &value)| !value);
                    if ui
                        .add_enabled(!all_true, egui::Button::new("Select All"))
                        .clicked()
                    {
                        self.selections.values_mut().for_each(|value| *value = true);
                    };
                    if ui
                        .add_enabled(!all_false, egui::Button::new("Deselect All"))
                        .clicked()
                    {
                        self.selections
                            .values_mut()
                            .for_each(|value| *value = false);
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        self.selections
                            .keys()
                            .cloned() // Clone the keys to avoid borrow issues
                            .collect::<Vec<_>>() // Collect into a Vec to avoid borrow checker issues
                            .iter() // Iterate over the cloned keys
                            .for_each(|category| {
                                let label = category.to_string();
                                // Use `get_mut` instead of `entry` to avoid double mutable borrow
                                if let Some(selected) = self.selections.get_mut(category) {
                                    ui.checkbox(selected, &label);
                                }
                            });
                    });
                });
            });
    }
}
