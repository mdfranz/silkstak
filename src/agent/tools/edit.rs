use rig::completion::ToolDefinition;
use rig::tool::Tool;

use crate::agent::tools::{
    AskSender, EditArgs, EditBlock, PermCheck, ToolError, check_perm_path,
    levenshtein_similarity, normalize_whitespace,
};

pub struct EditTool {
    pub permission: Option<PermCheck>,
    pub ask_tx: Option<AskSender>,
}

impl EditTool {
    pub fn new(permission: Option<PermCheck>, ask_tx: Option<AskSender>) -> Self {
        EditTool { permission, ask_tx }
    }
}

fn parse_blocks(raw: &str) -> Result<Vec<EditBlock>, ToolError> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut search_lines: Vec<String> = Vec::new();
    let mut replace_lines: Vec<String> = Vec::new();
    let mut phase: u8 = 0;

    for line in raw.lines() {
        match line.trim() {
            "<<<<<<< SEARCH" => {
                if in_block {
                    return Err(ToolError::Msg(
                        "Nested SEARCH/REPLACE block detected. Close each block with >>>>>>> REPLACE before starting a new one.".to_string(),
                    ));
                }
                in_block = true;
                search_lines.clear();
                replace_lines.clear();
                phase = 1;
            }
            "=======" if phase == 1 => {
                phase = 2;
            }
            ">>>>>>> REPLACE" if phase == 2 => {
                let search = search_lines.join("\n");
                if search.is_empty() {
                    return Err(ToolError::Msg(format!(
                        "Block {} has empty search text. Each block must have a non-empty SEARCH section.",
                        blocks.len() + 1
                    )));
                }
                blocks.push(EditBlock {
                    search,
                    replace: replace_lines.join("\n"),
                });
                in_block = false;
                phase = 0;
            }
            _ if phase == 1 => {
                search_lines.push(line.to_string());
            }
            _ if phase == 2 => {
                replace_lines.push(line.to_string());
            }
            _ => {}
        }
    }

    if blocks.is_empty() {
        return Err(ToolError::Msg(
            "No SEARCH/REPLACE blocks found. Use format:\n<<<<<<< SEARCH\nexisting code to find\n=======\nreplacement code\n>>>>>>> REPLACE\n\nMultiple blocks can be included for editing different parts of the same file."
                .to_string(),
        ));
    }

    Ok(blocks)
}

enum MatchResult {
    Exact(usize),
    Normalized(usize, usize),
    FuzzyApply(usize, usize, f64),
    FuzzySuggest(usize, f64, String),
    NotFound,
}

fn compute_byte_range(content: &str, norm_pos: usize, norm_len: usize) -> (usize, usize) {
    let content_norm = normalize_whitespace(content);
    let norm_end = (norm_pos + norm_len).min(content_norm.len());

    let orig_lines: Vec<&str> = content.lines().collect();

    // Walk original and normalized content line by line, tracking byte positions
    let mut orig_byte_start = 0usize;
    let mut orig_byte_end = 0usize;
    let mut norm_byte = 0usize;
    let mut found_start = false;

    for orig_line in &orig_lines {
        let orig_line_len = orig_line.len() + 1;
        let norm_line = normalize_whitespace(orig_line);
        let norm_line_len = norm_line.len() + 1;

        if !found_start && norm_byte + norm_line.len() >= norm_pos {
            found_start = true;
            orig_byte_start = orig_byte_end;
        }

        if found_start {
            if norm_byte + norm_line_len >= norm_end {
            // Match ends within this line
                return (orig_byte_start, orig_byte_end + orig_line_len.saturating_sub(1));
            }
        }

        orig_byte_end += orig_line_len;
        norm_byte += norm_line_len;
    }

    (orig_byte_start, content.len())
}

fn find_best_match(content: &str, search: &str) -> MatchResult {
    // Step 1: exact match in original content
    if let Some(pos) = content.find(search) {
        return MatchResult::Exact(pos);
    }

    // Step 2: normalized match in full text
    let content_norm = normalize_whitespace(content);
    let search_norm = normalize_whitespace(search);
    if let Some(norm_pos) = content_norm.find(&search_norm) {
        let (byte_start, byte_end) = compute_byte_range(content, norm_pos, search_norm.len());
        return MatchResult::Normalized(byte_start, byte_end);
    }

    // Step 3: fuzzy line-level matching
    let search_lines: Vec<&str> = search.lines().collect();
    let content_lines: Vec<&str> = content.lines().collect();

    if search_lines.is_empty() || content_lines.len() < search_lines.len() {
        return MatchResult::NotFound;
    }

    let search_norm_lines: Vec<String> =
        search_lines.iter().map(|l| normalize_whitespace(l)).collect();
    let search_norm_joined = search_norm_lines.join("\n");

    let mut best_sim = 0.0f64;
    let mut best_start = 0usize;

    for start in 0..=content_lines.len() - search_lines.len() {
        let window_norm: String = content_lines[start..start + search_lines.len()]
            .iter()
            .map(|l| normalize_whitespace(l))
            .collect::<Vec<_>>()
            .join("\n");
        let sim = levenshtein_similarity(&search_norm_joined, &window_norm);
        if sim > best_sim {
            best_sim = sim;
            best_start = start;
        }
        if sim >= 0.999 {
            break;
        }
    }

    if best_sim >= 0.85 {
        let byte_start: usize = content_lines[..best_start]
            .iter()
            .map(|l| l.len() + 1)
            .sum();
        let byte_end = byte_start
            + content_lines[best_start..best_start + search_lines.len()]
                .iter()
                .map(|l| l.len() + 1)
                .sum::<usize>()
                .saturating_sub(1);
        MatchResult::FuzzyApply(byte_start, byte_end, best_sim)
    } else if best_sim >= 0.60 {
        let preview: String = search_lines
            .iter()
            .take(3)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        MatchResult::FuzzySuggest(best_start + 1, best_sim, preview)
    } else {
        MatchResult::NotFound
    }
}

fn count_exact_matches(content: &str, search: &str) -> usize {
    content.match_indices(search).count()
}

impl Tool for EditTool {
    const NAME: &'static str = "edit";

    type Error = ToolError;
    type Args = EditArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "edit".to_string(),
            description: "Edit a file using aider-style SEARCH/REPLACE blocks. Each block finds exact text and replaces it. Multiple blocks in one call are applied atomically. If the search text is not an exact match, whitespace normalization and fuzzy matching are attempted as fallbacks.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Path to the file (relative or absolute)" },
                    "block": { "type": "string", "description": "One or more SEARCH/REPLACE blocks:\n<<<<<<< SEARCH\nexisting code to find\n=======\nreplacement code\n>>>>>>> REPLACE\n\nInclude multiple blocks for separate edits to the same file." }
                },
                "required": ["path", "block"]
            }),
        }
    }

    async fn call(&self, args: EditArgs) -> Result<String, ToolError> {
        let path = crate::fs::expand_tilde(&args.path);
        check_perm_path(&self.permission, &self.ask_tx, "edit", &path).await?;

        let blocks = parse_blocks(&args.block)?;

        let bytes = tokio::fs::read(&path).await?;
        let has_crlf = bytes.windows(2).any(|w| w == b"\r\n");
        let content = String::from_utf8_lossy(&bytes).replace("\r\n", "\n");

        struct Resolved {
            byte_start: usize,
            byte_end: usize,
            replace: String,
            note: String,
        }

        let mut resolved: Vec<Resolved> = Vec::new();

        for (i, block) in blocks.iter().enumerate() {
            let label = if blocks.len() > 1 {
                format!("Block {}: ", i + 1)
            } else {
                String::new()
            };

            match find_best_match(&content, &block.search) {
                MatchResult::Exact(pos) => {
                    let count = count_exact_matches(&content, &block.search);
                    if count > 1 {
                        let line_starts: Vec<usize> = std::iter::once(0)
                            .chain(content.match_indices('\n').map(|(i, _)| i + 1))
                            .collect();

                        let mut match_info = Vec::new();
                        for byte_idx in
                            content.match_indices(&block.search).map(|(i, _)| i)
                        {
                            let line_num = match line_starts.binary_search(&byte_idx) {
                                Ok(i) => i + 1,
                                Err(i) => i,
                            };
                            let ls = line_starts.get(line_num - 1).copied().unwrap_or(0);
                            let le = content[ls..]
                                .find('\n')
                                .map(|e| ls + e)
                                .unwrap_or(content.len());
                            let text: String = content[ls..le].chars().take(100).collect();
                            match_info.push(format!("  Line {}: {}", line_num, text));
                        }

                        return Err(ToolError::Msg(format!(
                            "{label}search text matched {} times in {}:\n{}\n\nAdd more surrounding context to the SEARCH block to make it unique.",
                            count,
                            path,
                            match_info.join("\n"),
                        )));
                    }
                    resolved.push(Resolved {
                        byte_start: pos,
                        byte_end: pos + block.search.len(),
                        replace: block.replace.clone(),
                        note: String::new(),
                    });
                }
                MatchResult::Normalized(start, end) => {
                    resolved.push(Resolved {
                        byte_start: start,
                        byte_end: end,
                        replace: block.replace.clone(),
                        note: "matched after whitespace normalization".to_string(),
                    });
                }
                MatchResult::FuzzyApply(start, end, sim) => {
                    resolved.push(Resolved {
                        byte_start: start,
                        byte_end: end,
                        replace: block.replace.clone(),
                        note: format!("fuzzy match, {:.0}% similarity", sim * 100.0),
                    });
                }
                MatchResult::FuzzySuggest(line, sim, preview) => {
                    return Err(ToolError::Msg(format!(
                        "{label}search text not found in '{}'. Closest match at line {}, {:.0}% similar:\n  {}\n\nRead the file around that area, copy the exact text, and retry the edit.",
                        path,
                        line,
                        sim * 100.0,
                        preview,
                    )));
                }
                MatchResult::NotFound => {
                    return Err(ToolError::Msg(format!(
                        "{label}search text not found in '{}'.\nRead the file and copy the exact text for the SEARCH block, ensuring whitespace and indentation match.",
                        path,
                    )));
                }
            }
        }

        // Apply last-to-first so earlier byte positions remain valid
        resolved.sort_by_key(|r| std::cmp::Reverse(r.byte_start));

        let mut modified = content;
        let mut notes = Vec::new();

        for rb in &resolved {
            if rb.byte_end > modified.len() || rb.byte_start > modified.len() {
                return Err(ToolError::Msg(
                    "Internal error: search range exceeds file bounds. The file may have changed. Re-read and retry."
                        .to_string(),
                ));
            }
            modified.replace_range(rb.byte_start..rb.byte_end, &rb.replace);
            if !rb.note.is_empty() {
                notes.push(rb.note.clone());
            }
        }

        let output = if has_crlf {
            modified.replace('\n', "\r\n")
        } else {
            modified
        };

        tokio::fs::write(&path, &output).await?;

        let mut result = format!("Applied {} edit(s) to {}", blocks.len(), path);
        for note in &notes {
            result.push_str(&format!("\n  Note: {}", note));
        }

        Ok(result)
    }
}
