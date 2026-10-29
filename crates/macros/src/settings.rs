use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::parse::Parser;
use syn::spanned::Spanned;
use syn::{Attribute, ItemStruct};

use crate::setting::Setting;

pub fn expand(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    if !attr.is_empty() {
        return Err(syn::Error::new(
            attr.span(),
            "`settings` takes no arguments",
        ));
    }
    let mut item: ItemStruct = syn::parse2(item)?;
    for field in &mut item.fields {
        field.attrs = field_attributes(std::mem::take(&mut field.attrs))?;
    }
    Ok(item.into_token_stream())
}

fn field_attributes(attrs: Vec<Attribute>) -> syn::Result<Vec<Attribute>> {
    let settings = attrs.iter().filter(|attr| attr.path().is_ident("setting"));
    if let Some(second) = settings.clone().nth(1) {
        return Err(syn::Error::new(
            second.span(),
            "a field takes one `setting`",
        ));
    }
    let has_default = attrs.iter().any(|attr| attr.path().is_ident("default"));
    let mut out = Vec::with_capacity(attrs.len() + 1);
    for attr in attrs {
        if !attr.path().is_ident("setting") {
            out.push(attr);
            continue;
        }
        let setting: Setting = attr.parse_args()?;
        if has_default && setting.gives_default() {
            return Err(syn::Error::new(
                attr.span(),
                "the default is given both in `setting` and in `#[default]`",
            ));
        }
        out.extend(Attribute::parse_outer.parse2(setting.attributes())?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use proc_macro2::TokenStream;
    use proptest::prelude::*;
    use quote::ToTokens;
    use syn::{Attribute, Ident, ItemStruct, LitStr, Token};

    use super::expand;
    use crate::setting::value_tokens;

    #[derive(Debug, Clone)]
    enum Value {
        Flag,
        Number(u32),
        Text(String),
        Object(u32),
    }

    impl Value {
        fn written(&self) -> String {
            match self {
                Value::Flag => String::new(),
                Value::Number(n) => format!(" = {n}"),
                Value::Text(text) => format!(" = {text:?}"),
                Value::Object(n) => format!(" = {{ \"at\": {n} }}"),
            }
        }

        fn expected(&self) -> String {
            match self {
                Value::Flag => "true".into(),
                Value::Number(n) => n.to_string(),
                Value::Text(text) => format!("{text:?}"),
                Value::Object(n) => format!("{{ \"at\" : {n} }}"),
            }
        }
    }

    fn name() -> impl Strategy<Value = String> {
        "[a-z]{1,6}(_[a-z]{1,6}){0,2}".prop_filter("a plain word", |name| {
            syn::parse_str::<Ident>(name).is_ok()
                && !["default", "range", "length", "transform", "check"].contains(&name.as_str())
        })
    }

    fn value() -> impl Strategy<Value = Value> {
        prop_oneof![
            Just(Value::Flag),
            any::<u32>().prop_map(Value::Number),
            "[A-Za-z ,.]{0,16}".prop_map(Value::Text),
            any::<u32>().prop_map(Value::Object),
        ]
    }

    fn extensions() -> impl Strategy<Value = Vec<(String, Value)>> {
        prop::collection::vec((name(), value()), 0..6).prop_map(|list| {
            let mut seen = std::collections::HashSet::new();
            list.into_iter()
                .filter(|(name, _)| seen.insert(name.clone()))
                .collect()
        })
    }

    struct Schemars {
        description: Option<String>,
        extend: Vec<(String, String)>,
    }

    fn read_schemars(attr: &Attribute) -> Schemars {
        let mut found = Schemars {
            description: None,
            extend: Vec::new(),
        };
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("description") {
                found.description = Some(meta.value()?.parse::<LitStr>()?.value());
            } else if meta.path.is_ident("extend") {
                let content;
                syn::parenthesized!(content in meta.input);
                while !content.is_empty() {
                    let key: LitStr = content.parse()?;
                    content.parse::<Token![=]>()?;
                    let value = value_tokens(&content)?.to_string();
                    found.extend.push((key.value(), value));
                    if !content.is_empty() {
                        content.parse::<Token![,]>()?;
                    }
                }
            }
            Ok(())
        })
        .unwrap();
        found
    }

    fn expanded(field: &str) -> syn::Result<Vec<Attribute>> {
        let item: TokenStream = format!("struct Settings {{ {field} value: u32 }}")
            .parse()
            .unwrap();
        let out: ItemStruct = syn::parse2(expand(TokenStream::new(), item)?)?;
        Ok(out.fields.into_iter().next().unwrap().attrs)
    }

    fn error(field: &str) -> String {
        expanded(field).err().unwrap().to_string()
    }

    proptest! {
        #[test]
        fn writes_every_part_of_a_setting_in_place_and_in_order(
            label in proptest::option::of("[A-Z][a-z ]{0,20}"),
            default in proptest::option::of(any::<u32>()),
            list in extensions(),
        ) {
            let mut items: Vec<String> = Vec::new();
            items.extend(label.iter().map(|label| format!("{label:?}")));
            items.extend(default.iter().map(|value| format!("default = {value}")));
            items.extend(list.iter().map(|(name, value)| format!("{name}{}", value.written())));
            let attrs = expanded(&format!(
                "#[serde(rename = \"kept\")] #[setting({})] #[specta(optional)]",
                items.join(", ")
            ))
            .unwrap();
            let names: Vec<String> = attrs
                .iter()
                .map(|attr| attr.path().to_token_stream().to_string())
                .collect();
            let mut expected = vec!["serde".to_string()];
            if default.is_some() {
                expected.push("default".into());
            }
            if label.is_some() || !list.is_empty() {
                expected.push("schemars".into());
            }
            expected.push("specta".into());
            prop_assert_eq!(&names, &expected);
            if let Some(value) = default {
                let attr = attrs.iter().find(|attr| attr.path().is_ident("default")).unwrap();
                let written = attr.parse_args::<syn::LitInt>().unwrap();
                prop_assert_eq!(written.base10_digits(), value.to_string());
            }
            if let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident("schemars")) {
                let schemars = read_schemars(attr);
                prop_assert_eq!(schemars.description, label);
                let wanted: Vec<(String, String)> = list
                    .iter()
                    .map(|(name, value)| (format!("x-{}", name.replace('_', "-")), value.expected()))
                    .collect();
                prop_assert_eq!(schemars.extend, wanted);
            }
        }
    }

    #[test]
    fn explains_what_it_cannot_take() {
        assert_eq!(
            error("#[setting(\"A\", help = \"x\", help = \"y\")]"),
            "`x-help` is given twice"
        );
        assert_eq!(
            error("#[setting(default = 1, default = 2)]"),
            "`default` is given twice"
        );
        assert_eq!(
            error("#[setting(check = red)]"),
            "`red` is not a check; use `colour`, `colour_or_empty` or a rule object"
        );
        assert_eq!(
            error("#[setting(\"A\")] #[setting(\"B\")]"),
            "a field takes one `setting`"
        );
        assert_eq!(
            error("#[default = 1] #[setting(default = 2)]"),
            "the default is given both in `setting` and in `#[default]`"
        );
    }
}
