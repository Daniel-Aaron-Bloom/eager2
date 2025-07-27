use eager2_core::pm::{ToTokens, TokenStream};

use crate::init::init;

pub fn eager_macro_rules(stream: TokenStream) -> TokenStream {
    init();
    let output = match eager2_core::rules::eager_macro_rules(stream) {
        Ok(output) => output,
        Err(err) => return err.into_token_stream(),
    };

    #[cfg(feature = "trace-macros")]
    println!("eager_macro_rules output: {}", output);

    output
}

pub fn eager_macro(attr: TokenStream, stream: TokenStream) -> TokenStream {
    init();
    let output = match eager2_core::rules::eager_macro(attr, stream) {
        Ok(output) => output,
        Err(err) => return err.into_token_stream(),
    };

    #[cfg(feature = "trace-macros")]
    println!("eager_macro output: {}", output);

    output
}

pub fn eager_proc_macro(attr: TokenStream, stream: TokenStream) -> TokenStream {
    init();
    let output = match eager2_core::rules::eager_proc_macro(attr, stream) {
        Ok(output) => output,
        Err(err) => return err.into_token_stream(),
    };

    #[cfg(feature = "trace-macros")]
    println!("eager_proc_macro output: {}", output);

    output
}
