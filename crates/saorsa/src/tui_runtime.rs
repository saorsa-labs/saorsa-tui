//! saorsa TUI built on saorsa-tui retained runtime.

use anyhow::Context;

use saorsa_tui::Color;
use saorsa_tui::app::{App, Dom, Leaf, StyledInteractive, StyledLeaf};
use saorsa_tui::color::NamedColor;
use saorsa_tui::event::{Event, KeyCode, KeyEvent, Modifiers};
use saorsa_tui::segment::Segment;
use saorsa_tui::style::Style;
use saorsa_tui::terminal::Terminal;
use saorsa_tui::widget::EventResult;
use saorsa_tui::widget::{BorderStyle, Container, Label, RichLog};

use crate::app::{AppState, AppStatus, ChatRole};

/// Retained UI wrapper for the `saorsa` application.
pub struct SaorsaUi {
    app: App,
    header: saorsa_tui::app::NodeRef,
    messages: saorsa_tui::app::NodeRef,
    input_box: saorsa_tui::app::NodeRef,
    input_text: saorsa_tui::app::NodeRef,
    last_sig: UiSignature,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct UiSignature {
    model: String,
    status: String,
    input: String,
    messages_len: usize,
    streaming_len: usize,
}

impl SaorsaUi {
    /// Construct the UI tree and initialize the framework runtime.
    pub fn new(terminal: &dyn Terminal) -> anyhow::Result<Self> {
        struct BuiltDom {
            dom: Dom,
            root: saorsa_tui::app::NodeRef,
            header: saorsa_tui::app::NodeRef,
            messages: saorsa_tui::app::NodeRef,
            input_box: saorsa_tui::app::NodeRef,
            input_text: saorsa_tui::app::NodeRef,
        }

        fn build_dom() -> BuiltDom {
            let mut dom = Dom::new();

            // Root node: layout-only (renders nothing).
            let root = dom.create(
                "Root",
                Box::new(Leaf::new(saorsa_tui::StaticWidget::new(vec![]))),
            );
            dom.set_root(root);
            dom.set_css_id(root, "root");

            // Header.
            let header = dom.create("Label", Box::new(StyledLeaf::new(Label::new(""))));
            dom.set_css_id(header, "header");

            // Messages.
            let messages = dom.create(
                "RichLog",
                Box::new(StyledInteractive::new(
                    RichLog::new().with_border(BorderStyle::None),
                )),
            );
            dom.set_css_id(messages, "messages");
            dom.set_focusable(messages, true);

            // Input box + input text label.
            let input_box = dom.create(
                "Container",
                Box::new(StyledLeaf::new(
                    Container::new()
                        .border(BorderStyle::Rounded)
                        .title("Type a message"),
                )),
            );
            dom.set_css_id(input_box, "input_box");

            let input_text = dom.create("Label", Box::new(StyledLeaf::new(Label::new(""))));
            dom.set_css_id(input_text, "input_text");

            BuiltDom {
                dom,
                root,
                header,
                messages,
                input_box,
                input_text,
            }
        }

        let default_tcss = include_str!("../saorsa.tcss");

        // Prefer file-based TCSS so we can hot reload (edit `crates/saorsa/saorsa.tcss`).
        let tcss_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("saorsa.tcss");
        let BuiltDom {
            dom,
            root,
            header,
            messages,
            input_box,
            input_text,
        } = build_dom();

        let (mut app, root, header, messages, input_box, input_text) =
            match App::from_tcss_file(terminal, dom, &tcss_path) {
                Ok(app) => (app, root, header, messages, input_box, input_text),
                Err(e) => {
                    // Fall back to embedded string if file watch is unavailable.
                    let BuiltDom {
                        dom,
                        root,
                        header,
                        messages,
                        input_box,
                        input_text,
                    } = build_dom();
                    let app = App::from_tcss_string(terminal, dom, default_tcss)
                        .map_err(|e2| anyhow::anyhow!("{e2}"))
                        .context(format!(
                            "Failed to initialize saorsa-tui App runtime (file={:?}, err={e})",
                            tcss_path
                        ))?;
                    (app, root, header, messages, input_box, input_text)
                }
            };

        // Now mount nodes using the runtime API (exercises lifecycle/layout updates).
        app.mount(root, header)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        app.mount(root, messages)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        app.mount(root, input_box)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        app.mount(input_box, input_text)
            .map_err(|e| anyhow::anyhow!("{e}"))?;

        // Manual test hook: Ctrl+R forces a render.
        app.register_action(
            "force_render",
            Box::new(|app| {
                app.request_render();
                Ok(EventResult::Consumed)
            }),
        );
        app.bind_key(
            KeyEvent::new(KeyCode::Char('r'), Modifiers::CTRL),
            "force_render",
        );

        Ok(Self {
            app,
            header,
            messages,
            input_box,
            input_text,
            last_sig: UiSignature::default(),
        })
    }

    /// Notify the runtime that the terminal size changed.
    pub fn handle_resize(&mut self, w: u16, h: u16) {
        self.app.handle_resize(saorsa_tui::Size::new(w, h));
    }

    /// Forward an event to the runtime (focus, resize, etc).
    pub fn handle_event(&mut self, event: &Event) -> anyhow::Result<()> {
        let _ = self
            .app
            .handle_event(event)
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    /// Poll for stylesheet reload events (if the runtime was created from a file).
    pub fn poll_stylesheet_reload(&mut self) -> anyhow::Result<()> {
        let _ = self
            .app
            .poll_stylesheet_reload()
            .map_err(|e| anyhow::anyhow!("{e}"))?;
        Ok(())
    }

    /// Update widget content from application state.
    pub fn sync_from_state(&mut self, state: &AppState) -> anyhow::Result<()> {
        let status_text = match &state.status {
            AppStatus::Idle => "Ready".to_string(),
            AppStatus::Thinking => "Thinking...".to_string(),
            AppStatus::ToolRunning { tool_name } => tool_name.clone(),
        };

        let sig = UiSignature {
            model: state.model.clone(),
            status: status_text.clone(),
            input: state.input.clone(),
            messages_len: state.messages.len(),
            streaming_len: state.streaming_text.len(),
        };

        if sig == self.last_sig {
            return Ok(());
        }

        // Header.
        if let Some(label) = self.app.dom_mut().downcast_widget_mut::<Label>(self.header) {
            label.set_text(format!(" saorsa | {} | {}", state.model, status_text));
        }

        // Input box title + input text.
        if let Some(container) = self
            .app
            .dom_mut()
            .downcast_widget_mut::<Container>(self.input_box)
        {
            if state.is_idle() {
                container.set_title_text("Type a message");
            } else {
                container.set_title_text("Waiting...");
            }
        }
        if let Some(label) = self
            .app
            .dom_mut()
            .downcast_widget_mut::<Label>(self.input_text)
        {
            label.set_text(state.input.clone());
        }

        // Messages: rebuild entries. This is simple but not incremental yet.
        if let Some(log) = self
            .app
            .dom_mut()
            .downcast_widget_mut::<RichLog>(self.messages)
        {
            log.clear();

            for msg in &state.messages {
                let (prefix, style) = match &msg.role {
                    ChatRole::User => (
                        "> ",
                        Style::default()
                            .fg(Color::Named(NamedColor::Green))
                            .bold(true),
                    ),
                    ChatRole::Assistant => {
                        ("  ", Style::default().fg(Color::Named(NamedColor::Cyan)))
                    }
                    ChatRole::Tool { name } => {
                        let prefix = format!("  [{name}] ");
                        let style = Style::default()
                            .fg(Color::Named(NamedColor::Yellow))
                            .dim(true);
                        log.push(vec![
                            Segment::styled(prefix, style.clone()),
                            Segment::styled(msg.content.clone(), style),
                        ]);
                        continue;
                    }
                    ChatRole::System => (
                        "  ",
                        Style::default()
                            .fg(Color::Named(NamedColor::Magenta))
                            .italic(true),
                    ),
                };

                log.push(vec![
                    Segment::styled(prefix, style.clone()),
                    Segment::styled(msg.content.clone(), style),
                ]);
            }

            if !state.streaming_text.is_empty() {
                let style = Style::default().fg(Color::Named(NamedColor::Cyan));
                log.push(vec![
                    Segment::styled("  ", style.clone()),
                    Segment::styled(state.streaming_text.clone(), style),
                ]);
            }
        }

        self.app.request_render();
        self.last_sig = sig;
        Ok(())
    }

    /// Scroll the message view up by `lines`.
    pub fn scroll_messages_up(&mut self, lines: usize) {
        if let Some(log) = self
            .app
            .dom_mut()
            .downcast_widget_mut::<RichLog>(self.messages)
        {
            log.scroll_up_by(lines);
        }
        self.app.request_render();
    }

    /// Scroll the message view down by `lines`.
    pub fn scroll_messages_down(&mut self, lines: usize) {
        if let Some(log) = self
            .app
            .dom_mut()
            .downcast_widget_mut::<RichLog>(self.messages)
        {
            log.scroll_down_by(lines);
        }
        self.app.request_render();
    }

    /// Render a frame if the runtime is dirty.
    pub fn render_if_needed(&mut self, terminal: &mut dyn Terminal) -> anyhow::Result<bool> {
        self.app
            .render_if_needed(terminal)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    /// Render a frame unconditionally.
    pub fn render_frame(&mut self, terminal: &mut dyn Terminal) -> anyhow::Result<()> {
        self.app
            .render_frame(terminal)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }
}
