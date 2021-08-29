use quote::quote;
use syn::DeriveInput;

use crate::{AmqpContractAttr, EncodingType, util::{convert_to_case, parse_described_attr}};

pub(crate) fn expand_serialize(input: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    let amqp_attr = parse_described_attr(input);
    let ident = &input.ident;
    match &input.data {
        syn::Data::Struct(data) => {
            expand_serialize_on_struct(&amqp_attr, ident, data, input)
        },
        _ => unimplemented!()
    }
}

fn expand_serialize_on_struct(
    amqp_attr: &AmqpContractAttr, 
    ident: &syn::Ident, 
    data: &syn::DataStruct,
    ctx: &DeriveInput,
) -> Result<proc_macro2::TokenStream, syn::Error> 
{
    // let ident_name = ident.to_string();
    let descriptor = match amqp_attr.code {
        Some(code) => quote!(fe2o3_amqp::types::Descriptor::Code(#code)),
        None => {
            let name = &amqp_attr.name[..];
            quote!(fe2o3_amqp::types::Descriptor::Name(fe2o3_amqp::types::Symbol::from(#name)))
        }
    };
    let struct_name = match amqp_attr.encoding {
        EncodingType::Basic => quote!(fe2o3_amqp::constants::DESCRIBED_BASIC),
        EncodingType::List => quote!(fe2o3_amqp::constants::DESCRIBED_LIST),
        EncodingType::Map => quote!(fe2o3_amqp::constants::DESCRIBED_MAP)
    };
    let field_idents: Vec<syn::Ident> = data.fields.iter().map(|f| f.ident.clone().unwrap()).collect();
    let field_names: Vec<String> = field_idents.iter()
        .map(|i| 
            convert_to_case(&amqp_attr.rename_field, i.to_string(), ctx
        ).unwrap()).collect();
    let len = field_idents.len();
    let token = quote! {
        #[automatically_derived]
        impl fe2o3_amqp::serde::ser::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: fe2o3_amqp::serde::ser::Serializer,
            {
                use fe2o3_amqp::serde::ser::SerializeStruct;
                // len + 1 for compatibility with other serializer
                let mut state = serializer.serialize_struct(#struct_name, #len + 1)?;
                // serialize descriptor
                // descriptor does not count towards number of element in list
                // in fe2o3_amqp serializer, this will be deducted
                state.serialize_field(fe2o3_amqp::constants::DESCRIPTOR, &#descriptor)?; 
                #( state.serialize_field(#field_names, &self.#field_idents)?; )*
                state.end()
            }
        }
    };
    Ok(token)
}