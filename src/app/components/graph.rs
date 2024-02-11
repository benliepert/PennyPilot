use crate::category::CategoryInfo;
use crate::category::CategoryManager;
use crate::category::CategoryName;
use crate::colors::*;
use crate::organize::*;

use crate::datamanager::*;
use egui::CollapsingHeader;
use egui::{
    plot::{Bar, BarChart, Legend, Plot, PlotPoint},
    Grid, Response, Ui,
};
use std::collections::BTreeMap;
use std::ops::RangeInclusive;
use strum::IntoEnumIterator;

/// In charge of plotting planner data. Stores its own settings, which can draw a ui to edit them.
pub struct Graph {
    // the graph settings window
    pub settings: GraphSettings,

    pub plot_reset_next_frame: bool,
}

const DATA_ASPECT: f32 = 0.5;

fn get_width_spacing(group_by: GroupBy) -> (f64, f64) {
    match group_by {
        GroupBy::Day => (0.7, 1.0),
        GroupBy::Month => (21.0, 30.0),
        GroupBy::Year => (252.0, 360.0),
    }
}

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
        };

        Self {
            settings,
            plot_reset_next_frame: false,
        }
    }
}

impl Graph {
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        data_mgr: &mut DataManager,
        cat_mgr: &CategoryManager,
    ) -> Response {
        let chart = self.build_chart(data_mgr, cat_mgr);
        self.plot(ui, chart)
    }

    fn build_chart(&self, data_mgr: &DataManager, cat_mgr: &CategoryManager) -> Vec<BarChart> {
        let map = data_mgr.cost_map(self.settings.group_by(), cat_mgr.selected_categories());

        // TODO: calculate this based on width as well since a wide bar will pass over the line x = 0
        // used to track spacing between bars
        let counter = self.settings.spacing() / 2.0;

        let colors = self.settings.theme().colors();
        let mut bar_charts: Vec<BarChart> = Vec::new();
        for (colors_idx, (category, inner_map)) in map.iter().enumerate() {
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
        }

        bar_charts
    }

    /// Plot various bar charts on a ui
    fn plot(&mut self, ui: &mut Ui, charts: Vec<BarChart>) -> Response {
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
        if self.plot_reset_next_frame {
            debug!("Plot detected data was loaded! Resetting plot view");
            self.plot_reset_next_frame = false;
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
// #[derive(Clone)]
pub struct GraphSettings {
    width: f64,
    spacing: f64,
    data_aspect: f32,

    theme: Theme,
    group_by: GroupBy,
}

impl GraphSettings {
    pub fn ui(&mut self, ui: &mut Ui, cat_mgr: &mut CategoryManager) {
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
        self.display_category_ui(ui, &mut cat_mgr.categories);
    }

    // passing the whole name : info map for simplicity, even though we use whether it's displayed or not
    fn display_category_ui(
        &mut self,
        ui: &mut egui::Ui,
        categories: &mut BTreeMap<CategoryName, CategoryInfo>,
    ) {
        CollapsingHeader::new("Displayed Categories:")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let all_true = categories.iter().all(|(_, &info)| info.displayed);
                    let all_false = categories.iter().all(|(_, &info)| !info.displayed);
                    if ui
                        .add_enabled(!all_true, egui::Button::new("Select All"))
                        .clicked()
                    {
                        categories
                            .values_mut()
                            .for_each(|info| info.displayed = true);
                    };
                    if ui
                        .add_enabled(!all_false, egui::Button::new("Deselect All"))
                        .clicked()
                    {
                        categories
                            .values_mut()
                            .for_each(|info| info.displayed = false);
                    }
                });
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.vertical(|ui| {
                        categories
                            .keys()
                            .cloned() // Clone the keys to avoid borrow issues
                            .collect::<Vec<_>>() // Collect into a Vec to avoid borrow checker issues
                            .iter() // Iterate over the cloned keys
                            .for_each(|category| {
                                let label = category.to_string();
                                // Use `get_mut` instead of `entry` to avoid double mutable borrow
                                if let Some(info) = categories.get_mut(category) {
                                    ui.checkbox(&mut info.displayed, &label);
                                }
                            });
                    });
                });
            });
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

    fn reset_bar_sizing(&mut self) {
        (self.width, self.spacing) = get_width_spacing(self.group_by);
        self.data_aspect = DATA_ASPECT;
    }
}
