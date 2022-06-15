mod storage;

use std::{io, time::{Duration, Instant}};
use tui::{
    backend::{CrosstermBackend, Backend},
    // widgets::{Widget, Block, Borders, Paragraph, Wrap, Table, Row},
    layout::{Rect/*, Alignment*/},
    // layout::{Layout, Constraint, Direction, Rect},
    Terminal,
    // text,
    Frame,
    self
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use hudsucker::HttpContext;
use tokio::sync::mpsc::Receiver;
use crate::cruster_handler::request_response::CrusterWrapper;
use crate::ui::storage::HTTPStorage;

// https://docs.rs/tui/latest/tui/widgets/index.html

pub(crate) async fn render(ui_rx: Receiver<(CrusterWrapper, HttpContext)>) -> Result<(), io::Error> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let tick_rate = Duration::from_millis(250);
    let mut terminal = Terminal::new(backend)?;

    run_app(&mut terminal, tick_rate, ui_rx).await?;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    tick_rate: Duration,
    mut ui_rx: Receiver<(CrusterWrapper, HttpContext)>
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut ui_storage = storage::UI::new();
    let mut something_changed = true;
    let mut http_storage = storage::HTTPStorage::default();
    loop {
        match ui_rx.try_recv() {
            Ok((wrapper, ctx)) => {
                let string_reference = ctx.client_addr;
                match wrapper {
                    CrusterWrapper::Request(request) => http_storage.put_request(request, string_reference),
                    CrusterWrapper::Response(response) => http_storage.put_response(response, &string_reference)
                }
                something_changed = true;
            },
            Err(_) => {
                // something_changed = true;
            }
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    return Ok(());
                }
                else if let KeyCode::Char('w') = key.code {
                    // something_changed = true;
                    // let test_paragraph = Paragraph::new("Test Hello");
                    // ui_storage.add_paragraph(test_paragraph, 0);
                }
                else if let KeyCode::Up = key.code {
                    let index = match ui_storage.proxy_history_state.selected() {
                        Some(i) => if i == 0 { 0 } else { i - 1 },
                        None => 0 as usize
                    };
                    ui_storage.proxy_history_state.select(Some(index));
                    something_changed = true;
                }
                else if let KeyCode::Down = key.code {
                    let index = match ui_storage.proxy_history_state.selected() {
                        Some(i) => if i == http_storage.len() - 1 { http_storage.len() - 1 } else { i + 1 },
                        None => 0 as usize
                    };
                    ui_storage.proxy_history_state.select(Some(index));
                    something_changed = true;
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }

        if something_changed {
            terminal.draw(|f| new_ui(f, &mut ui_storage, &http_storage))?;
            something_changed = false;
        }
    }
}

fn new_ui<B: Backend>(f: &mut Frame<B>, uis: &mut storage::UI<'static>, http_storage: &HTTPStorage) {
    let window_width = f.size().width;
    let window_height = f.size().height;
    uis.make_table_widths(window_width, http_storage);

    // 0 - Rect for requests log,
    // 1 - Rect for requests
    // 2 - Rect for responses
    let rects: [Rect; 3] = [
        Rect::new(
            f.size().x,
            f.size().y,
            window_width,
            window_height / 2
        ),
        Rect::new(
            f.size().x,
            f.size().y + window_height / 2,
            window_width / 2,
            window_height / 2)
        ,
        Rect::new(
            f.size().x + window_width / 2,
            f.size().y + window_height / 2,
            window_width / 2,
            window_height / 2
        )
    ];

    for ruint in uis.widgets.iter() {
        match ruint {
            storage::RenderUnit::TUIBlock((block, area_index)) => {
                let new_block = block.clone();
                let index = area_index.clone();
                f.render_widget(new_block, rects[index]);
            },
            storage::RenderUnit::TUIParagraph((paragraph, area_index)) => {
                let new_paragraph = paragraph.clone();
                let index = area_index.clone();
                f.render_widget(new_paragraph, rects[index]);
            },
            storage::RenderUnit::TUITable((table, area_index)) => {
                let new_table = table.clone();
                let index = area_index.clone();
                f.render_stateful_widget(new_table, rects[index], &mut uis.proxy_history_state);
            }
        }
    }
}