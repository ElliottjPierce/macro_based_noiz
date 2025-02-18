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
    output: Type,
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
        _ = input.parse::<Token![->]>()?;
        let output = input.parse()?;

        _ = input.parse::<Token![=]>()?;
        let args = input.parse()?;

        _ = input.parse::<Token![impl]>()?;
        let operations = Punctuated::<Operation, Token![;]>::parse_terminated(input)?;
        for op in operations.iter() {
            op.store_fields(&mut noise.data);
        }
        Ok(Self {
            noise,
            input: input_types,
            output,
            args,
            operations,
        })
    }
}

impl ToTokens for NoiseDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let NoiseDefinition {
            noise,
            input,
            output,
            args,
            operations,
        } = self;
        let creation = operations.iter().map(Operation::quote_construction);
        let noise_impl = operations.iter().map(Operation::quote_noise);
        let noise_name = &noise.name;
        let args_name = &args.name;
        let noise_fields = noise.filed_names().into_iter();
        let args_fields = args.filed_names().into_iter();

        tokens.extend(quote! {
            #noise

            #args

            impl #noise_name  {
                pub fn new(args: #args_name) -> Self {
                    let #args_name {
                        #(#args_fields,)*
                    } = args;

                    #(#creation)*

                    Self {
                        #(#noise_fields,)*
                    }
                }
            }

            impl noiz::noise::NoiseOp<#input> for #noise_name {
                type Output = #output;

                fn get(&self, input: #input) -> Self::Output{
                    #(#noise_impl)*
                    input
                }
            }
        });
    }
}

struct FullStruct {
    name: Ident,
    visibility: Visibility,
    attributes: Vec<Attribute>,
    data: Punctuated<Field, Token![,]>,
}

impl FullStruct {
    fn filed_names(&self) -> impl IntoIterator<Item = &Ident> {
        self.data
            .iter()
            .map(|field| field.ident.as_ref().expect("Fields must be named."))
    }
}

impl Parse for FullStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let visibility = input.parse()?;
        _ = input.parse::<Token![struct]>()?;
        let name = input.parse()?;
        let fields;
        braced!(fields in input);
        let data = Punctuated::parse_terminated_with(&fields, |input| Field::parse_named(input))?;
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

    fn quote_construction(&self) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(ConstructableField { field, constructor }) => {
                let name = field
                    .ident
                    .as_ref()
                    .expect("Fields must have names in noise operations.");
                quote! {let #name = #constructor;}
            }
        }
    }

    fn quote_noise(&self) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(_) => {
                quote! {}
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
    quote! {#noise}.into()
}
