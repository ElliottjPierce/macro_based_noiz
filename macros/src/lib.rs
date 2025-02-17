extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use quote::{
    format_ident,
    quote,
};
use syn::{
    Attribute,
    Data,
    DataStruct,
    Expr,
    Field,
    Fields,
    Ident,
    Result,
    Token,
    Type,
    TypeParam,
    Visibility,
    bracketed,
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
    operations: Vec<Operation>,
}

impl Parse for NoiseDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let noise = {
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
        Ok(Self {
            noise,
            input: input_types,
            args,
            operations: parse_many(input),
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
        bracketed!(fields in input);
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
            panic!("Unable to parse a noise operation. Expected a key word like 'data'.");
        }
    }
}

struct ConstructableField {
    field: Field,
    constructor: Expr,
}

/// Creates a noise operation.
#[proc_macro]
pub fn noise_op(_item: TokenStream) -> TokenStream {
    println!("Yay");
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
