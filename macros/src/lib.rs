extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{
    ToTokens,
    quote,
};
use syn::{
    Attribute,
    Expr,
    Field,
    FieldMutability,
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
};

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
        let noise_fields = noise.filed_names().into_iter().collect::<Vec<_>>();
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
                    let Self {
                        #(#noise_fields,)*
                    } = self;

                    #(#noise_impl)*

                    input
                }
            }

            impl noiz::noise::Noise for #noise_name {
                type Input = #input;
            }

            impl From<#args_name> for #noise_name {
                fn from(value: #args_name) -> Self {
                    Self::new(value)
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
    Noise(ConstructableField<Token![do]>),
    Data(ConstructableField<Token![let]>),
    // Convert,
    // Morph,
}

impl Operation {
    fn store_fields(&self, fields: &mut Punctuated<Field, Token![,]>) {
        match self {
            Operation::Data(field) => fields.push(field.field()),
            Operation::Noise(field) => fields.push(field.field()),
        }
    }

    fn quote_construction(&self) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(field) => {
                let name = &field.ident;
                let constructor = &field.constructor;
                quote! {let #name = #constructor;}
            }
            Operation::Noise(field) => {
                let name = &field.ident;
                let constructor = &field.constructor;
                quote! {let #name = #constructor;}
            }
        }
    }

    fn quote_noise(&self) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(_) => {
                quote! {}
            }
            Operation::Noise(field) => {
                let name = &field.ident;
                quote! {let input = #name.get(input); }
            }
        }
    }
}

impl Parse for Operation {
    fn parse(input: ParseStream) -> Result<Self> {
        if let Ok(op) = input.parse::<ConstructableField<Token![let]>>() {
            Ok(Self::Data(op))
        } else if let Ok(op) = input.parse::<ConstructableField<Token![do]>>() {
            Ok(Self::Noise(op))
        } else {
            Err(input
                .error("Unable to parse a noise operation. Expected a noise key word like 'let'."))
        }
    }
}

struct ConstructableField<K: Parse> {
    attrs: Vec<Attribute>,
    #[expect(unused, reason = "This makes it easier to parse.")]
    key_word: K,
    vis: Visibility,
    ident: Ident,
    colon: Token![:],
    ty: Type,
    #[expect(unused, reason = "This makes it easier to parse.")]
    eq: Token![=],
    constructor: Expr,
}

impl<K: Parse> ConstructableField<K> {
    fn field(&self) -> Field {
        Field {
            attrs: self.attrs.clone(),
            vis: self.vis.clone(),
            mutability: FieldMutability::None,
            ident: Some(self.ident.clone()),
            colon_token: Some(self.colon.clone()),
            ty: self.ty.clone(),
        }
    }
}

impl<K: Parse> Parse for ConstructableField<K> {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            attrs: Attribute::parse_outer(input)?,
            key_word: input.parse()?,
            vis: input.parse()?,
            ident: input.parse()?,
            colon: input.parse()?,
            ty: input.parse()?,
            eq: input.parse().unwrap_or_default(),
            constructor: input
                .parse()
                .unwrap_or_else(|_| parse_quote! {Default::default()}),
        })
    }
}

/// Creates a noise operation.
#[proc_macro]
pub fn noise_op(input: TokenStream) -> TokenStream {
    let noise = parse_macro_input!(input as NoiseDefinition);
    quote! {#noise}.into()
}
