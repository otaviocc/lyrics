// Copyright (c) 2026 Otávio C.
// SPDX-License-Identifier: MIT

//! The `--folder` song list: the cursor over it, and the overlay's geometry.

use std::ops::Range;

use ratatui::layout::Size;

use crate::tui::app::Song;
use crate::tui::clamp_fraction;

pub const TITLE: &str = " Lyrics ";
const LABEL_SEPARATOR: &str = " — ";
const PLAYING_MARKER: &str = "▸ ";
const IDLE_MARKER: &str = "  ";
const WIDTH_FRACTION: u16 = 70;
const MIN_WIDTH: u16 = 24;
const MAX_WIDTH: u16 = 72;
const HEIGHT_FRACTION: u16 = 80;
const MIN_HEIGHT: u16 = 5;
const MAX_HEIGHT: u16 = 26;
const BORDER_ROWS: u16 = 2;

#[must_use]
pub fn outer(area: Size, rows: usize) -> Size {
    let width = clamp_fraction(area.width, WIDTH_FRACTION, MIN_WIDTH, MAX_WIDTH);
    let ceiling = clamp_fraction(area.height, HEIGHT_FRACTION, MIN_HEIGHT, MAX_HEIGHT);
    let wanted = u16::try_from(rows)
        .unwrap_or(u16::MAX)
        .saturating_add(BORDER_ROWS);
    Size::new(width, wanted.min(ceiling))
}

#[derive(Debug, Clone)]
pub struct Picker {
    songs: Vec<Song>,
    selected: usize,
    chosen: Option<usize>,
}

impl Picker {
    #[must_use]
    pub const fn new(songs: Vec<Song>) -> Self {
        Self {
            songs,
            selected: 0,
            chosen: None,
        }
    }

    #[must_use]
    pub const fn len(&self) -> usize {
        self.songs.len()
    }

    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.songs.is_empty()
    }

    #[must_use]
    pub const fn selected(&self) -> usize {
        self.selected
    }

    #[must_use]
    pub fn current(&self) -> Option<&Song> {
        self.songs.get(self.selected)
    }

    #[must_use]
    pub const fn chosen(&self) -> Option<usize> {
        self.chosen
    }

    pub fn choose(&mut self) -> Option<&Song> {
        let song = self.songs.get(self.selected)?;
        self.chosen = Some(self.selected);
        Some(song)
    }

    pub const fn rewind(&mut self) {
        if let Some(chosen) = self.chosen {
            self.selected = chosen;
        }
    }

    pub const fn select_next(&mut self) {
        let last = self.songs.len().saturating_sub(1);
        if self.selected < last {
            self.selected = self.selected.saturating_add(1);
        }
    }

    pub const fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    #[must_use]
    pub fn label(&self, index: usize) -> Option<String> {
        let song = self.songs.get(index)?;
        let marker = if self.chosen == Some(index) {
            PLAYING_MARKER
        } else {
            IDLE_MARKER
        };
        let body = song.artist.as_ref().map_or_else(
            || song.title.clone(),
            |artist| format!("{artist}{LABEL_SEPARATOR}{}", song.title),
        );
        Some(format!("{marker}{body}"))
    }

    #[must_use]
    pub fn window(&self, height: usize) -> Range<usize> {
        let total = self.songs.len();
        let start = self
            .selected
            .saturating_sub(height.saturating_div(2))
            .min(total.saturating_sub(height));
        start..start.saturating_add(height).min(total)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_artist_reaches_the_label_and_a_missing_row_has_none() {
        let picker = Picker::new(vec![
            Song {
                title: String::from("01 First"),
                artist: None,
                lines: Vec::new(),
            },
            Song {
                title: String::from("Second"),
                artist: Some(String::from("A Band")),
                lines: Vec::new(),
            },
        ]);

        assert_eq!(picker.label(0).unwrap(), "  01 First");
        assert_eq!(picker.label(1).unwrap(), "  A Band — Second");
        assert_eq!(picker.label(2), None);
    }

    fn picker_of(count: usize) -> Picker {
        let songs = (0..count)
            .map(|index| Song {
                title: format!("Song {index}"),
                artist: None,
                lines: Vec::new(),
            })
            .collect();
        Picker::new(songs)
    }

    #[test]
    fn the_cursor_clamps_at_both_ends_rather_than_wrapping() {
        let mut picker = picker_of(3);

        picker.select_previous();
        assert_eq!(picker.selected(), 0, "up from the top stays at the top");

        picker.select_next();
        picker.select_next();
        picker.select_next();
        picker.select_next();
        assert_eq!(picker.selected(), 2, "down from the bottom stays put");
    }

    #[test]
    fn the_cursor_does_not_move_in_an_empty_list() {
        let mut picker = picker_of(0);
        picker.select_next();
        picker.select_previous();
        assert_eq!(picker.selected(), 0);
        assert!(picker.current().is_none());
    }

    #[test]
    fn choosing_records_the_row_and_marks_it_in_the_label() {
        let mut picker = picker_of(3);
        picker.select_next();

        assert_eq!(picker.chosen(), None);
        assert_eq!(picker.choose().unwrap().title, "Song 1");
        assert_eq!(picker.chosen(), Some(1));
        assert_eq!(picker.label(1).unwrap(), "▸ Song 1");
        assert_eq!(picker.label(0).unwrap(), "  Song 0");
    }

    #[test]
    fn rewind_returns_the_cursor_to_the_song_being_followed() {
        let mut picker = picker_of(5);
        picker.select_next();
        picker.choose();
        picker.select_next();
        picker.select_next();
        assert_eq!(picker.selected(), 3);

        picker.rewind();
        assert_eq!(picker.selected(), 1);
    }

    #[test]
    fn rewind_is_a_no_op_before_anything_has_been_chosen() {
        let mut picker = picker_of(5);
        picker.select_next();
        picker.rewind();
        assert_eq!(picker.selected(), 1);
    }

    #[test]
    fn the_whole_list_is_visible_when_it_fits() {
        let picker = picker_of(4);
        assert_eq!(picker.window(10), 0..4);
    }

    #[test]
    fn the_window_scrolls_to_keep_the_cursor_visible() {
        let mut picker = picker_of(20);
        assert_eq!(picker.window(5), 0..5);

        for _ in 0..10 {
            picker.select_next();
        }
        let window = picker.window(5);
        assert!(
            window.contains(&picker.selected()),
            "{window:?} lost the cursor at {}",
            picker.selected()
        );

        for _ in 0..20 {
            picker.select_next();
        }
        let window = picker.window(5);
        assert_eq!(window, 15..20, "the last row sits at the bottom edge");
    }

    #[test]
    fn a_zero_height_window_is_empty_rather_than_a_panic() {
        let picker = picker_of(20);
        assert_eq!(picker.window(0), 0..0);
        assert_eq!(picker_of(0).window(5), 0..0);
    }

    #[test]
    fn the_box_never_outgrows_the_area_it_floats_over() {
        for width in 1u16..200 {
            for height in 1u16..40 {
                for rows in [0usize, 1, 7, 200] {
                    let size = outer(Size::new(width, height), rows);
                    assert!(size.width <= width, "{width}x{height} rows={rows}");
                    assert!(size.height <= height, "{width}x{height} rows={rows}");
                }
            }
        }
    }

    #[test]
    fn the_box_shrinks_to_a_short_list_instead_of_filling_the_screen() {
        let short = outer(Size::new(80, 40), 3);
        assert_eq!(short.height, 5, "three rows plus the border");

        let long = outer(Size::new(80, 40), 100);
        assert_eq!(long.height, MAX_HEIGHT);
    }
}
