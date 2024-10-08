#![no_std]

#[proc_macro_attribute]
pub fn main(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType};

// Define the attribute macro
#[proc_macro_attribute]
pub fn async_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(item as ItemFn);

    // Check if the function is async
    let is_async = input.sig.asyncness.is_some();
    if !is_async {
        return syn::Error::new_spanned(input.sig.fn_token, "Function must be async")
            .to_compile_error()
            .into();
    }

    // Check if the return type is ()
    let is_returning_unit = matches!(input.sig.output, ReturnType::Default);
    if !is_returning_unit {
        return syn::Error::new_spanned(input.sig.output, "Function must return ()")
            .to_compile_error()
            .into();
    }

    // Get the function identifier (name)
    let fun_name = input.sig.ident.clone();
    let fun_body = input.block.clone();

    // Generate the new function that wraps around the provided function
    let expanded = quote! {
        // #input

        [cortex_m_rt::entry]
        fn main() -> ! {
            cortex_m_asyncrt::os::init_heap();
            let mut executor = cortex_m_async::os::executor::Executor::new::<64>();

            executor.spawn(Task::new( async #fun_body ));

            executor.run();
            loop {}
        }
    };

    // Return the generated code as a TokenStream
    TokenStream::from(expanded)
}
