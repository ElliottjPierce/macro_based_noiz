extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{
    ToTokens,
    format_ident,
    quote,
};
use syn::{
    Attribute,
    Expr,
    Field,
    Ident,
    Result,
    Token,
    Type,
    Visibility,
    braced,
    parse::{
        Parse,
        ParseStream,
    },
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token,
};

mod key_words {
    use syn::custom_keyword;

    custom_keyword!(data);
}

struct NoiseDefinition {
    noise: FullStruct,
    input: Type,
    args: FullStruct,
    operations: Punctuated<Operation, Token![;]>,
}

impl Parse for NoiseDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut noise = {
            let attributes = Attribute::parse_outer(input)?;
            let visibility = input.parse()?;
            _ = input.parse::<Token![struct]>()?;
            let name = input.parse()?;
            FullStruct {
                name,
                visibility,
                attributes,
                data: Default::default(),
            }
        };
        _ = input.parse::<Token![for]>()?;
        let input_types = input.parse()?;
        _ = input.parse::<Token![=]>()?;
        let args = input.parse()?;

        _ = input.parse::<Token![impl]>()?;
        let operations = Punctuated::<Operation, Token![;]>::parse_separated_nonempty(input)?;
        for op in operations.iter() {
            op.store_fields(&mut noise.data);
        }
        Ok(Self {
            noise,
            input: input_types,
            args,
            operations: Default::default(),
        })
    }
}

impl ToTokens for NoiseDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let NoiseDefinition {
            noise,
            input,
            args,
            operations,
        } = self;
        let operations = operations.iter();

        tokens.extend(quote! {
            #noise

            #args
        });
    }
}

struct FullStruct {
    name: Ident,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    data: Punctuated<Field, Token![,]>,
}

impl Parse for FullStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let visibility = input.parse()?;
        _ = input.parse::<Token![struct]>()?;
        let name = input.parse()?;
        let fields;
        braced!(fields in input);
        let data =
            Punctuated::parse_separated_nonempty_with(&fields, |input| Field::parse_named(input))?;
        Ok(Self {
            name,
            visibility,
            attributes,
            data,
        })
    }
}

impl ToTokens for FullStruct {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let FullStruct {
            name,
            visibility,
            attributes,
            data,
        } = self;
        let data = data.iter();
        tokens.extend(quote! {
            #(#attributes)*
            #visibility struct #name {
                #(#data,)*
            }
        });
    }
}

enum Operation {
    // Noise,
    Data(ConstructableField),
    // Convert,
    // Morph,
}

impl Operation {
    fn store_fields(&self, fields: &mut Punctuated<Field, Token![,]>) {
        match self {
            Operation::Data(constructable_field) => {
                fields.push(constructable_field.field.clone());
            }
        }
    }
}

impl Parse for Operation {
    fn parse(input: ParseStream) -> Result<Self> {
        let peeker = input.lookahead1();
        if peeker.peek(key_words::data) {
            _ = input.parse::<key_words::data>()?;
            let field = Field::parse_named(input)?;
            _ = input.parse::<Token![=]>()?;
            let constructor = input.parse()?;
            Ok(Self::Data(ConstructableField { field, constructor }))
        } else {
            Err(input.error("Unable to parse a noise operation. Expected a key word like 'data'."))
        }
    }
}

struct ConstructableField {
    field: Field,
    constructor: Expr,
}

/// Creates a noise operation.
#[proc_macro]
pub fn noise_op(input: TokenStream) -> TokenStream {
    let noise = parse_macro_input!(input as NoiseDefinition);
    println!("field len {}", noise.noise.data.len());
    quote! {#noise}.into()
}
