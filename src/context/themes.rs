use std::collections::HashMap;
use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::config::ColorsConfig;
use crate::ui::renderer::Renderer;
use crate::ui::parse_color;

static EMBEDDED: Dir = include_dir!("$CARGO_MANIFEST_DIR/themes");

pub fn global_themes_dir() -> PathBuf {
    crate::session::storage::data_dir().join("themes")
}

pub fn load() -> HashMap<String, String> {
    let mut themes: HashMap<String, String> = HashMap::new();

    for file in EMBEDDED.files() {
        if file.path().extension().is_some_and(|e| e == "json")
            && let Some(name) = file.path().file_stem().and_then(|s| s.to_str())
            && let Some(content) = file.contents_utf8()
        {
            themes
                .entry(name.to_string())
                .or_insert_with(|| content.to_string());
        }
    }

    let global = global_themes_dir();
    if global.exists()
        && let Ok(entries) = std::fs::read_dir(&global)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(content) = std::fs::read_to_string(&path)
            {
                themes.insert(name.to_string(), content);
            }
        }
    }

    let local = PathBuf::from("themes");
    if local.exists()
        && let Ok(entries) = std::fs::read_dir(&local)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "json")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && let Ok(content) = std::fs::read_to_string(&path)
            {
                themes.insert(name.to_string(), content);
            }
        }
    }

    themes
}

pub fn ensure_global() -> anyhow::Result<()> {
    let dir = global_themes_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
        copy_embedded(&dir)?;
    }
    Ok(())
}

pub fn regen() -> anyhow::Result<()> {
    let dir = global_themes_dir();
    std::fs::create_dir_all(&dir)?;
    copy_embedded(&dir)
}

fn copy_embedded(dest: &Path) -> anyhow::Result<()> {
    for file in EMBEDDED.files() {
        if let Some(name) = file.path().file_name().and_then(|s| s.to_str()) {
            let dest_path = dest.join(name);
            if let Some(content) = file.contents_utf8() {
                std::fs::write(&dest_path, content)?;
            }
        }
    }
    Ok(())
}

pub fn apply(content: &str, renderer: &mut Renderer) {
    if let Ok(colors) = serde_json::from_str::<ColorsConfig>(content) {
        let chat_bg = colors.chat_background.as_deref().and_then(parse_color);
        let input_bg = colors.input_background.as_deref().and_then(parse_color);
        let status_bg = colors.status_background.as_deref().and_then(parse_color);
        renderer.set_background_colors(chat_bg, input_bg, status_bg);
    }
}
