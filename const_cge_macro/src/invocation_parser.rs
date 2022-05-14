use proc_macro2::Ident;
use syn::{Token, ExprLit, TypePath, Lit, Expr, ExprPath};
use crate::{macro_core::{Config, CgeType}, numeric_type::NumericType};

impl syn::parse::Parse for crate::macro_core::Config {
  fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
    // extract the CGE
    let cge = {
      match input.parse::<Expr>() {
        // -  invoked like #[network("path/to/file.cge")]
        // or invoked like #[network("literal cge data")]
        Ok(Expr::Lit(ExprLit { lit: Lit::Str(string), .. })) => {
          // we must determine if the literal is a valid path, or if it is data.
          // to do this, I will first try to treat it as a path.
          // if that fails, I will try to treat it as data.

          let string = string.value();
          match std::path::PathBuf::try_from(string.clone()) {
            // it parses as a valid path AND it exists
            Ok(p) if p.exists() => CgeType::File(string),

            // it either isn't a valid path, or it could be, but that file doesn't exist.
            // so we will assume it is a `Direct` CGE string
            _ => CgeType::Direct(string)
          }
        },

        // invoked like #[network(ocr_network)]
        Ok(Expr::Path(ExprPath { path, .. })) => CgeType::Module(path),

        // abort compilation with error
        _ => panic!("Expected either a string path to a CGE file, or a module name. Make sure the CGE path string or module path is the first argument, like: `#[network(\"path/to/file.cge\")]` or `#[network(some_netcrate)]`.")
      }
    };

    // manually parse remaining arguments.
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

    Ok(Config { cge, numeric_type })
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
    }
  };
}

#[allow(unused_imports)]
pub(crate) use parse_invocation;