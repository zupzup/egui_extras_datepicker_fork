//! This is a fork of the datepicker from `egui_extras` (https://github.com/emilk/egui/tree/master/crates/egui_extras)

use chrono::{Datelike, Duration, NaiveDate, Weekday};
use egui::{
    Align, Area, Button, Color32, ComboBox, Direction, Frame, Id, InnerResponse, Key, Layout,
    Order, RichText, Ui, Vec2, Widget,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};

#[derive(Debug)]
struct Week {
    number: u8,
    days: Vec<NaiveDate>,
}

fn month_data(year: i32, month: u32) -> Vec<Week> {
    let first = NaiveDate::from_ymd_opt(year, month, 1).expect("Could not create NaiveDate");
    let mut start = first;
    while start.weekday() != Weekday::Mon {
        start = start.checked_sub_signed(Duration::days(1)).unwrap();
    }
    let mut weeks = vec![];
    let mut week = vec![];
    while start < first || start.month() == first.month() || start.weekday() != Weekday::Mon {
        week.push(start);

        if start.weekday() == Weekday::Sun {
            weeks.push(Week {
                number: start.iso_week().week() as u8,
                days: std::mem::take(&mut week),
            });
        }
        start = start.checked_add_signed(Duration::days(1)).unwrap();
    }

    weeks
}

#[derive(Default, Clone)]
pub(crate) struct DatePickerButtonState {
    pub picker_visible: bool,
}

/// Shows a date, and will open a date picker popup when clicked.
pub struct DatePickerButton<'a> {
    selection: &'a mut NaiveDate,
    id_salt: Option<&'a str>,
    combo_boxes: bool,
    arrows: bool,
    calendar: bool,
    calendar_week: bool,
    show_icon: bool,
    format: String,
    highlight_weekends: bool,
}

impl<'a> DatePickerButton<'a> {
    pub fn new(selection: &'a mut NaiveDate) -> Self {
        Self {
            selection,
            id_salt: None,
            combo_boxes: true,
            arrows: true,
            calendar: true,
            calendar_week: true,
            show_icon: true,
            format: "%Y-%m-%d".to_owned(),
            highlight_weekends: true,
        }
    }

    /// Add id source.
    /// Must be set if multiple date picker buttons are in the same Ui.
    #[inline]
    pub fn id_salt(mut self, id_salt: &'a str) -> Self {
        self.id_salt = Some(id_salt);
        self
    }

    /// Add id source.
    /// Must be set if multiple date picker buttons are in the same Ui.
    #[inline]
    #[deprecated = "Renamed id_salt"]
    pub fn id_source(self, id_salt: &'a str) -> Self {
        self.id_salt(id_salt)
    }

    /// Show combo boxes in date picker popup. (Default: true)
    #[inline]
    pub fn combo_boxes(mut self, combo_boxes: bool) -> Self {
        self.combo_boxes = combo_boxes;
        self
    }

    /// Show arrows in date picker popup. (Default: true)
    #[inline]
    pub fn arrows(mut self, arrows: bool) -> Self {
        self.arrows = arrows;
        self
    }

    /// Show calendar in date picker popup. (Default: true)
    #[inline]
    pub fn calendar(mut self, calendar: bool) -> Self {
        self.calendar = calendar;
        self
    }

    /// Show calendar week in date picker popup. (Default: true)
    #[inline]
    pub fn calendar_week(mut self, week: bool) -> Self {
        self.calendar_week = week;
        self
    }

    /// Show the calendar icon on the button. (Default: true)
    #[inline]
    pub fn show_icon(mut self, show_icon: bool) -> Self {
        self.show_icon = show_icon;
        self
    }

    /// Change the format shown on the button. (Default: %Y-%m-%d)
    /// See [`chrono::format::strftime`] for valid formats.
    #[inline]
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = format.into();
        self
    }

    /// Highlight weekend days. (Default: true)
    #[inline]
    pub fn highlight_weekends(mut self, highlight_weekends: bool) -> Self {
        self.highlight_weekends = highlight_weekends;
        self
    }
}

impl<'a> Widget for DatePickerButton<'a> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let id = ui.make_persistent_id(self.id_salt);
        let mut button_state = ui
            .data_mut(|data| data.get_persisted::<DatePickerButtonState>(id))
            .unwrap_or_default();

        let mut text = if self.show_icon {
            RichText::new(format!("{} 📆", self.selection.format(&self.format)))
        } else {
            RichText::new(format!("{}", self.selection.format(&self.format)))
        };
        let visuals = ui.visuals().widgets.open;
        if button_state.picker_visible {
            text = text.color(visuals.text_color());
        }
        let mut button = Button::new(text);
        if button_state.picker_visible {
            button = button.fill(visuals.weak_bg_fill).stroke(visuals.bg_stroke);
        }
        let mut button_response = ui.add(button);
        if button_response.clicked() {
            button_state.picker_visible = true;
            ui.data_mut(|data| data.insert_persisted(id, button_state.clone()));
        }

        if button_state.picker_visible {
            let width = 333.0;
            let mut pos = button_response.rect.left_bottom();
            let width_with_padding = width
                + ui.style().spacing.item_spacing.x
                + ui.style().spacing.window_margin.left
                + ui.style().spacing.window_margin.right;
            if pos.x + width_with_padding > ui.clip_rect().right() {
                pos.x = button_response.rect.right() - width_with_padding;
            }

            // Check to make sure the calendar never is displayed out of window
            pos.x = pos.x.max(ui.style().spacing.window_margin.left);

            //TODO(elwerene): Better positioning

            let InnerResponse {
                inner: saved,
                response: area_response,
            } = Area::new(ui.make_persistent_id(self.id_salt))
                .kind(egui::UiKind::Picker)
                .order(Order::Foreground)
                .fixed_pos(pos)
                .show(ui.ctx(), |ui| {
                    let frame = Frame::popup(ui.style());
                    frame
                        .show(ui, |ui| {
                            ui.set_min_width(width);
                            ui.set_max_width(width);

                            DatePickerPopup {
                                selection: self.selection,
                                button_id: id,
                                combo_boxes: self.combo_boxes,
                                arrows: self.arrows,
                                calendar: self.calendar,
                                calendar_week: self.calendar_week,
                                highlight_weekends: self.highlight_weekends,
                            }
                            .draw(ui)
                        })
                        .inner
                });

            if saved {
                button_response.mark_changed();
            }

            if !button_response.clicked()
                && (ui.input(|i| i.key_pressed(Key::Escape)) || area_response.clicked_elsewhere())
            {
                button_state.picker_visible = false;
                ui.data_mut(|data| data.insert_persisted(id, button_state));
            }
        }

        button_response
    }
}

#[derive(Clone, Debug, Default)]
struct DatePickerPopupState {
    year: i32,
    month: u32,
    day: u32,
    setup: bool,
}

impl DatePickerPopupState {
    fn last_day_of_month(&self) -> u32 {
        let date: NaiveDate =
            NaiveDate::from_ymd_opt(self.year, self.month, 1).expect("Could not create NaiveDate");
        date.with_day(31)
            .map(|_| 31)
            .or_else(|| date.with_day(30).map(|_| 30))
            .or_else(|| date.with_day(29).map(|_| 29))
            .unwrap_or(28)
    }
}

pub(crate) struct DatePickerPopup<'a> {
    pub selection: &'a mut NaiveDate,
    pub button_id: Id,
    pub combo_boxes: bool,
    pub arrows: bool,
    pub calendar: bool,
    pub calendar_week: bool,
    pub highlight_weekends: bool,
}

impl<'a> DatePickerPopup<'a> {
    /// Returns `true` if user pressed `Save` button.
    pub fn draw(&mut self, ui: &mut Ui) -> bool {
        let id = ui.make_persistent_id("date_picker");
        let today = chrono::offset::Utc::now().date_naive();
        let mut popup_state = ui
            .data_mut(|data| data.get_persisted::<DatePickerPopupState>(id))
            .unwrap_or_default();
        if !popup_state.setup {
            popup_state.year = self.selection.year();
            popup_state.month = self.selection.month();
            popup_state.day = self.selection.day();
            popup_state.setup = true;
            ui.data_mut(|data| data.insert_persisted(id, popup_state.clone()));
        }

        let weeks = month_data(popup_state.year, popup_state.month);
        let (mut close, mut saved) = (false, false);
        let height = 20.0;
        let spacing = 2.0;
        ui.spacing_mut().item_spacing = Vec2::splat(spacing);

        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend); // Don't wrap any text

        StripBuilder::new(ui)
            .clip(false)
            .sizes(
                Size::exact(height),
                match (self.combo_boxes, self.arrows) {
                    (true, true) => 2,
                    (true, false) | (false, true) => 1,
                    (false, false) => 0,
                },
            )
            .sizes(
                Size::exact((spacing + height) * (weeks.len() + 1) as f32),
                self.calendar as usize,
            )
            .size(Size::exact(height))
            .vertical(|mut strip| {
                if self.combo_boxes {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 3).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ComboBox::from_id_salt("date_picker_year")
                                    .selected_text(popup_state.year.to_string())
                                    .show_ui(ui, |ui| {
                                        for year in today.year() - 100..today.year() + 10 {
                                            if ui
                                                .selectable_value(
                                                    &mut popup_state.year,
                                                    year,
                                                    year.to_string(),
                                                )
                                                .changed()
                                            {
                                                popup_state.day = popup_state
                                                    .day
                                                    .min(popup_state.last_day_of_month());
                                                ui.memory_mut(|mem| {
                                                    mem.data
                                                        .insert_persisted(id, popup_state.clone());
                                                });
                                            }
                                        }
                                    });
                            });
                            strip.cell(|ui| {
                                ComboBox::from_id_salt("date_picker_month")
                                    .selected_text(month_name(popup_state.month))
                                    .show_ui(ui, |ui| {
                                        for month in 1..=12 {
                                            if ui
                                                .selectable_value(
                                                    &mut popup_state.month,
                                                    month,
                                                    month_name(month),
                                                )
                                                .changed()
                                            {
                                                popup_state.day = popup_state
                                                    .day
                                                    .min(popup_state.last_day_of_month());
                                                ui.memory_mut(|mem| {
                                                    mem.data
                                                        .insert_persisted(id, popup_state.clone());
                                                });
                                            }
                                        }
                                    });
                            });
                            strip.cell(|ui| {
                                ComboBox::from_id_salt("date_picker_day")
                                    .selected_text(popup_state.day.to_string())
                                    .show_ui(ui, |ui| {
                                        for day in 1..=popup_state.last_day_of_month() {
                                            if ui
                                                .selectable_value(
                                                    &mut popup_state.day,
                                                    day,
                                                    day.to_string(),
                                                )
                                                .changed()
                                            {
                                                ui.memory_mut(|mem| {
                                                    mem.data
                                                        .insert_persisted(id, popup_state.clone());
                                                });
                                            }
                                        }
                                    });
                            });
                        });
                    });
                }

                if self.arrows {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 6).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                    if ui
                                        .button("<<<")
                                        .on_hover_text("subtract one year")
                                        .clicked()
                                    {
                                        popup_state.year -= 1;
                                        popup_state.day =
                                            popup_state.day.min(popup_state.last_day_of_month());
                                        ui.data_mut(|data| {
                                            data.insert_persisted(id, popup_state.clone());
                                        });
                                    }
                                });
                            });
                            strip.cell(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                    if ui
                                        .button("<<")
                                        .on_hover_text("subtract one month")
                                        .clicked()
                                    {
                                        popup_state.month -= 1;
                                        if popup_state.month == 0 {
                                            popup_state.month = 12;
                                            popup_state.year -= 1;
                                        }
                                        popup_state.day =
                                            popup_state.day.min(popup_state.last_day_of_month());
                                        ui.data_mut(|data| {
                                            data.insert_persisted(id, popup_state.clone());
                                        });
                                    }
                                });
                            });
                            strip.cell(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                    if ui.button("<").on_hover_text("subtract one day").clicked() {
                                        popup_state.day -= 1;
                                        if popup_state.day == 0 {
                                            popup_state.month -= 1;
                                            if popup_state.month == 0 {
                                                popup_state.year -= 1;
                                                popup_state.month = 12;
                                            }
                                            popup_state.day = popup_state.last_day_of_month();
                                        }
                                        ui.data_mut(|data| {
                                            data.insert_persisted(id, popup_state.clone());
                                        });
                                    }
                                });
                            });
                            strip.cell(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                    if ui.button(">").on_hover_text("add one day").clicked() {
                                        popup_state.day += 1;
                                        if popup_state.day > popup_state.last_day_of_month() {
                                            popup_state.day = 1;
                                            popup_state.month += 1;
                                            if popup_state.month > 12 {
                                                popup_state.month = 1;
                                                popup_state.year += 1;
                                            }
                                        }
                                        ui.data_mut(|data| {
                                            data.insert_persisted(id, popup_state.clone());
                                        });
                                    }
                                });
                            });
                            strip.cell(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                    if ui.button(">>").on_hover_text("add one month").clicked() {
                                        popup_state.month += 1;
                                        if popup_state.month > 12 {
                                            popup_state.month = 1;
                                            popup_state.year += 1;
                                        }
                                        popup_state.day =
                                            popup_state.day.min(popup_state.last_day_of_month());
                                        ui.data_mut(|data| {
                                            data.insert_persisted(id, popup_state.clone());
                                        });
                                    }
                                });
                            });
                            strip.cell(|ui| {
                                ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                    if ui.button(">>>").on_hover_text("add one year").clicked() {
                                        popup_state.year += 1;
                                        popup_state.day =
                                            popup_state.day.min(popup_state.last_day_of_month());
                                        ui.data_mut(|data| {
                                            data.insert_persisted(id, popup_state.clone());
                                        });
                                    }
                                });
                            });
                        });
                    });
                }

                if self.calendar {
                    strip.cell(|ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(1.0, 2.0);
                        TableBuilder::new(ui)
                            .vscroll(false)
                            .columns(Column::remainder(), if self.calendar_week { 8 } else { 7 })
                            .header(height, |mut header| {
                                if self.calendar_week {
                                    header.col(|ui| {
                                        ui.with_layout(
                                            Layout::centered_and_justified(Direction::TopDown),
                                            |ui| {
                                                ui.label("Week");
                                            },
                                        );
                                    });
                                }

                                //TODO(elwerene): Locale
                                for name in ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"] {
                                    header.col(|ui| {
                                        ui.with_layout(
                                            Layout::centered_and_justified(Direction::TopDown),
                                            |ui| {
                                                ui.label(name);
                                            },
                                        );
                                    });
                                }
                            })
                            .body(|mut body| {
                                for week in weeks {
                                    body.row(height, |mut row| {
                                        if self.calendar_week {
                                            row.col(|ui| {
                                                ui.label(week.number.to_string());
                                            });
                                        }
                                        for day in week.days {
                                            row.col(|ui| {
                                                ui.with_layout(
                                                    Layout::top_down_justified(Align::Center),
                                                    |ui| {
                                                        let fill_color = if popup_state.year
                                                            == day.year()
                                                            && popup_state.month == day.month()
                                                            && popup_state.day == day.day()
                                                        {
                                                            ui.visuals().selection.bg_fill
                                                        } else if (day.weekday() == Weekday::Sat
                                                            || day.weekday() == Weekday::Sun)
                                                            && self.highlight_weekends
                                                        {
                                                            if ui.visuals().dark_mode {
                                                                Color32::DARK_RED
                                                            } else {
                                                                Color32::LIGHT_RED
                                                            }
                                                        } else {
                                                            ui.visuals().extreme_bg_color
                                                        };

                                                        let mut text_color = ui
                                                            .visuals()
                                                            .widgets
                                                            .inactive
                                                            .text_color();

                                                        if day.month() != popup_state.month {
                                                            text_color =
                                                                text_color.linear_multiply(0.5);
                                                        };

                                                        let button_response = ui.add(
                                                            Button::new(
                                                                RichText::new(
                                                                    day.day().to_string(),
                                                                )
                                                                .color(text_color),
                                                            )
                                                            .fill(fill_color),
                                                        );

                                                        if day == today {
                                                            // Encircle today's date
                                                            let stroke = ui
                                                                .visuals()
                                                                .widgets
                                                                .inactive
                                                                .fg_stroke;
                                                            ui.painter().circle_stroke(
                                                                button_response.rect.center(),
                                                                8.0,
                                                                stroke,
                                                            );
                                                        }

                                                        if button_response.clicked() {
                                                            popup_state.year = day.year();
                                                            popup_state.month = day.month();
                                                            popup_state.day = day.day();
                                                            ui.data_mut(|data| {
                                                                data.insert_persisted(
                                                                    id,
                                                                    popup_state.clone(),
                                                                );
                                                            });
                                                        }
                                                    },
                                                );
                                            });
                                        }
                                    });
                                }
                            });
                    });
                }

                strip.strip(|builder| {
                    builder.sizes(Size::remainder(), 3).horizontal(|mut strip| {
                        strip.empty();
                        strip.cell(|ui| {
                            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                if ui.button("Cancel").clicked() {
                                    close = true;
                                }
                            });
                        });
                        strip.cell(|ui| {
                            ui.with_layout(Layout::top_down_justified(Align::Center), |ui| {
                                if ui.button("Save").clicked() {
                                    *self.selection = NaiveDate::from_ymd_opt(
                                        popup_state.year,
                                        popup_state.month,
                                        popup_state.day,
                                    )
                                    .expect("Could not create NaiveDate");
                                    saved = true;
                                    close = true;
                                }
                            });
                        });
                    });
                });
            });

        if close {
            popup_state.setup = false;
            ui.data_mut(|data| {
                data.insert_persisted(id, popup_state);
                data.get_persisted_mut_or_default::<DatePickerButtonState>(self.button_id)
                    .picker_visible = false;
            });
        }

        saved && close
    }
}

fn month_name(i: u32) -> &'static str {
    match i {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => panic!("Unknown month: {i}"),
    }
}
