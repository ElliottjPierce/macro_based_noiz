extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{
    ToTokens,
    quote,
};
use syn::{
    Attribute,
    Block,
    Expr,
    Field,
    FieldMutability,
    Ident,
    Result,
    Token,
    Type,
    Visibility,
    braced,
    parenthesized,
    parse::{
        Parse,
        ParseStream,
        discouraged::Speculative,
    },
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token::Paren,
};

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
        let mut noise_ops = 0u32;

        let operations = {
            let mut punctuated = Punctuated::new();

            loop {
                if input.is_empty() {
                    break;
                }
                let value = Operation::parse(input, &mut noise_ops)?;
                punctuated.push_value(value);
                if input.is_empty() {
                    break;
                }
                let punct = input.parse()?;
                punctuated.push_punct(punct);
            }

            punctuated
        };
        for op in operations.iter() {
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

impl ToTokens for NoiseDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let NoiseDefinition {
            noise,
            input,
            args,
            operations,
        } = self;
        let creation = operations.iter().map(Operation::quote_construction);
        let noise_name = &noise.name;
        let args_name = &args.name;
        let noise_fields = noise.filed_names().into_iter().collect::<Vec<_>>();
        let args_fields = args.filed_names().into_iter();

        let mut noise_impl = Vec::new();
        let mut last_type = input.clone();
        for op in operations.iter() {
            noise_impl.push(op.quote_noise(&mut last_type));
        }

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
                type Output = #last_type;

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
    Convert(ConversionChain),
    Morph(Morph),
}

impl Operation {
    fn store_fields(&self, fields: &mut Punctuated<Field, Token![,]>) {
        match self {
            Operation::Data(field) => fields.push(field.field()),
            Operation::Noise(field) => fields.push(field.field()),
            Operation::Convert(_) | Operation::Morph(_) => {}
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
            Operation::Convert(_) | Operation::Morph(_) => quote! {},
        }
    }

    fn quote_noise(&self, source_type: &mut Type) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(_) => {
                quote! {}
            }
            Operation::Noise(field) => {
                let noise_type = &field.ty;
                *source_type =
                    parse_quote!(<#noise_type as noiz::noise::NoiseOp<#source_type>>::Output);
                let name = &field.ident;
                quote! {let input: #source_type = #name.get(input); }
            }
            Operation::Convert(conversions) => {
                *source_type = conversions.conversions.last().unwrap().clone();
                let conversions = conversions.conversions.iter();
                quote! {
                    #(let input = input.adapt::<#conversions>();)*
                    let input: #source_type = input;
                }
            }
            Operation::Morph(morph) => {
                let block = &morph.block;
                let input_name = &morph.input_name;
                if morph
                    .input_type
                    .as_ref()
                    .is_some_and(|input_type| input_type.ne(source_type))
                {
                    panic!("Morph block has different input type that what is passed into it.")
                }

                *source_type = morph.output.clone();

                let input = if morph.mutable {
                    quote! {let mut #input_name = input;}
                } else {
                    quote! {let #input_name = input;}
                };
                quote! {
                    #input
                    let input: #source_type  = #block;
                }
            }
        }
    }

    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        *noise_amount += 1;
        if let Ok(op) = ConstructableField::<Token![let]>::parse(input, *noise_amount) {
            Ok(Self::Data(op))
        } else if let Ok(op) = ConstructableField::<Token![do]>::parse(input, *noise_amount) {
            Ok(Self::Noise(op))
        } else if let Ok(_is_converter) = input.parse::<Token![as]>() {
            let conversions = Punctuated::parse_separated_nonempty(input)?;
            Ok(Self::Convert(ConversionChain { conversions }))
        } else if let Ok(op) = input.parse::<Morph>() {
            Ok(Self::Morph(op))
        } else {
            Err(input.error(
                "Unable to parse a noise operation. Expected a noise key word like 'let', 'do', \
                 'as', or 'morph'.",
            ))
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

    fn parse_named_no_constructor(input: ParseStream) -> Result<(Self, ParseStream)> {
        Ok((
            Self {
                attrs: Attribute::parse_outer(input)?,
                key_word: input.parse()?,
                vis: input.parse()?,
                ident: input.parse()?,
                colon: input.parse()?,
                ty: input.parse()?,
                eq: Default::default(),
                constructor: parse_quote! {Default::default()},
            },
            input,
        ))
    }

    fn parse_unnamed_no_constructor(
        input: ParseStream,
        ident_hint: u32,
    ) -> Result<(Self, ParseStream)> {
        Ok((
            Self {
                attrs: Attribute::parse_outer(input)?,
                key_word: input.parse()?,
                vis: input.parse()?,
                ident: Ident::new(&format!("val{ident_hint}"), input.span()),
                colon: Default::default(),
                ty: input.parse()?,
                eq: Default::default(),
                constructor: parse_quote! {Default::default()},
            },
            input,
        ))
    }

    fn parse(input: ParseStream, ident_hint: u32) -> Result<Self> {
        let name_fork = input.fork();
        let unnamed_fork = input.fork();
        Self::parse_named_no_constructor(&name_fork)
            .or_else(|_| Self::parse_unnamed_no_constructor(&unnamed_fork, ident_hint))
            .and_then(|(mut result, fork)| {
                input.advance_to(fork);

                if let Ok(_found_custom_constructor) = input.parse::<Token![=]>() {
                    result.constructor = input.parse::<Expr>()?;
                }

                Ok(result)
            })
    }
}

struct ConversionChain {
    conversions: Punctuated<Type, Token![,]>,
}

struct Morph {
    name: Option<Ident>,
    mutable: bool,
    input_name: Ident,
    input_type: Option<Type>,
    output: Type,
    block: Block,
}

impl Parse for Morph {
    fn parse(input: ParseStream) -> Result<Self> {
        _ = input.parse::<Token![fn]>()?;
        let name = input.parse::<Ident>().ok();
        let (input_name, mutable, input_type) = if input.peek(Paren) {
            let params;
            parenthesized!(params in input);
            let mutable = params.parse::<Token![mut]>().is_ok();
            let input_name = params
                .parse()
                .unwrap_or_else(|_| Ident::new("input", params.span()));
            let input_type = if params.parse::<Token![:]>().is_ok() {
                Some(params.parse::<Type>()?)
            } else {
                None
            };
            (input_name, mutable, input_type)
        } else {
            (Ident::new("input", input.span()), false, None)
        };
        _ = input.parse::<Token![->]>()?;
        Ok(Self {
            name,
            mutable,
            input_name,
            input_type,
            output: input.parse()?,
            block: input.parse()?,
        })
    }
}

/// Creates a noise operation.
#[proc_macro]
pub fn noise_op(input: TokenStream) -> TokenStream {
    let noise = parse_macro_input!(input as NoiseDefinition);
    quote! {#noise}.into()
}
