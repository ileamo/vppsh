use tui::backend::Backend;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::Modifier;
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::List;
use tui::widgets::ListItem;
use tui::widgets::ListState;
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;
use tui::Terminal;

use crate::history::History;

pub fn draw(terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>, history: &mut History) {
    terminal.draw(|rect| ui_draw(rect, history)).unwrap();
}

fn ui_draw<B>(rect: &mut Frame<B>, history: &History)
where
    B: Backend,
{
    let size = rect.size();

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    let mut list_state = ListState::default();
    list_state.select(Some(history.get_selected()));

    let history_widget = draw_history(history);
    rect.render_stateful_widget(history_widget, body_chunks[0], &mut list_state);

    let conf_widget = draw_conf(history);
    rect.render_widget(conf_widget, body_chunks[1]);
}

fn draw_history<'a>(history: &'a History) -> List<'a> {
    let items: Vec<_> = history
        .list
        .iter()
        .map(|cmd| ListItem::new(Spans::from(vec![Span::styled(cmd, Style::default())])))
        .collect();
    List::new(items)
        .block(
            Block::default()
                .title("History")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Yellow)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
}

fn draw_conf<'a>(_history: &'a History) -> Paragraph<'a> {
    Paragraph::new(vec![Spans::from(Span::raw("text"))])
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Configuration")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}
