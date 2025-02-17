extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{
    format_ident,
    quote,
};
use syn::{
    Attribute,
    Error,
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
    operations: Vec<Operation>,
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
                data: Vec::new(),
            }
        };
        _ = input.parse::<Token![for]>()?;
        let input_types = input.parse()?;
        _ = input.parse::<Token![=]>()?;
        let args = input.parse()?;

        _ = input.parse::<Token![impl]>()?;
        let operations = parse_many::<Operation>(input);
        for op in &operations {
            op.store_fields(&mut noise.data);
        }
        Ok(Self {
            noise,
            input: input_types,
            args,
            operations,
        })
    }
}

struct FullStruct {
    name: Ident,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    data: Vec<Field>,
}

impl Parse for FullStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let visibility = input.parse()?;
        _ = input.parse::<Token![struct]>()?;
        let name = input.parse()?;
        let fields;
        braced!(fields in input);
        let data = parse_many_with(&fields, |input| Field::parse_named(input));
        Ok(Self {
            name,
            visibility,
            attributes,
            data,
        })
    }
}

enum Operation {
    // Noise,
    Data(ConstructableField),
    // Convert,
    // Morph,
}

impl Operation {
    fn store_fields(&self, fields: &mut Vec<Field>) {
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
    quote! {}.into()
}

fn parse_many_with<T>(input: ParseStream, parse: fn(&ParseStream) -> Result<T>) -> Vec<T> {
    let mut result = Vec::new();
    loop {
        match parse(&input) {
            Ok(val) => result.push(val),
            Err(_) => break,
        }
    }
    result
}

fn parse_many<T: Parse>(input: ParseStream) -> Vec<T> {
    parse_many_with(input, |input| input.parse::<T>())
}
