use std::path::Path;

use crate::session::Session;

pub struct StatusLine;

fn fmt_tokens(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{}k", n / 1000)
    } else {
        n.to_string()
    }
}

impl StatusLine {
    pub fn render(
        session: &Session,
        is_running: bool,
        _spinner_tick: u64,
        loop_label: Option<&str>,
        prompt_name: Option<&str>,
        perm_mode: Option<&str>,
        btw_in: u64,
        btw_out: u64,
    ) -> String {
        let state = if is_running { "running" } else { "ready" };
        let dir = Path::new(&session.working_dir)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&session.working_dir);

        let ctx = session.context_window;
        let used = session.total_estimated_tokens;
        let pct = if ctx > 0 { (used * 100) / ctx } else { 0 };

        let btw_badge = if btw_in > 0 || btw_out > 0 {
            format!(" btw:{}/{}", fmt_tokens(btw_in), fmt_tokens(btw_out))
        } else {
            String::new()
        };

        let token_detail = if session.total_input_tokens > 0 || session.total_output_tokens > 0 {
            format!(
                " i:{} o:{}",
                fmt_tokens(session.total_input_tokens),
                fmt_tokens(session.total_output_tokens),
            )
        } else {
            String::new()
        };

        let compact_badge = if session.compactions.is_empty() {
            String::new()
        } else {
            format!(" cmp:{}", session.compactions.len())
        };

        let loop_badge = match loop_label {
            Some(label) => format!(" [{}]", label),
            None => String::new(),
        };

        let prompt_badge = match prompt_name {
            Some(name) => format!(" [{}]", name),
            None => String::new(),
        };

        let perm_badge = match perm_mode {
            Some(m) if m != "standard" => format!(" | mode:{}", m),
            _ => String::new(),
        };

        format!(
            "{}{} | {}/{} ({}%) | {}msgs{}{} | {}{}{}{}",
            dir,
            btw_badge,
            fmt_tokens(used),
            fmt_tokens(ctx),
            pct,
            session.messages.len(),
            token_detail,
            compact_badge,
            state,
            loop_badge,
            prompt_badge,
            perm_badge,
        )
    }
}
