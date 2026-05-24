use std::path::Path;
use users::os::unix::UserExt;

pub fn expand_tilde(s: &str) -> String {
    if s == "~" || s == "$HOME" {
        if let Some(home) = dirs::home_dir() {
            return home.to_string_lossy().to_string();
        }
        return s.to_string();
    }
    if let Some(rest) = s.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return Path::new(&home).join(rest).to_string_lossy().to_string();
        }
        return s.to_string();
    }
    if let Some(rest) = s.strip_prefix('~') {
        if let Some(slash_pos) = rest.find('/') {
            let user = &rest[..slash_pos];
            let path_after = &rest[slash_pos + 1..];
            if let Some(user_entry) = users::get_user_by_name(user) {
                let home = user_entry.home_dir();
                return Path::new(home)
                    .join(path_after)
                    .to_string_lossy()
                    .to_string();
            }
        }
        return s.to_string();
    }
    if let Some(rest) = s.strip_prefix("$HOME/") {
        if let Some(home) = dirs::home_dir() {
            return Path::new(&home).join(rest).to_string_lossy().to_string();
        }
    }
    s.to_string()
}
