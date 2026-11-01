use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, Fields, Ident, ItemEnum, LitStr, Token, Variant};

pub const CASES: [&str; 8] = [
    "lowercase",
    "UPPERCASE",
    "PascalCase",
    "camelCase",
    "snake_case",
    "SCREAMING_SNAKE_CASE",
    "kebab-case",
    "SCREAMING-KEBAB-CASE",
];

pub struct Options {
    export: bool,
    rename_all: LitStr,
}

impl Parse for Options {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut options = Self {
            export: false,
            rename_all: LitStr::new("kebab-case", input.span()),
        };
        while !input.is_empty() {
            let name: Ident = input.parse()?;
            match name.to_string().as_str() {
                "export" => options.export = true,
                "rename_all" => {
                    input.parse::<Token![=]>()?;
                    let case: LitStr = input.parse()?;
                    if !CASES.contains(&case.value().as_str()) {
                        return Err(syn::Error::new(
                            case.span(),
                            format!("`{}` is not a serde case", case.value()),
                        ));
                    }
                    options.rename_all = case;
                }
                other => {
                    return Err(syn::Error::new(
                        name.span(),
                        format!("`choice` takes `export` and `rename_all`, not `{other}`"),
                    ));
                }
            }
            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(options)
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let options: Options = syn::parse2(attr)?;
    let mut item: ItemEnum = syn::parse2(item)?;
    if let Some(derive) = item.attrs.iter().find(|a| a.path().is_ident("derive")) {
        return Err(syn::Error::new(
            derive.span(),
            "`choice` writes the derives; remove this `derive`",
        ));
    }
    let mut labels = Vec::with_capacity(item.variants.len());
    let mut has_default = false;
    for variant in &mut item.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "a choice is a plain word; this variant carries data",
            ));
        }
        has_default |= variant.attrs.iter().any(|a| a.path().is_ident("default"));
        let label = take_label(variant)?;
        let wire = wire_name(variant, &options.rename_all.value())?;
        labels.push((variant.ident.clone(), wire, label));
    }
    let labelled = labels
        .iter()
        .filter(|(_, _, label)| label.is_some())
        .count();
    if labelled != 0 && labelled != labels.len() {
        let (ident, _, _) = labels
            .iter()
            .find(|(_, _, label)| label.is_none())
            .expect("an unlabelled variant");
        return Err(syn::Error::new(ident.span(), "label every variant or none"));
    }
    let default = has_default.then(|| quote!(Default,));
    let export = options.export.then(|| quote!(, ::specta::Type));
    let case = &options.rename_all;
    let name = &item.ident;
    let (schema, method) = if labelled == 0 {
        (None, None)
    } else {
        let pairs = labels.iter().map(|(_, wire, label)| quote!(#wire: #label));
        let arms = labels
            .iter()
            .map(|(ident, _, label)| quote!(Self::#ident => #label));
        (
            Some(quote!(#[schemars(extend("x-labels" = { #(#pairs),* }))])),
            Some(quote! {
                impl #name {
                    pub fn label(self) -> &'static str {
                        match self {
                            #(#arms),*
                        }
                    }
                }
            }),
        )
    };
    let body = item.into_token_stream();
    Ok(quote! {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            #default
            ::serde::Serialize,
            ::serde::Deserialize,
            ::schemars::JsonSchema
            #export
        )]
        #[serde(rename_all = #case)]
        #schema
        #body
        #method
    })
}

fn take_label(variant: &mut Variant) -> syn::Result<Option<LitStr>> {
    let mut found: Option<LitStr> = None;
    let mut kept: Vec<Attribute> = Vec::with_capacity(variant.attrs.len());
    for attr in std::mem::take(&mut variant.attrs) {
        if !attr.path().is_ident("label") {
            kept.push(attr);
            continue;
        }
        if found.is_some() {
            return Err(syn::Error::new(attr.span(), "a variant takes one `label`"));
        }
        found = Some(attr.parse_args()?);
    }
    variant.attrs = kept;
    Ok(found)
}

fn wire_name(variant: &Variant, case: &str) -> syn::Result<LitStr> {
    for attr in variant.attrs.iter().filter(|a| a.path().is_ident("serde")) {
        let mut renamed: Option<LitStr> = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                renamed = Some(meta.value()?.parse()?);
            } else if meta.input.peek(Token![=]) {
                meta.value()?.parse::<TokenStream>()?;
            }
            Ok(())
        })?;
        if let Some(name) = renamed {
            return Ok(name);
        }
    }
    let ident = variant.ident.to_string();
    Ok(LitStr::new(&renamed(&ident, case), variant.ident.span()))
}

pub fn renamed(variant: &str, case: &str) -> String {
    let snake = snake(variant);
    match case {
        "lowercase" => variant.to_ascii_lowercase(),
        "UPPERCASE" => variant.to_ascii_uppercase(),
        "camelCase" => {
            let mut chars = variant.chars();
            chars.next().map_or_else(String::new, |first| {
                first.to_ascii_lowercase().to_string() + chars.as_str()
            })
        }
        "snake_case" => snake,
        "SCREAMING_SNAKE_CASE" => snake.to_ascii_uppercase(),
        "kebab-case" => snake.replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => snake.replace('_', "-").to_ascii_uppercase(),
        _ => variant.to_owned(),
    }
}

fn snake(variant: &str) -> String {
    let mut out = String::with_capacity(variant.len() + 4);
    for (i, ch) in variant.char_indices() {
        if i > 0 && ch.is_uppercase() {
            out.push('_');
        }
        out.push(ch.to_ascii_lowercase());
    }
    out
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use proptest::prelude::*;

    use super::{CASES, expand, renamed};

    fn error(attr: &str, item: &str) -> String {
        expand(attr.parse().unwrap(), item.parse::<TokenStream>().unwrap())
            .err()
            .unwrap()
            .to_string()
    }

    fn expanded(attr: &str, item: &str) -> String {
        expand(attr.parse().unwrap(), item.parse::<TokenStream>().unwrap())
            .unwrap()
            .to_string()
    }

    macro_rules! serde_names {
        ($($case:literal => $name:ident),* $(,)?) => {
            $(
                #[derive(serde::Serialize, Clone, Copy)]
                #[serde(rename_all = $case)]
                enum $name {
                    LeftToRight,
                    Grid2x2,
                    A,
                    WithURL,
                }
            )*

            #[test]
            fn names_variants_as_serde_does() {
                let variants = ["LeftToRight", "Grid2x2", "A", "WithURL"];
                $(
                    let made: Vec<String> = [$name::LeftToRight, $name::Grid2x2, $name::A, $name::WithURL]
                        .iter()
                        .map(|v| serde_json::to_value(v).unwrap().as_str().unwrap().to_owned())
                        .collect();
                    let ours: Vec<String> = variants.iter().map(|v| renamed(v, $case)).collect();
                    assert_eq!(ours, made, "{}", $case);
                )*
                assert_eq!(CASES.len(), [$($case),*].len());
            }
        };
    }

    serde_names! {
        "lowercase" => Lower,
        "UPPERCASE" => Upper,
        "PascalCase" => Pascal,
        "camelCase" => Camel,
        "snake_case" => Snake,
        "SCREAMING_SNAKE_CASE" => ScreamingSnake,
        "kebab-case" => Kebab,
        "SCREAMING-KEBAB-CASE" => ScreamingKebab,
    }

    proptest! {
        #[test]
        fn labels_every_variant_by_its_name_on_the_wire(labels in prop::collection::vec("[A-Z][a-z ]{0,12}", 1..6)) {
            let variants: Vec<String> = labels
                .iter()
                .enumerate()
                .map(|(i, label)| format!("#[label({label:?})] TwoWords{i}"))
                .collect();
            let out = expanded("", &format!("enum Pick {{ {} }}", variants.join(", ")));
            for (i, label) in labels.iter().enumerate() {
                let pair = format!("\"two-words{i}\" : {label:?}");
                prop_assert!(out.contains(&pair), "{}", out);
                let arm = format!("Self :: TwoWords{i} => {label:?}");
                prop_assert!(out.contains(&arm), "{}", out);
            }
        }
    }

    #[test]
    fn keeps_a_variant_renamed_by_hand() {
        let out = expanded(
            "rename_all = \"snake_case\"",
            "enum Grid { #[serde(rename = \"2x2\")] #[label(\"Small\")] Small, #[label(\"Large one\")] LargeOne }",
        );
        assert!(out.contains("\"2x2\" : \"Small\""), "{out}");
        assert!(out.contains("\"large_one\" : \"Large one\""), "{out}");
    }

    #[test]
    fn derives_default_and_the_type_only_when_asked() {
        let plain = expanded("", "enum Pick { A, B }");
        assert!(!plain.contains("Default") && !plain.contains("specta"));
        assert!(!plain.contains("x-labels") && !plain.contains("fn label"));
        let both = expanded("export", "enum Pick { #[default] A, B }");
        assert!(both.contains("Default") && both.contains(":: specta :: Type"));
    }

    #[test]
    fn explains_what_it_cannot_take() {
        assert_eq!(
            error("", "enum Pick { #[label(\"A\")] A, B }"),
            "label every variant or none"
        );
        assert_eq!(
            error("", "#[derive(Debug)] enum Pick { A }"),
            "`choice` writes the derives; remove this `derive`"
        );
        assert_eq!(
            error("", "enum Pick { A(u32) }"),
            "a choice is a plain word; this variant carries data"
        );
        assert_eq!(
            error("rename_all = \"Title Case\"", "enum Pick { A }"),
            "`Title Case` is not a serde case"
        );
        assert_eq!(
            error("hidden", "enum Pick { A }"),
            "`choice` takes `export` and `rename_all`, not `hidden`"
        );
        assert_eq!(
            error("", "enum Pick { #[label(\"A\")] #[label(\"B\")] A }"),
            "a variant takes one `label`"
        );
    }
}
