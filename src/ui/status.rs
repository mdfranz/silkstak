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

pub struct RenderArgs<'a> {
    pub session: &'a Session,
    pub is_running: bool,
    pub loop_label: Option<&'a str>,
    pub prompt_name: Option<&'a str>,
    pub perm_mode: Option<&'a str>,
    pub btw_in: u64,
    pub btw_out: u64,
}

impl StatusLine {
    pub fn render(args: RenderArgs) -> String {
        let state = if args.is_running { "running" } else { "ready" };
        let dir = Path::new(&args.session.working_dir)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&args.session.working_dir);

        let ctx = args.session.context_window;
        let used = args.session.total_estimated_tokens;
        let pct = used
            .checked_mul(100)
            .and_then(|v| v.checked_div(ctx))
            .unwrap_or(0);

        let btw_badge = if args.btw_in > 0 || args.btw_out > 0 {
            format!(
                " btw:{}/{}",
                fmt_tokens(args.btw_in),
                fmt_tokens(args.btw_out)
            )
        } else {
            String::new()
        };

        let token_detail =
            if args.session.total_input_tokens > 0 || args.session.total_output_tokens > 0 {
                format!(
                    " i:{} o:{}",
                    fmt_tokens(args.session.total_input_tokens),
                    fmt_tokens(args.session.total_output_tokens),
                )
            } else {
                String::new()
            };

        let compact_badge = if args.session.compactions.is_empty() {
            String::new()
        } else {
            format!(" cmp:{}", args.session.compactions.len())
        };

        let loop_badge = match args.loop_label {
            Some(label) => format!(" [{}]", label),
            None => String::new(),
        };

        let prompt_badge = match args.prompt_name {
            Some(name) => format!(" [{}]", name),
            None => String::new(),
        };

        let perm_badge = match args.perm_mode {
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
            args.session.messages.len(),
            token_detail,
            compact_badge,
            state,
            loop_badge,
            prompt_badge,
            perm_badge,
        )
    }
}
