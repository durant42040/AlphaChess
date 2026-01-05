use std::io::Write;
use std::process::{Command, Stdio};

pub fn generate_move() -> std::io::Result<String> {
    let input = "position startpos\n";

    let mut child = Command::new("stockfish")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    {
        let stdin = child.stdin.as_mut().unwrap();
        stdin.write_all(input.as_bytes())?;
        stdin.write_all(b"go depth 20\n")?;
        stdin.flush()?;
    }

    let output = child.wait_with_output()?;
    let text = String::from_utf8_lossy(&output.stdout);

    for line in text.lines() {
        if let Some(idx) = line.find("bestmove") {
            let rest = &line[idx + 9..];
            let bestmove = rest.split_whitespace().next().unwrap_or("");
            return Ok(bestmove.to_string());
        }
    }

    Ok(String::new())
}
