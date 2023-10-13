/// Pads an hexadecimal value to be 32 bytes long + 0x prefix.
pub fn pad_hex(input: &str) -> String {
    if input.len() > 64 + 2 {
        return String::new();
    }

    if input.len() == 64 + 2 {
        return input.to_string();
    }

    let s = input.strip_prefix("0x").unwrap_or(input);

    let mut padded = String::with_capacity(64);
    let padding_count = 64 - s.len();

    for _ in 0..padding_count {
        padded.push('0');
    }

    padded.push_str(s);

    format!("0x{padded}")
}

/// Returns true if the given string is an hexadecimal string with `0x` prefix, false otherwise.
pub fn is_hexadecimal_with_prefix(input: &str) -> bool {
    if input.len() < 3 {
        return false;
    }

    if &input[0..2] != "0x" {
        return false;
    }

    for c in input[2..].chars() {
        if !c.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}
