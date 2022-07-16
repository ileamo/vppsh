use std::cmp::max;

use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::layout::Alignment;
use tui::layout::Rect;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Modifier;
use tui::style::{Color, Style};
use tui::text::Span;
use tui::text::Spans;
use tui::widgets::Clear;
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::Paragraph;
use tui::widgets::{Block, BorderType, Borders};
use tui::Frame;
use tui::Terminal;

use crate::history::ActiveWidget;
use crate::history::History;

pub fn draw(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, history: &mut History) {
    terminal.draw(|rect| ui_draw(rect, history)).unwrap();
}

fn ui_draw<B>(rect: &mut Frame<B>, history: &mut History)
where
    B: Backend,
{
    let size = rect.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(1)].as_ref())
        .split(size);

    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let mut list_state = ListState::default();
    list_state.select(Some(history.get_hist_selected()));

    let history_widget = draw_hist(history);
    rect.render_stateful_widget(history_widget, body_chunks[0], &mut list_state);

    list_state.select(Some(history.get_conf_selected()));
    let conf_widget = draw_conf(history);
    rect.render_stateful_widget(conf_widget, body_chunks[1], &mut list_state);

    let help_widget = draw_help();
    rect.render_widget(help_widget, chunks[1]);

    if let Some(text_info) = history.get_info_text()  {
        let (info, area) = centered_rect(size, text_info);
        rect.render_widget(Clear, area); //this clears out the background
        rect.render_widget(info, area);
        history.clear_info_text();
    }
}

fn draw_hist<'a>(history: &'a History) -> List<'a> {
    let mut border_modifier = Modifier::DIM;
    let mut hl_bg = Color::DarkGray;
    if let ActiveWidget::Hist = history.get_active_widget() {
        border_modifier = Modifier::empty();
        hl_bg = Color::LightBlue;
    };

    let items: Vec<_> = history
        .hist
        .iter()
        .map(|cmd| ListItem::new(Spans::from(vec![Span::styled(cmd, Style::default())])))
        .collect();
    List::new(items)
        .block(
            Block::default()
                .title(tr!("History"))
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain)
                .border_style(Style::default().add_modifier(border_modifier)),
        )
        .highlight_style(
            Style::default()
                .bg(hl_bg)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
}

fn draw_conf<'a>(history: &'a History) -> List<'a> {
    let mut border_modifier = Modifier::DIM;
    let mut active = false;
    if let ActiveWidget::Conf = history.get_active_widget() {
        border_modifier = Modifier::empty();
        active = true;
    };

    let items: Vec<_> = history
        .conf
        .iter()
        .map(|cmd| ListItem::new(Spans::from(vec![Span::styled(cmd, Style::default())])))
        .collect();
    let mut list = List::new(items).block(
        Block::default()
            .title(tr!("Configuration"))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain)
            .border_style(Style::default().add_modifier(border_modifier)),
    );

    if active {
        list = list.highlight_style(
            Style::default()
                .bg(Color::LightBlue)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
    };
    list
}

fn draw_help<'a>() -> Paragraph<'a> {
    let style = Style::default().bg(Color::LightBlue).fg(Color::Black);
    let help_text = Spans::from(vec![
        Span::styled("Home [h]", style),
        Span::raw(" "),
        Span::styled("vppctl [i]", style),
        Span::raw(" "),
        Span::styled("Copy [\u{2192}]", style),
        Span::raw(" "),
        Span::styled("Toggle [TAB]", style),
        Span::raw(" "),
        Span::styled("Scroll [\u{2191}\u{2193}]", style),
        Span::raw(" "),
        Span::styled("Quit [q]", style),
        Span::raw(" "),
        Span::styled("Save [s]", style),
    ]);
    Paragraph::new(help_text).alignment(Alignment::Left)
}

fn centered_rect(r: Rect, info_text: &str) -> (Paragraph, Rect) {
    let info = Paragraph::new(info_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Info")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        );

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length((r.height - 3 - 1) / 2),
                Constraint::Length(3),
                Constraint::Min(0),
            ]
            .as_ref(),
        )
        .split(r);

    let len = max((info_text.len() + 2 + 2) as u16, r.width / 2);
    let area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length((r.width - len - 2) / 2),
                Constraint::Min(0),
                Constraint::Length((r.width - len - 2) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1];

    (info, area)
}
