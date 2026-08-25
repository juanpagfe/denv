use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::Stylize,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

use crate::history::Entry;

/// Interactive fuzzy picker over clipboard history entries.
/// Returns the selected entry index, or None if cancelled.
pub fn run(entries: &[Entry], use_color: bool) -> io::Result<Option<usize>> {
    if entries.is_empty() {
        return Ok(None);
    }

    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let result = pick_loop(&mut stdout, entries, use_color);

    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    result
}

fn pick_loop(
    stdout: &mut io::Stdout,
    entries: &[Entry],
    use_color: bool,
) -> io::Result<Option<usize>> {
    let mut query = String::new();
    let mut selected: usize = 0;
    let mut scroll_offset: usize = 0;

    loop {
        let (_, term_height) = terminal::size()?;
        let max_visible = (term_height as usize).saturating_sub(3); // header + query + footer

        // Filter entries by query
        let filtered: Vec<(usize, &Entry)> = entries
            .iter()
            .enumerate()
            .filter(|(_, e)| {
                if query.is_empty() {
                    true
                } else {
                    let q = query.to_lowercase();
                    e.content.to_lowercase().contains(&q)
                }
            })
            .collect();

        if selected >= filtered.len() {
            selected = filtered.len().saturating_sub(1);
        }

        // Adjust scroll offset to keep selection visible
        if selected < scroll_offset {
            scroll_offset = selected;
        }
        if selected >= scroll_offset + max_visible {
            scroll_offset = selected - max_visible + 1;
        }

        // Render
        execute!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(ClearType::All)
        )?;

        // Header
        if use_color {
            write!(
                stdout,
                "{} ({} entries, {} matched)\r\n",
                "clipboard history".bold(),
                entries.len(),
                filtered.len()
            )?;
        } else {
            write!(
                stdout,
                "clipboard history ({} entries, {} matched)\r\n",
                entries.len(),
                filtered.len()
            )?;
        }

        // Query line
        write!(stdout, "> {query}_\r\n")?;

        // Entries
        let visible = &filtered[scroll_offset..filtered.len().min(scroll_offset + max_visible)];
        for (vis_idx, (_, entry)) in visible.iter().enumerate() {
            let abs_idx = scroll_offset + vis_idx;
            let is_selected = abs_idx == selected;

            // Truncate content to single line preview
            let preview = preview_content(&entry.content, 70);
            let time = entry.timestamp.format("%Y-%m-%d %H:%M");

            if is_selected && use_color {
                write!(
                    stdout,
                    "  {} {} {}\r\n",
                    "▸".green(),
                    preview.clone().reverse(),
                    format!("[{time}]").dark_grey(),
                )?;
            } else if is_selected {
                write!(stdout, "  > {preview}  [{time}]\r\n")?;
            } else if use_color {
                write!(
                    stdout,
                    "    {} {}\r\n",
                    preview,
                    format!("[{time}]").dark_grey()
                )?;
            } else {
                write!(stdout, "    {preview}  [{time}]\r\n")?;
            }
        }

        // Footer
        if use_color {
            write!(
                stdout,
                "\r\n{}",
                " ↑/↓ navigate  enter select  esc cancel  type to filter ".dark_grey()
            )?;
        } else {
            write!(
                stdout,
                "\r\n ↑/↓ navigate  enter select  esc cancel  type to filter"
            )?;
        }

        stdout.flush()?;

        // Input
        if let Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Ok(None);
                }
                KeyCode::Enter => {
                    if let Some(&(original_idx, _)) = filtered.get(selected) {
                        return Ok(Some(original_idx));
                    }
                    return Ok(None);
                }
                KeyCode::Up => {
                    selected = selected.saturating_sub(1);
                }
                KeyCode::Down => {
                    if selected + 1 < filtered.len() {
                        selected += 1;
                    }
                }
                KeyCode::Backspace => {
                    query.pop();
                    selected = 0;
                    scroll_offset = 0;
                }
                KeyCode::Char(c) => {
                    query.push(c);
                    selected = 0;
                    scroll_offset = 0;
                }
                _ => {}
            }
        }
    }
}

/// Create a single-line preview of content, truncated to max_len.
fn preview_content(content: &str, max_len: usize) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let suffix = if content.lines().count() > 1 {
        " …"
    } else {
        ""
    };

    let display = first_line.replace('\t', "  ");
    if display.len() + suffix.len() > max_len {
        let truncated: String = display.chars().take(max_len - 3 - suffix.len()).collect();
        format!("{truncated}...{suffix}")
    } else {
        format!("{display}{suffix}")
    }
}
