use eager2_core::{exec, parse};

use crate::{
    exec::{execute_ccase, execute_concat},
    parse::{expect_call_literal, expect_string_literal, get_string_literal},
};

pub fn init() {
    let _ = parse::FNS.set(parse::Fns {
        expect_call_literal,
        expect_string_literal,
        get_string_literal,
    });
    let _ = exec::FNS.set(exec::Fns {
        execute_concat,
        execute_ccase,
    });
}
