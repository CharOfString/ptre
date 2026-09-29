// Turn Cpplint output into diagnostics for the editor. Each finding is one line:
//   /path/main.cpp:3:  Missing spaces around =  [whitespace/operators] [4]
// Line 0 means the whole file, such as a missing copyright line.

use crate::lsp::{Diagnostic, Severity};
use std::path::Path;

// Findings for `path`, placed in `text` (the content Cpplint checked).
pub(super) fn parse(output: &str, path: &Path, text: &str) -> Vec<Diagnostic> {
    let prefix = format!("{}:", path.display());
    let lines: Vec<&str> = text.split('\n').collect();
    let mut diagnostics = Vec::new();
    for finding in output.lines() {
        let Some((number, rest)) = finding
            .strip_prefix(&prefix)
            .and_then(|rest| rest.split_once(':'))
        else {
            continue;
        };
        let Ok(number) = number.parse::<usize>() else {
            continue;
        };
        // The message ends with "  [category] [confidence]".
        let rest = rest.trim();
        let (message, category) = match rest.rsplit_once("  [") {
            Some((message, tail)) => (message.trim(), tail.split(']').next()),
            None => (rest, None),
        };

        // Mark the code of the line, without its indentation.
        let index = number.saturating_sub(1).min(lines.len() - 1);
        let line_start: usize = lines[..index]
            .iter()
            .map(|line| line.chars().count() + 1)
            .sum();
        let line = lines[index];
        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        diagnostics.push(Diagnostic {
            range: line_start + indent.min(line.chars().count())..line_start + line.chars().count(),
            severity: Severity::Warning,
            message: message.to_owned(),
            source: Some(category.map_or("cpplint".into(), |c| format!("cpplint: {c}"))),
        });
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings_become_line_diagnostics() {
        // Recorded from cpplint 2.0.2, on a path with a space in it.
        let path = Path::new("/tmp/lint dir/main.cpp");
        let output = "\
/tmp/lint dir/main.cpp:0:  No copyright message found.  You should have a line: \"Copyright [year] <Copyright Owner>\"  [legal/copyright] [5]
/tmp/lint dir/main.cpp:2:  Missing space before {  [whitespace/braces] [5]
/tmp/lint dir/main.cpp:3:  Missing spaces around =  [whitespace/operators] [4]
/tmp/lint dir/other.cpp:1:  Not this file  [build/x] [1]
Done processing /tmp/lint dir/main.cpp
Total errors found: 3
";
        let text = "#include <stdio.h>\nint main(void){\n  int x=1;  \n}\n";
        let diagnostics = parse(output, path, text);
        let summary: Vec<_> = diagnostics
            .iter()
            .map(|d| (d.range.clone(), d.message.as_str(), d.source.as_deref()))
            .collect();
        assert_eq!(
            summary,
            [
                (
                    0..18,
                    "No copyright message found.  You should have a line: \
                     \"Copyright [year] <Copyright Owner>\"",
                    Some("cpplint: legal/copyright")
                ),
                (
                    19..34,
                    "Missing space before {",
                    Some("cpplint: whitespace/braces")
                ),
                (
                    37..47,
                    "Missing spaces around =",
                    Some("cpplint: whitespace/operators")
                ),
            ]
        );
        assert!(diagnostics.iter().all(|d| d.severity == Severity::Warning));
        assert_eq!(
            parse("main.cpp:99:  x", Path::new("main.cpp"), "a")[0].range,
            0..1
        );
    }
}
