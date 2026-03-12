/// A command parsed from serial input.
#[derive(Debug, PartialEq)]
pub enum Command<'a> {
    /// Print current temperature status
    Status,
    /// Change the die-to-ambient offset
    SetOffset(f32),
    /// Show available commands
    Help,
    /// Input was not a recognised command
    Unknown(&'a str),
}

/// Parse a line of text into a command.
pub fn parse(input: &str) -> Command<'_> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Command::Unknown(trimmed);
    }

    // Split into command and optional argument
    let (cmd, arg) = match trimmed.find(' ') {
        Some(pos) => (&trimmed[..pos], trimmed[pos + 1..].trim()),
        None => (trimmed, ""),
    };

    match cmd {
        "status" => Command::Status,
        "help" => Command::Help,
        "offset" => match arg.parse::<f32>() {
            Ok(val) => Command::SetOffset(val),
            Err(_) => Command::Unknown(trimmed),
        },
        _ => Command::Unknown(trimmed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_status() {
        assert_eq!(parse("status"), Command::Status);
        assert_eq!(parse("  status  "), Command::Status);
    }

    #[test]
    fn parse_help() {
        assert_eq!(parse("help"), Command::Help);
    }

    #[test]
    fn parse_offset_valid() {
        assert_eq!(parse("offset 25.5"), Command::SetOffset(25.5));
        assert_eq!(parse("offset -10"), Command::SetOffset(-10.0));
    }

    #[test]
    fn parse_offset_missing_value() {
        assert_eq!(parse("offset"), Command::Unknown("offset"));
    }

    #[test]
    fn parse_offset_invalid_value() {
        assert_eq!(parse("offset abc"), Command::Unknown("offset abc"));
    }

    #[test]
    fn parse_unknown() {
        assert_eq!(parse("reboot"), Command::Unknown("reboot"));
    }

    #[test]
    fn parse_empty() {
        assert_eq!(parse(""), Command::Unknown(""));
    }
}
