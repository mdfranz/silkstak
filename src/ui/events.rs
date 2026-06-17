use chrono::Datelike;
use compact_str::CompactString;
use crossterm::style::Color;

use crate::cli::Cli;
use crate::config::Config;
use crate::context::ContextFiles;
use crate::session::{MessageRole, Session};
use crate::ui::markdown;
use crate::ui::renderer::Renderer;

pub fn format_time(rfc3339: &str) -> CompactString {
    let dt = chrono::DateTime::parse_from_rfc3339(rfc3339).ok();
    let dt = match dt {
        Some(dt) => dt,
        None => return CompactString::new(rfc3339),
    };
    let local = dt.with_timezone(&chrono::Local);
    let now = chrono::Local::now();
    if local.date_naive() == now.date_naive() {
        CompactString::new(local.format("%H:%M").to_string())
    } else if local.year() == now.year() {
        CompactString::new(local.format("%b %d %H:%M").to_string())
    } else {
        CompactString::new(local.format("%Y-%m-%d %H:%M").to_string())
    }
}

pub fn render_session(
    renderer: &mut Renderer,
    session: &Session,
    _cli: &Cli,
    _cfg: &Config,
    context: &ContextFiles,
) -> anyhow::Result<()> {
    renderer.clear_content()?;
    let welcome = format!(
        "silkstak {} ({})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_COMMIT_HASH"),
    );
    renderer.write_line(&welcome, Color::Cyan)?;
    renderer.write_line("", Color::White)?;
    if context.agents.is_some() {
        renderer.write_line("loaded AGENTS.md", Color::DarkGrey)?;
        renderer.write_line("", Color::White)?;
    }
    #[cfg(feature = "archmd")]
    if context.architecture.is_some() {
        renderer.write_line("loaded ARCHITECTURE.md", Color::DarkGrey)?;
        renderer.write_line("", Color::White)?;
    }
    if !session.compactions.is_empty() {
        renderer.write_line(
            &format!(
                "compacted {} times (saved ~{} tokens)",
                session.compactions.len(),
                session
                    .compactions
                    .last()
                    .map(|c| c.token_savings)
                    .unwrap_or(0),
            ),
            Color::DarkGrey,
        )?;
        renderer.write_line("", Color::White)?;
    }
    let user_color = renderer.user_color();
    let text_color = renderer.text_color();
    for msg in &session.messages {
        let (prefix, _c) = match msg.role {
            MessageRole::User => (">", user_color),
            MessageRole::Assistant => ("<", text_color),
            MessageRole::System => ("#", Color::DarkGrey),
        };
        if msg.role == MessageRole::Assistant {
            let max_width = renderer.line_width();
            let mut styled = markdown::markdown_to_styled(&msg.content, max_width);
            if !styled.is_empty() {
                styled[0].text = CompactString::from(format!("{} {}", prefix, styled[0].text));
            }
            for mut entry in styled {
                if entry.color == Color::White {
                    entry.color = text_color;
                }
                renderer.write_line(&entry.text, entry.color)?;
            }
        } else {
            for line in msg.content.lines() {
                renderer.write_line(&format!("{} {}", prefix, line), _c)?;
            }
        }
        renderer.write_line("", Color::White)?;
        if msg.role == MessageRole::User {
            renderer.write_line("", Color::White)?;
        }
    }
    Ok(())
}

pub fn show_welcome(renderer: &mut Renderer) -> std::io::Result<()> {
    use super::C_TOOL;
    use crossterm::style::Color;
    let text_color = renderer.text_color();

    renderer.write_line("──────────────────────────────────────────", Color::Cyan)?;
    renderer.write_line("  silkstak Quickstart", Color::Cyan)?;
    renderer.write_line("──────────────────────────────────────────", Color::Cyan)?;
    renderer.write_line("", Color::White)?;
    renderer.write_line("  Pickers:", C_TOOL)?;
    renderer.write_line(
        "    @<path>     File picker / auto-complete paths",
        text_color,
    )?;
    renderer.write_line(
        "    !<command>  Run a shell command (output stored as assistant)",
        text_color,
    )?;
    renderer.write_line(
        "    .<prompt>   Switch prompt or one-shot .<prompt> <message>",
        text_color,
    )?;
    renderer.write_line("", Color::White)?;
    renderer.write_line("  Slash Commands:", C_TOOL)?;
    renderer.write_line("    /model        Switch model", text_color)?;
    renderer.write_line("    /prompt       List / activate prompts", text_color)?;
    renderer.write_line(
        "    .autoconfig        Switches to auto-configurator",
        text_color,
    )?;
    renderer.write_line("    /mode         Change security mode", text_color)?;
    renderer.write_line("    /clear        Clear session", text_color)?;
    renderer.write_line("    /undo         Undo last exchange", text_color)?;
    renderer.write_line("    /compress     Free context window space", text_color)?;
    renderer.write_line("    /help         Show all commands", text_color)?;
    renderer.write_line("", Color::White)?;
    renderer.write_line("  Keybindings:", C_TOOL)?;
    renderer.write_line("    Ctrl+G     Open input in $EDITOR", text_color)?;
    renderer.write_line("    Ctrl+H     Launch lazygit", text_color)?;
    renderer.write_line("    Ctrl+S     Save session", text_color)?;
    renderer.write_line("    Tab        File picker / auto-complete", text_color)?;
    renderer.write_line(
        "  Website: https://github.com/gi-dellav/silkstak",
        text_color,
    )?;
    renderer.write_line("", Color::White)?;
    renderer.write_line("──────────────────────────────────────────", Color::Cyan)?;
    renderer.write_line("", Color::White)?;
    Ok(())
}

pub fn sanitize_output(text: &str) -> CompactString {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.next() {
                Some('[') | Some(']') => {
                    for next in &mut chars {
                        if next.is_ascii_alphabetic() || next == '~' {
                            break;
                        }
                    }
                }
                Some(_) => {}
                None => break,
            }
        } else if c.is_ascii_control() && c != '\n' && c != '\t' && c != '\r' {
            continue;
        } else {
            result.push(c);
        }
    }
    CompactString::from(result)
}
