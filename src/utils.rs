macro_rules! get_cge_path {
  (
    $stream:ident,
    $name:expr
  ) => {
    {
      // parse first stream item as expression (I am bad at macros this is all I could manage lol)
      // somebody good at macros please help me
      let expression = syn::parse_macro_input!($stream as syn::Expr);

      match expression {
        syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(string), .. }) => string.value(),
        _ => panic!("Didn't get string literal path to CGE file. Usage: `#[{}(\"path/to/cge/file.cge\")]`", $name),
      }
    }
  };
}

#[allow(unused_imports)]
pub(crate) use get_cge_path;