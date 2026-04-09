use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    widgets::Widget,
};
use vt100::{Cell, Color as VtColor, Screen};

pub struct PtyView;

#[derive(Clone, Debug, Default)]
pub struct PtyRenderFilter {
    hidden_commentary: Vec<String>,
}

impl PtyRenderFilter {
    pub fn new<I, S>(hidden_commentary: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            hidden_commentary: hidden_commentary
                .into_iter()
                .map(|line| line.as_ref().trim().to_string())
                .filter(|line| !line.is_empty())
                .collect(),
        }
    }
}

pub struct PtyScreenWidget<'a> {
    screen: &'a Screen,
    filter: PtyRenderFilter,
}

impl Default for PtyView {
    fn default() -> Self {
        Self::new()
    }
}

impl PtyView {
    pub fn new() -> Self {
        Self
    }

    pub fn render<'a>(&self, screen: &'a Screen, filter: PtyRenderFilter) -> PtyScreenWidget<'a> {
        PtyScreenWidget { screen, filter }
    }
}

impl Widget for PtyScreenWidget<'_> {
    fn render(self, area: Rect, buffer: &mut Buffer) {
        for y in 0..area.height {
            for x in 0..area.width {
                buffer[(area.x + x, area.y + y)].reset();
            }
        }

        let (rows, cols) = self.screen.size();
        let cols = cols.min(area.width);
        let plain_rows = self.screen.rows(0, cols).collect::<Vec<_>>();
        let mut target_row = 0_u16;

        for row in 0..rows {
            if target_row >= area.height {
                break;
            }
            if plain_rows
                .get(usize::from(row))
                .is_some_and(|line| should_filter_row(line, &self.filter))
            {
                continue;
            }

            for col in 0..cols {
                if let Some(cell) = self.screen.cell(row, col) {
                    render_cell(buffer, area, target_row, col, cell);
                }
            }
            target_row = target_row.saturating_add(1);
        }
    }
}

fn should_filter_row(row: &str, filter: &PtyRenderFilter) -> bool {
    let trimmed = row.trim();
    if trimmed.is_empty() {
        return false;
    }

    let normalized = strip_codex_prefix(trimmed);
    if normalized.is_empty() {
        return false;
    }

    if is_hook_status(normalized) || is_activity_summary(trimmed, normalized) {
        return true;
    }

    normalized.len() >= 12
        && filter.hidden_commentary.iter().any(|message| {
            let message = message.trim();
            message.contains(normalized) || normalized.contains(message)
        })
}

fn strip_codex_prefix(row: &str) -> &str {
    row.trim()
        .strip_prefix('•')
        .map(str::trim_start)
        .or_else(|| row.trim().strip_prefix('└').map(str::trim_start))
        .or_else(|| row.trim().strip_prefix('│').map(str::trim_start))
        .unwrap_or_else(|| row.trim())
}

fn is_hook_status(row: &str) -> bool {
    (row.starts_with("Running ") && row.ends_with(" hook"))
        || row.ends_with(" hook (completed)")
        || row.ends_with(" hook (failed)")
}

fn is_activity_summary(original: &str, normalized: &str) -> bool {
    if !matches!(original.trim().chars().next(), Some('•' | '└' | '│')) {
        return false;
    }

    [
        "Starting ",
        "Explored",
        "Read ",
        "Opened ",
        "Viewed ",
        "Checked ",
        "Searched ",
        "Edited ",
        "Applied ",
        "Updated ",
        "Listed ",
    ]
    .iter()
    .any(|prefix| normalized.starts_with(prefix))
}

fn render_cell(buffer: &mut Buffer, area: Rect, row: u16, col: u16, cell: &Cell) {
    let (fg, bg) = resolved_colors(cell);

    let mut modifier = Modifier::empty();
    if cell.bold() {
        modifier |= Modifier::BOLD;
    }
    if cell.italic() {
        modifier |= Modifier::ITALIC;
    }
    if cell.underline() {
        modifier |= Modifier::UNDERLINED;
    }

    let symbol = if cell.is_wide_continuation() {
        " ".to_string()
    } else if cell.has_contents() {
        cell.contents()
    } else {
        " ".to_string()
    };

    let buffer_cell = &mut buffer[(area.x + col, area.y + row)];
    buffer_cell.set_symbol(&symbol);
    buffer_cell.fg = fg;
    buffer_cell.bg = bg;
    buffer_cell.modifier = modifier;
}

fn resolved_colors(cell: &Cell) -> (Color, Color) {
    let mut fg = map_color(cell.fgcolor());
    let mut bg = map_color(cell.bgcolor());

    if cell.inverse() {
        std::mem::swap(&mut fg, &mut bg);
    }

    (fg, bg)
}

fn map_color(color: VtColor) -> Color {
    match color {
        VtColor::Default => Color::Reset,
        VtColor::Idx(0) => Color::Black,
        VtColor::Idx(1) => Color::Red,
        VtColor::Idx(2) => Color::Green,
        VtColor::Idx(3) => Color::Yellow,
        VtColor::Idx(4) => Color::Blue,
        VtColor::Idx(5) => Color::Magenta,
        VtColor::Idx(6) => Color::Cyan,
        VtColor::Idx(7) => Color::Gray,
        VtColor::Idx(8) => Color::DarkGray,
        VtColor::Idx(9) => Color::LightRed,
        VtColor::Idx(10) => Color::LightGreen,
        VtColor::Idx(11) => Color::LightYellow,
        VtColor::Idx(12) => Color::LightBlue,
        VtColor::Idx(13) => Color::LightMagenta,
        VtColor::Idx(14) => Color::LightCyan,
        VtColor::Idx(15) => Color::White,
        VtColor::Idx(index) => Color::Indexed(index),
        VtColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
    }
}
