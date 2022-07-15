use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Modifier;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::{Block, BorderType, Borders};
use tui::Frame;
use tui::Terminal;

use crate::history::ActiveWidget;
use crate::history::History;

pub fn draw(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, history: &mut History) {
    terminal.draw(|rect| ui_draw(rect, history)).unwrap();
}

fn ui_draw<B>(rect: &mut Frame<B>, history: &History)
where
    B: Backend,
{
    let size = rect.size();

    let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(2)
    .constraints(
        [
            Constraint::Min(4),
            Constraint::Length(1),
        ]
        .as_ref(),
    )
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
