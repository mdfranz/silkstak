use std::io::Write;

use crossterm::ExecutableCommand;

use crate::ui::slash::{SlashCtx, write_error, write_ok};

pub(crate) const AGENTS_CREATION_PROMPT: &str = "\
Create an AGENTS.md file for this project. Read existing AGENTS.md or CLAUDE.md files \
in parent directories, README.md, and any config files to understand the project first. \
Then write a comprehensive AGENTS.md that documents: \
1) the overall purpose and architecture \
2) build/test/lint commands \
3) coding style and conventions \
4) directory layout \
Keep it focused and actionable for a coding agent.";

const AGENTS_DESC: &str = "tells coding agents how to build, test, and work with this codebase";

fn ask_yn(question: &str) -> bool {
    print!("{question} ");
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn exit_tui_for_io() {
    let _ = crossterm::terminal::disable_raw_mode();
    let mut stdout = std::io::stdout();
    let _ = stdout.execute(crossterm::event::DisableMouseCapture);
    let _ = stdout.execute(crossterm::terminal::LeaveAlternateScreen);
    let _ = stdout.flush();
}

fn restore_tui_and_render(ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();
    let _ = stdout.execute(crossterm::terminal::EnterAlternateScreen);
    let _ = stdout.execute(crossterm::terminal::Clear(
        crossterm::terminal::ClearType::All,
    ));
    let _ = stdout.execute(crossterm::event::EnableMouseCapture);
    let _ = crossterm::terminal::enable_raw_mode();
    crate::ui::events::render_session(ctx.renderer, ctx.session, ctx.cli, ctx.cfg, ctx.context)
}

fn build_question(label: &str, desc: &str, exists: bool, cwd: &std::path::Path) -> String {
    if exists {
        format!("{} already exists ({desc}). Overwrite? [y/N]", label)
    } else {
        format!(
            "No {} found in {} ({desc}). Create one? [y/N]",
            label,
            cwd.display()
        )
    }
}

pub async fn handle(parts: &[&str], ctx: &mut SlashCtx<'_>) -> anyhow::Result<()> {
    let force = parts.len() >= 2 && parts[1] == "force";

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let agents_path = cwd.join("AGENTS.md");

    let agents_exists = agents_path.exists();

    let create_agents = if force {
        if agents_exists {
            write_ok(ctx.renderer, "AGENTS.md exists — will overwrite");
        }
        true
    } else {
        exit_tui_for_io();

        let create_a = ask_yn(&build_question(
            "AGENTS.md",
            AGENTS_DESC,
            agents_exists,
            &cwd,
        ));

        restore_tui_and_render(ctx)?;

        create_a
    };

    if create_agents {
        if !ctx.context.prompts.contains_key("code") {
            write_error(
                ctx.renderer,
                "no 'code' prompt found. Run /regen-prompts first.",
            );
            return Ok(());
        }
        write_ok(ctx.renderer, "delegating AGENTS.md creation to agent...");
        return Err(anyhow::anyhow!("DEFER_INIT:{}", AGENTS_CREATION_PROMPT));
    }

    Ok(())
}
