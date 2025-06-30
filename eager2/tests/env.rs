#[test]
fn test_env() {
    const FOO: Option<&str> = eager2::option_env!("PATH");
    const BAR: Option<&str> = option_env!("PATH");

    assert_eq!(FOO, BAR);
}

#[test]
fn test_line() {
    const FOO: u32 = eager2::line!();
    const BAR: u32 = line!();

    assert_eq!(FOO + 1, BAR);
}

#[test]
fn test_column() {
    const FOO: u32 = eager2::column!();
    const BAR: u32 = column!();

    assert_eq!(FOO, BAR);
}

#[test]
fn test_file() {
    const FOO: &str = eager2::file!();
    const BAR: &str = file!();

    assert_eq!(FOO, BAR);
}
