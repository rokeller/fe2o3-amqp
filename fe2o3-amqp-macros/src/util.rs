use darling::FromDeriveInput;
use syn::DeriveInput;

use crate::{AmqpContractAttr, DescribedAttr, EncodingType};

pub(crate) fn parse_described_attr(input: &syn::DeriveInput) -> AmqpContractAttr {
    let attr = DescribedAttr::from_derive_input(&input).unwrap();

    let name = attr.name.unwrap_or_else(|| input.ident.to_string());
    let code = attr.code;
    let encoding = attr.encoding.unwrap_or(EncodingType::List);
    let rename_field = attr.rename_field;
    AmqpContractAttr { name, code, encoding , rename_field}
}

pub(crate) fn convert_to_case(case: &str, source: String, ctx: &DeriveInput) -> Result<String, syn::Error> {
    use convert_case::{Case, Casing};
    let s = match case {
        "" => source, 
        "lowercase" => source.to_lowercase(), 
        "UPPERCASE" => source.to_uppercase(), 
        "PascalCase" => source.to_case(Case::Pascal), 
        "camelCase" => source.to_case(Case::Camel), 
        "snake_case" => source.to_case(Case::Snake), 
        "SCREAMING_SNAKE_CASE" => source.to_case(Case::ScreamingSnake), 
        "kebab-case" => source.to_case(Case::Kebab), 
        e @ _ => {
            let span = ctx.attrs.iter()
                .find_map(|attr| {
                    match attr.path.get_ident() {
                        Some(i) => {
                            if i.to_string() == "rename_all" {
                                Some(i.span())
                            } else {
                                None
                            }
                        },
                        None => None
                    }
                });
            match span {
                Some(span) => return Err(syn::Error::new(span, format!("{} case is not implemented", e))),
                None => return Err(syn::Error::new(ctx.ident.span(), format!("{} case is not implemented", e)))
            }
        }
    };

    Ok(s)
}

