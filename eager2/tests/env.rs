#[test]
fn test_env() {
    const FOO: Option<&str> = eager2::option_env!("PATH");
    const BAR: Option<&str> = option_env!("PATH");

    assert_eq!(FOO, BAR);
}

#[test]
fn test_file() {
    const FOO: &str = eager2::file!();
    const BAR: &str = file!();

    assert_eq!(FOO, BAR);
}
