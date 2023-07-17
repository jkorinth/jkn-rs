use crossterm::{event, event::KeyCode, execute, terminal::*};
use log::debug;
use std::{cell::RefCell, io, time::Duration};
use termimad::{Area, MadSkin, MadView};
use tui::{
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::*,
    text::*,
    widgets::*,
    Frame, Terminal,
};

struct UserInterface<'a> {
    notes: Vec<String>,
    layout: Layout,
    content: Block<'a>,
    notelist: List<'a>,
    notelist_state: ListState,
    mv: MadView,
}

impl Default for UserInterface<'_> {
    fn default() -> Self {
        let notes: Vec<String> = vec![];
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let notes_block = Block::default().title(" Notes ").borders(Borders::ALL);
        let notelist = List::new(
            notes
                .iter()
                .map(|s| ListItem::new(Text::raw(s.clone())))
                .collect::<Vec<_>>(),
        )
        .block(notes_block)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
        .highlight_symbol(">> ");
        let content = Block::default().title(" Content ").borders(Borders::ALL);
        let mut state = ListState::default();
        let mdstr = String::from("**Hello**\nlet's see *some* Markdown\nOi!\n\n## Bla\nfeck");
        let mv = MadView::from(mdstr, Area::new(0, 0, 0, 0), MadSkin::default());

        Self {
            notes: notes,
            layout: layout,
            notelist: notelist,
            notelist_state: state,
            content: content,
            mv: mv,
        }
    }
}

impl UserInterface<'_> {
    pub fn render<B: Backend>(&mut self, f: &mut Frame<B>) {
        let cs = self.layout.split(f.size());
        f.render_stateful_widget(self.notelist.clone(), cs[0], &mut self.notelist_state);
        f.render_widget(self.content.clone(), cs[1]);
        let r = self.content.inner(cs[1]);
        let a = Area::new(r.x + 1, r.y + 1, r.width - 2, /*r.height - */ 2);
        self.mv.resize(&a);
        self.mv.write();
    }

    pub fn handle_keypress(&mut self, ev: &event::KeyEvent) -> bool {
        match ev.code {
            KeyCode::Down => self.mv.try_scroll_lines(1),
            KeyCode::Up => self.mv.try_scroll_lines(-1),
            KeyCode::PageDown => self.mv.try_scroll_pages(1),
            KeyCode::PageUp => self.mv.try_scroll_pages(-1),
            KeyCode::Char('w') => self.previous(),
            KeyCode::Char('s') => self.next(),
            KeyCode::Char('q') => return true,
            _ => return false,
        }
        false
    }

    fn update_content(&mut self, idx: Option<usize>) {
        if let Some(n) = idx {
            self.mv = MadView::from(
                format!("**Now Showing**\n\nNote *#{}*", n),
                Area::new(0, 0, 0, 0),
                MadSkin::default(),
            );
        }
    }

    fn next(&mut self) {
        if let Some(idx) = self.notelist_state.selected() {
            self.notelist_state
                .select(Some((idx + 1) % self.notes.len()));
            self.update_content(self.notelist_state.selected());
        }
    }

    fn previous(&mut self) {
        if let Some(idx) = self.notelist_state.selected() {
            self.notelist_state.select(if idx == 0 {
                Some(self.notes.len() - 1)
            } else {
                Some(idx - 1)
            });
            self.update_content(self.notelist_state.selected());
        }
    }
}

fn main() -> Result<(), io::Error> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    let mut ui = UserInterface::default();

    loop {
        terminal.draw(|f| ui.render(f))?;
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                event::Event::Key(ev) => {
                    if ui.handle_keypress(&ev) {
                        break;
                    }
                }
                _ => todo! {},
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
