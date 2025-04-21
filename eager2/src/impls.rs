use eager2_core::pm::{ToTokens, TokenStream};

use crate::init::init;

pub fn eval(stream: TokenStream, eager: bool) -> TokenStream {
    init();
    let _name = if eager { "eager" } else { "lazy" };

    #[cfg(feature = "trace_macros")]
    println!("{} input: {}", _name, stream);

    let output = match eager2_core::funcs::mode(stream, eager) {
        Ok(output) => output,
        Err(err) => return err.into_token_stream(),
    };

    #[cfg(feature = "trace_macros")]
    println!("{} output: {}", _name, output);

    output
}

#[allow(clippy::needless_pass_by_value)]
pub fn eager_wrap(stream: TokenStream, name: &str) -> TokenStream {
    init();
    #[cfg(feature = "trace_macros")]
    println!("{} input: {}", name, stream);

    let output = match eager2_core::funcs::eager_wrap(stream, name) {
        Ok(output) => output,
        Err(err) => return err.into_token_stream(),
    };

    #[cfg(feature = "trace_macros")]
    println!("{} output: {}", name, output);

    output
}
