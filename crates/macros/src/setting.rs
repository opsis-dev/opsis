use proc_macro2::{Span, TokenStream, TokenTree};
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Ident, LitStr, Path, Token};

pub struct Setting {
    description: Option<LitStr>,
    default: Option<Expr>,
    schema: Vec<TokenStream>,
    extend: Vec<(LitStr, TokenStream)>,
}

impl Setting {
    pub fn attributes(&self) -> TokenStream {
        let default = self
            .default
            .as_ref()
            .map(|value| quote!(#[default(#value)]));
        let mut items: Vec<TokenStream> = Vec::new();
        if let Some(description) = &self.description {
            items.push(quote!(description = #description));
        }
        items.extend(self.schema.iter().cloned());
        if !self.extend.is_empty() {
            let pairs = self.extend.iter().map(|(key, value)| quote!(#key = #value));
            items.push(quote!(extend(#(#pairs),*)));
        }
        let schemars = (!items.is_empty()).then(|| quote!(#[schemars(#(#items),*)]));
        quote!(#default #schemars)
    }

    pub const fn gives_default(&self) -> bool {
        self.default.is_some()
    }

    fn add_extension(&mut self, key: LitStr, value: TokenStream) -> syn::Result<()> {
        if self
            .extend
            .iter()
            .any(|(seen, _)| seen.value() == key.value())
        {
            return Err(syn::Error::new(
                key.span(),
                format!("`{}` is given twice", key.value()),
            ));
        }
        self.extend.push((key, value));
        Ok(())
    }
}

impl Parse for Setting {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut setting = Self {
            description: None,
            default: None,
            schema: Vec::new(),
            extend: Vec::new(),
        };
        if input.peek(LitStr) && !input.peek2(Token![=]) {
            setting.description = Some(input.parse()?);
            comma(input)?;
        }
        while !input.is_empty() {
            if input.peek(LitStr) {
                let key: LitStr = input.parse()?;
                input.parse::<Token![=]>()?;
                let value = value_tokens(input)?;
                setting.add_extension(key, value)?;
            } else {
                let name: Ident = input.parse()?;
                let word = name.to_string();
                match word.as_str() {
                    "default" => {
                        if setting.default.is_some() {
                            return Err(syn::Error::new(name.span(), "`default` is given twice"));
                        }
                        input.parse::<Token![=]>()?;
                        setting.default = Some(input.parse()?);
                    }
                    "range" | "length" => {
                        let content;
                        syn::parenthesized!(content in input);
                        let bounds: TokenStream = content.parse()?;
                        setting.schema.push(quote!(#name(#bounds)));
                    }
                    "transform" => {
                        input.parse::<Token![=]>()?;
                        let path: Path = input.parse()?;
                        setting.schema.push(quote!(transform = #path));
                    }
                    "check" => {
                        input.parse::<Token![=]>()?;
                        let value = check_value(input)?;
                        setting.add_extension(extension_key(&name), value)?;
                    }
                    _ => {
                        let value = if input.peek(Token![=]) {
                            input.parse::<Token![=]>()?;
                            value_tokens(input)?
                        } else {
                            quote!(true)
                        };
                        setting.add_extension(extension_key(&name), value)?;
                    }
                }
            }
            comma(input)?;
        }
        Ok(setting)
    }
}

fn comma(input: ParseStream) -> syn::Result<()> {
    if !input.is_empty() {
        input.parse::<Token![,]>()?;
    }
    Ok(())
}

fn extension_key(name: &Ident) -> LitStr {
    LitStr::new(
        &format!("x-{}", name.to_string().replace('_', "-")),
        name.span(),
    )
}

fn check_value(input: ParseStream) -> syn::Result<TokenStream> {
    if !input.peek(Ident) {
        return value_tokens(input);
    }
    let rule: Ident = input.parse()?;
    match rule.to_string().as_str() {
        "colour" => Ok(colour_rule("colour", rule.span())),
        "colour_or_empty" => Ok(colour_rule("colour-or-empty", rule.span())),
        other => Err(syn::Error::new(
            rule.span(),
            format!("`{other}` is not a check; use `colour`, `colour_or_empty` or a rule object"),
        )),
    }
}

fn colour_rule(rule: &str, span: Span) -> TokenStream {
    let rule = LitStr::new(rule, span);
    quote!({ "rule": #rule, "message": "Pick a colour" })
}

pub fn value_tokens(input: ParseStream) -> syn::Result<TokenStream> {
    input.step(|cursor| {
        let mut rest = *cursor;
        let mut tokens = TokenStream::new();
        while let Some((tree, next)) = rest.token_tree() {
            if matches!(&tree, TokenTree::Punct(punct) if punct.as_char() == ',') {
                break;
            }
            tree.to_tokens(&mut tokens);
            rest = next;
        }
        if tokens.is_empty() {
            return Err(cursor.error("expected a value"));
        }
        Ok((tokens, rest))
    })
}
