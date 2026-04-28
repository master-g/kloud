//! Ratatui-based TUI backend.
//!
//! Runs a full-screen terminal UI with a messages area, input bar, and
//! status line. Uses `tokio::select!` to multiplex terminal events,
//! application events, and a render tick.

pub mod constants;
pub mod input;
pub mod state;
pub mod text_area;
pub mod theme;
pub mod virtual_scroll;
pub mod widgets;

use std::io;
use std::time::Duration;

use crossterm::event::EventStream;
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{event as ct_event, execute};
use futures::{StreamExt, stream};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tokio::time;
use tokio_stream::wrappers::ReceiverStream;

use crate::ui::tui::constants::TUI_FPS;

use self::state::{Screen, TuiState};
use super::backend::{UiBackend, UiChannels};
use super::events::AppEvent;
use crate::ui::events::UiAction;

/// A full-screen ratatui terminal UI.
pub struct RatatuiBackend {
    /// Model name shown in the status bar.
    pub model: String,
    /// Maximum context window tokens for the current model.
    pub max_context_tokens: u32,
    /// Workspace path shown in the dashboard.
    pub workspace: String,
    /// Current git branch shown in the dashboard.
    pub branch: String,
    /// Effort label shown in the dashboard.
    pub effort: String,
    /// Number of tools available in the current session.
    pub tool_count: usize,
    /// Instruction files discovered for the current workspace.
    pub instruction_files: Vec<String>,
    /// Number of active hooks in the current repository.
    pub hook_count: usize,
    /// Active color scheme for the TUI.
    pub theme: theme::Theme,
    /// Whether to render the one-line title bar.
    pub show_title_bar: bool,
}

/// RAII guard that restores the terminal on drop (even on panic).
struct TerminalGuard;

impl TerminalGuard {
    fn setup() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stderr(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stderr(), LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

#[async_trait::async_trait]
impl UiBackend for RatatuiBackend {
    async fn run(self, mut channels: UiChannels) -> crate::Result<()> {
        let Self {
            model,
            max_context_tokens,
            workspace,
            branch,
            effort,
            tool_count,
            instruction_files,
            hook_count,
            theme,
            show_title_bar,
        } = self;

        // Set up terminal with RAII cleanup guard
        let _guard = TerminalGuard::setup()?;
        let backend = CrosstermBackend::new(io::stderr());
        let mut terminal = Terminal::new(backend)?;

        let mut state = TuiState::new(
            model,
            max_context_tokens,
            workspace,
            branch,
            effort,
            tool_count,
            instruction_files,
            hook_count,
        );
        let mut event_stream = EventStream::new();
        let mut theme = theme;
        let mut tick = time::interval(Duration::from_millis(1000 / TUI_FPS));
        tick.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        // Convert optional key injection receiver into a boxed stream.
        let mut inject_stream: std::pin::Pin<
            Box<dyn futures::Stream<Item = ct_event::Event> + Send>,
        > = match channels.key_inject_rx.take() {
            Some(rx) => Box::pin(ReceiverStream::new(rx)),
            None => Box::pin(stream::pending()),
        };

        loop {
            tokio::select! {
                // (a) Terminal events (keyboard, resize)
                maybe_event = event_stream.next() => {
                    if let Some(Ok(event)) = maybe_event {
                        if let ct_event::Event::Resize(_, _) = &event {
                            // Just re-render on next tick
                        } else if let Some(action) = input::handle_event(&event, &mut state) {
                            match &action {
                                UiAction::SearchActivate => {
                                    state.app.screen = Screen::Search;
                                }
                                UiAction::SearchExit => {
                                    state.app.screen = Screen::Prompt;
                                }
                                _ => {}
                            }
                            let _ = channels.action_tx.send(action).await;
                        }
                    }
                }
                // (b) Application events (stream deltas, errors, shutdown)
                maybe_app = channels.event_rx.recv() => {
                    match maybe_app {
                        Some(AppEvent::View(view)) => {
                            state.apply_view(*view);
                        }
                        Some(AppEvent::ThemeChanged(name)) => {
                            if let Ok(scheme) = name.parse() {
                                theme = theme::Theme::from_scheme(scheme, theme.no_color);
                            }
                        }
                        Some(AppEvent::Shutdown) | None => {
                            state.should_quit = true;
                        }
                    }
                }
                // (c) Render tick
                _ = tick.tick() => {
                    state.activity_clock.try_tick();
                    let char_target = state.response_char_count.max(state.output_tokens as usize * 4);
                    state.token_counter.set_target(char_target);
                    state.token_counter.advance();
                    state.tick_thinking_status();
                    state.tick_activity_snapshot();
                    state.stalled_state.update(
                        state.response_char_count,
                        !state.app.active_tools.is_empty(),
                    );
                }
                // (d) Key injection (demo/test only)
                maybe_inject = inject_stream.next() => {
                    if let Some(event) = maybe_inject
                        && let Some(action) = input::handle_event(&event, &mut state)
                    {
                        match &action {
                            UiAction::SearchActivate => {
                                state.app.screen = Screen::Search;
                            }
                            UiAction::SearchExit => {
                                state.app.screen = Screen::Prompt;
                            }
                            _ => {}
                        }
                        let _ = channels.action_tx.send(action).await;
                    }
                }
            }

            // Draw
            terminal.draw(|frame| widgets::render(frame, &mut state, &theme, show_title_bar))?;

            if state.should_quit {
                break;
            }
        }

        Ok(())
    }
}
