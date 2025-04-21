const FOO: Option<&str> = eager2::option_env!("PATH");
const BAR: Option<&str> = option_env!("PATH");

#[test]
fn test() {
    assert_eq!(FOO, BAR);
}
