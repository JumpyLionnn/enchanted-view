use std::path::Path;
use byte_unit::Byte;

pub fn num_length(n: usize, base: usize) -> usize {
    let mut power = base;
    let mut count = 1;
    while n >= power {
        count += 1;
        if let Some(new_power) = power.checked_mul(base) {
            power = new_power;
        } else {
            break;
        }
    }
    count
}


pub fn format_bytes(bytes: u128) -> String {
    Byte::from_bytes(bytes).get_appropriate_unit(true).to_string()
}

#[cfg(not(target_os = "windows"))]
pub fn format_path<P: AsRef<Path>>(p: P) -> String {
    p.as_ref().display().to_string()
}

#[cfg(target_os = "windows")]
pub fn format_path<P: AsRef<Path>>(p: P) -> String {
    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let p = p.as_ref().display().to_string();
    if p.starts_with(VERBATIM_PREFIX) {
        p[VERBATIM_PREFIX.len()..].to_string()
    } else {
        p
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn num_length_test() {
        assert_eq!(num_length(200, 10), 3);
        assert_eq!(num_length(20540, 10), 5);
        assert_eq!(num_length(0, 10), 1);
        assert_eq!(num_length(3, 10), 1);
        assert_eq!(num_length(14, 10), 2);
        assert_eq!(num_length(200, 10), 3);
        assert_eq!(num_length(200, 16), 2);
        assert_eq!(num_length(200, 2), 8);
    }
}