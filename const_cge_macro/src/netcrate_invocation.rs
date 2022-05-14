pub struct NetcrateInvocation {
  pub name: syn::Ident,
  pub path: String
}

impl syn::parse::Parse for NetcrateInvocation {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let name  = input.parse::<syn::Ident>()
      .expect("Usage: `netcrate!(public_name = \"path/to/file.cge\");`");

    let _ = input.parse::<syn::Token![=]>()
      .expect("Usage: `netcrate!(public_name = \"path/to/file.cge\");`");
    
    let path = input.parse::<syn::LitStr>()
      .expect("Usage: `netcrate!(public_name = \"path/to/file.cge\");`").value();
    
    Ok(NetcrateInvocation { name, path })
  }
}