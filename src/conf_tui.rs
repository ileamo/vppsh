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

pub struct ConfTui {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl ConfTui {
    pub fn new() -> ConfTui {
        let backend = CrosstermBackend::new(std::io::stdout());
        let terminal = Terminal::new(backend).unwrap();
        ConfTui { terminal }
    }

    pub fn draw(&mut self, history: &Vec<String>) {
        self.terminal.clear().unwrap();
        self.terminal.hide_cursor().unwrap();
        self.terminal.draw(|rect| ui_draw(rect, history)).unwrap();
    }
}

fn ui_draw<B>(rect: &mut Frame<B>, history: &Vec<String>)
where
    B: Backend,
{
    let size = rect.size();

    // Body & Help
    let body_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(size);

    let mut pet_list_state = ListState::default();
    pet_list_state.select(Some(1));

    let history = draw_history(history);
    rect.render_stateful_widget(history, body_chunks[0], &mut pet_list_state);

    let help = draw_conf();
    rect.render_widget(help, body_chunks[1]);
}

fn draw_history<'a>(history: &'a Vec<String>) -> List<'a> {
    let items: Vec<_> = history
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

fn draw_conf<'a>() -> Paragraph<'a> {
    Paragraph::new(vec![Spans::from(Span::raw("text"))])
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                // .title("Body")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .border_type(BorderType::Plain),
        )
}
