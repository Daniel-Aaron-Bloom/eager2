use eager2::eager;

#[test]
fn to_case() {
    let r = eager! {ccase!("myVarName", t: "snake")};
    assert_eq!(r, "my_var_name");

    let r = eager! {token_eq!({my_var_name}, {ccase!(myVarName, t: "snake")})};
    assert!(r);

    let r = eager! {ccase!("myVarName", t: "kebab")};
    assert_eq!(r, "my-var-name");
    let r = eager! {ccase!("my Var Name", t: "kebab")};
    assert_eq!(r, "my-var-name");
}

#[test]
fn from_case() {
    let r = eager! {ccase!("my_var-name", f: "snake", t: "pascal")};
    assert_eq!(r, "MyVar-name");

    let r = eager! {ccase!("MyVar-name", f: "pascal", t: "snake")};
    assert_eq!(r, "my_var-name");

    let r = eager! {ccase!("my Var-name", f: "lower", t: "snake")};
    assert_eq!(r, "my_var-name");
}

#[test]
fn pattern_only() {
    let r = eager! {ccase!("MY_VAR_NAME", p: "capital")};
    assert_eq!(r, "MyVarName");
    let r = eager! {ccase!("MY_VAR_NAME", p: "Sentence")};
    assert_eq!(r, "Myvarname");
}

#[test]
fn delimiter() {
    let r = eager! {ccase!("myVarName", p: "sentence", d: ".")};
    assert_eq!(r, "My.var.name");
}

#[test]
fn case_inputs_not_lower() {
    let r = eager! {ccase!("myVarName", t: "SNAKE")};
    assert_eq!(r, "my_var_name");

    let r = eager! {ccase!("myVarName", t: "SnAkE")};
    assert_eq!(r, "my_var_name");

    let r = eager! {ccase!("my-varName", t: "SnAkE", f: "KEBab")};
    assert_eq!(r, "my_varname");

    let r = eager! {ccase!("my-varName", t: "SnAkE", f: "KEBAB")};
    assert_eq!(r, "my_varname");
}

#[test]
fn empty_string_input() {
    let r = eager! {ccase!("", t: "snake", f: "KEBAB")};
    assert_eq!(r, "");
}

#[test]
fn boundaries() {
    let r = eager! {ccase!("myVar-Name-Longer", t: "snake", b: "aA")};
    assert_eq!(r, "my_var-name-longer");

    let r = eager! {ccase!("myVar-Name-Longer", t: "snake", b: "-")};
    assert_eq!(r, "myvar_name_longer");
}
