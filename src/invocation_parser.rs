use proc_macro2::Ident;
use syn::{Token, ExprLit, TypePath, Lit};
use crate::{macro_core::Config, numeric_type::NumericType};

impl syn::parse::Parse for crate::macro_core::Config {
  fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
    // extract the CGE path
    let cge_path = {
      match input.parse::<ExprLit>() {
        Ok(ExprLit { lit: Lit::Str(string), .. }) => string.value(),

        // abort compilation with error
        _ => panic!("Didn't get string literal path to CGE file. Make sure the CGE path is the first argument, like: `#[network(\"path/to/file.cge\")]`.")
      }
    };

    // parse the argument name
    let numeric_type = {
      if let Some(_) = input.parse::<Option<Token![,]>>().ok() {
        if let Some(arg_name) = input.parse::<Ident>().ok() {
          if arg_name != "numeric_type" { return Err(syn::Error::new_spanned(arg_name, "Nonsense attribute `{}`. Expected either `numeric_type` or nothing.")); }

          // discard '='
          let _: Token![=] = input.parse()
            .unwrap_or_else(|_| panic!("Expected '=' after `numeric_type` argument."));

          // parse the numeric type
          let name = input.parse::<TypePath>()
            .unwrap_or_else(|_| panic!("Expected type after `numeric_type = `. Please choose one of {{ {} }}", NumericType::VARIANTS_LIST));
          let type_ident = name.path.get_ident()
            .unwrap_or_else(|| panic!("Invalid `numeric_type`. Please use one of {{ {} }}.", NumericType::VARIANTS_LIST));

          match type_ident.to_string().as_ref() {
            "f32" => NumericType::Float32,
            "f64" => NumericType::Float64,
            _ => panic!("Invalid `numeric_type`. Please use one of {{ {} }}.", NumericType::VARIANTS_LIST)
          }
        } else {
          // assume f64 if not specified
          NumericType::Float64
        }
      } else {
        // assume f64 if not specified
        NumericType::Float64
      }
    };

    Ok(Config { cge_path, numeric_type })
  }
}

macro_rules! parse_invocation {
  (
    $attr_stream: ident,
    $item_stream: ident,
    $recurrent:   expr
  ) => {
    {
      crate::macro_core::Invocation {
        config: syn::parse_macro_input!($attr_stream as crate::macro_core::Config),
        item:   syn::parse_macro_input!($item_stream as syn::Item),
        recurrency_constraint: $recurrent
      }

      // let expression = syn::parse_macro_input!($attr_stream as syn::Expr);
      // let string = match expression {
      //   syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::Str(string), .. }) => string.value(),
      //   _ => panic!("Didn't get string literal path to CGE file. Usage: `#[{}(\"path/to/cge/file.cge\")]`", $name),
      // };
    }
  };
}

#[allow(unused_imports)]
pub(crate) use parse_invocation;