use std::io;

/// Parse a line range spec and extract the matching lines from content.
///
/// Supported formats:
///   "5"      → line 5 only
///   "5-10"   → lines 5 through 10 (inclusive)
///   "5-"     → line 5 to end
///   "-10"    → line 1 to 10
///
/// Lines are 1-indexed.
pub fn filter_lines(content: &str, spec: &str) -> io::Result<String> {
    let all_lines: Vec<&str> = content.lines().collect();
    let total = all_lines.len();

    if total == 0 {
        return Ok(String::new());
    }

    let (start, end) = parse_range(spec, total)?;

    let selected: Vec<&str> = all_lines
        .into_iter()
        .enumerate()
        .filter(|(i, _)| {
            let line_num = i + 1; // 1-indexed
            line_num >= start && line_num <= end
        })
        .map(|(_, line)| line)
        .collect();

    if selected.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("no lines matched range \"{spec}\" (file has {total} lines)"),
        ));
    }

    Ok(selected.join("\n"))
}

fn parse_range(spec: &str, total: usize) -> io::Result<(usize, usize)> {
    let spec = spec.trim();

    if let Some(rest) = spec.strip_suffix('-') {
        // "5-" → from 5 to end
        if rest.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid line range: \"-\" (missing start)",
            ));
        }
        let start = parse_num(rest)?;
        Ok((start, total))
    } else if let Some(rest) = spec.strip_prefix('-') {
        // "-10" → from 1 to 10
        let end = parse_num(rest)?;
        Ok((1, end))
    } else if spec.contains('-') {
        // "5-10"
        let parts: Vec<&str> = spec.splitn(2, '-').collect();
        let start = parse_num(parts[0])?;
        let end = parse_num(parts[1])?;
        if start > end {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid line range: start ({start}) > end ({end})"),
            ));
        }
        Ok((start, end))
    } else {
        // Single line "5"
        let line = parse_num(spec)?;
        Ok((line, line))
    }
}

fn parse_num(s: &str) -> io::Result<usize> {
    s.parse::<usize>().map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("\"{s}\" is not a valid line number"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_line() {
        let content = "line1\nline2\nline3\nline4\nline5";
        assert_eq!(filter_lines(content, "3").unwrap(), "line3");
    }

    #[test]
    fn test_range() {
        let content = "a\nb\nc\nd\ne";
        assert_eq!(filter_lines(content, "2-4").unwrap(), "b\nc\nd");
    }

    #[test]
    fn test_open_end() {
        let content = "a\nb\nc\nd\ne";
        assert_eq!(filter_lines(content, "3-").unwrap(), "c\nd\ne");
    }

    #[test]
    fn test_open_start() {
        let content = "a\nb\nc\nd\ne";
        assert_eq!(filter_lines(content, "-2").unwrap(), "a\nb");
    }

    #[test]
    fn test_out_of_range() {
        let content = "a\nb";
        assert!(filter_lines(content, "5-10").is_err());
    }

    #[test]
    fn test_inverted_range() {
        let content = "a\nb\nc";
        assert!(filter_lines(content, "3-1").is_err());
    }
}
