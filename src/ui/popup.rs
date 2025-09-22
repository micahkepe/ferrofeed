use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
};
/// Represents potential popups
#[derive(Debug)]
pub enum PopupState {
    /// The help menu popup.
    Help,
}

/// Helper function to create a centered rect using up certain percentage of the available rect
/// `r`.
///
/// Taken from: https://ratatui.rs/examples/apps/popup/
pub(crate) fn get_centered_popup_area(area: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let vertical = Layout::vertical([Constraint::Percentage(percent_y)]).flex(Flex::Center);
    let horizontal = Layout::horizontal([Constraint::Percentage(percent_x)]).flex(Flex::Center);
    let [area] = vertical.areas(area);
    let [area] = horizontal.areas(area);
    area
}

/// Helper function to pad top lines with empty lines to center them in a rect.
pub(crate) fn pad_top_lines_center(
    lines: Vec<Line<'_>>,
    area: Rect,
    bordered: bool,
) -> Vec<Line<'_>> {
    let total_lines = lines.len();
    let available_height = if bordered {
        // minus 2 for top and bottom borders
        area.height.saturating_sub(2)
    } else {
        area.height
    };
    let top_padding = (available_height.saturating_sub(total_lines as u16)) / 2;
    let mut padded_lines = Vec::new();
    padded_lines.extend(std::iter::repeat_n(Line::from(""), top_padding as usize));
    padded_lines.extend(lines.clone());
    padded_lines
}
