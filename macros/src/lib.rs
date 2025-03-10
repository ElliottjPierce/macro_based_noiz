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
    LitInt,
    Local,
    Result,
    Stmt,
    Token,
    Type,
    Visibility,
    braced,
    bracketed,
    parse::{
        Parse,
        ParseStream,
        discouraged::Speculative,
    },
    parse_macro_input,
    parse_quote,
    punctuated::Punctuated,
    token::{
        Brace,
        Colon,
        Eq,
    },
};

struct NoiseDefinition {
    noise: FullStruct,
    input: Type,
    output: Type,
    source: NoiseSource,
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
                data: Default::default(),
            }
        };

        _ = input.parse::<Token![for]>()?;
        let input_types = input.parse()?;
        _ = input.parse::<Token![->]>()?;
        let output = input.parse()?;

        _ = input.parse::<Token![=]>()?;
        let source = input.parse()?;

        _ = input.parse::<Token![impl]>()?;
        let mut noise_count = 0u32;
        let operations = Operation::parse_many(input, &mut noise_count)?;
        for op in operations.iter() {
            op.store_fields(&mut noise.data);
        }
        Ok(Self {
            noise,
            input: input_types,
            output,
            source,
            operations,
        })
    }
}

impl ToTokens for NoiseDefinition {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let NoiseDefinition {
            noise,
            input,
            source,
            operations,
            output,
        } = self;
        let creation = operations
            .iter()
            .map(|op| op.quote_construction(&noise.name));
        let noise_name = &noise.name;
        let noise_fields = noise.filed_names().into_iter().collect::<Vec<_>>();

        let mut noise_impl = Vec::new();
        for op in operations.iter() {
            noise_impl.push(op.quote_noise());
        }

        let source = source.quote_source(noise_name, creation, noise_fields.iter().copied());

        tokens.extend(quote! {
            #noise

            #source

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
        });
    }
}

enum NoiseSource {
    Custom(FullStruct),
    Existing(Type),
    RawParams(Punctuated<Field, Token![,]>),
}

impl NoiseSource {
    fn quote_source<'b>(
        &self,
        noise_name: &Ident,
        creation: impl Iterator<Item = proc_macro2::TokenStream>,
        noise_fields: impl Iterator<Item = &'b Ident>,
    ) -> proc_macro2::TokenStream {
        match self {
            NoiseSource::Custom(args) => {
                let args_name = &args.name;
                let args_fields = args.filed_names().into_iter().collect::<Vec<_>>();
                let args_params = args.filed_names_and_types();
                quote! {
                    #args

                    impl #noise_name  {
                        pub fn new(#args_params) -> Self {
                            #(#creation)*

                            Self {
                                #(#noise_fields,)*
                            }
                        }
                    }

                    impl From<#args_name> for #noise_name {
                        fn from(value: #args_name) -> Self {
                            let #args_name {
                                #(#args_fields,)*
                            } = value;
                            Self::new(#(#args_fields,)*)
                        }
                    }
                }
            }
            NoiseSource::Existing(existing) => {
                quote! {

                    impl #noise_name  {
                        pub fn new(mut args: #existing) -> Self {
                            #(#creation)*

                            Self {
                                #(#noise_fields,)*
                            }
                        }
                    }

                    impl From<#existing> for #noise_name {
                        fn from(value: #existing) -> Self {
                            Self::new(value)
                        }
                    }
                }
            }
            NoiseSource::RawParams(params) => {
                let params = params.iter();
                quote! {
                    impl #noise_name  {
                        pub fn new(#(mut #params),*) -> Self {
                            #(#creation)*

                            Self {
                                #(#noise_fields,)*
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Parse for NoiseSource {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.peek(Brace) {
            let outer = input;
            let input;
            _ = braced!(input in outer);
            let params =
                Punctuated::parse_terminated_with(&input, |input| Field::parse_named(input))?;
            return Ok(Self::RawParams(params));
        }

        if let Ok(custom) = input.parse::<FullStruct>() {
            Ok(Self::Custom(custom))
        } else if let Ok(existing) = input.parse::<Type>() {
            Ok(Self::Existing(existing))
        } else {
            panic!(
                "Unexpected noise source. Must be a non-generic struct declaration, parameter \
                 names in braces, or the name of another type."
            );
        }
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

    fn filed_names_and_types(&self) -> proc_macro2::TokenStream {
        let params = self.data.iter().map(|field| {
            let name = field.ident.as_ref().expect("Fields must be named.");
            let ty = &field.ty;
            quote! {mut #name: #ty}
        });
        quote! { #(#params),* }
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

#[derive(Clone)]
struct Mapping {
    operation: Box<Operation>,
    mapped: Type,
}

#[derive(Clone)]
struct FbmOp {
    attrs: Vec<Attribute>,
    fbm_ident: Ident,
    settings_constructor: Expr,
    accumulator_constructor: Expr,
    octaves: Vec<FbmOctave>,
}

#[derive(Clone)]
struct FbmOctave {
    attrs: Vec<Attribute>,
    storage_ident: Ident,
    octave_ident: Ident,
    octave_const_ident: Ident,
    octave_storage: Type,
    octave_constructor: Expr,
    ops: Vec<Operation>,
}

impl FbmOctave {
    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        let _kw = input.parse::<Token![where]>()?;
        let octave_ident = input.parse()?;
        let _kw = input.parse::<Token![:]>()?;
        let octave_storage = input.parse()?;
        let _kw = input.parse::<Token![as]>()?;
        let octave_constructor = input.parse()?;
        let _kw = input.parse::<Token![impl]>()?;

        let ops_stream;
        let _ = braced!(ops_stream in input);
        let ops = Operation::parse_many(&ops_stream, noise_amount)?;

        Ok(Self {
            octave_storage,
            storage_ident: Ident::new(&format!("octave_storage_{0}", *noise_amount), input.span()),
            octave_const_ident: Ident::new(
                &format!("octave_constructor_{0}", *noise_amount),
                input.span(),
            ),
            attrs,
            octave_ident,
            octave_constructor,
            ops,
        })
    }
}

impl FbmOp {
    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        let _kw = input.parse::<Token![loop]>()?;
        let accumulator_constructor = input.parse()?;
        let _kw = input.parse::<Token![where]>()?;
        let attrs = Attribute::parse_outer(input)?;
        let fbm_ident = input.parse()?;
        let _kw = input.parse::<Token![=]>()?;
        let settings_constructor = input.parse()?;
        let _kw = input.parse::<Token![enum]>()?;

        let octaves_stream;
        let _ = bracketed!(octaves_stream in input);
        let mut octaves = Vec::new();
        while !octaves_stream.is_empty() {
            let repeat = match octaves_stream.parse::<LitInt>() {
                Ok(lit) => lit.base10_parse::<u8>()?,
                Err(_) => 1,
            };

            if repeat == 0 {
                continue;
            }

            let additional = repeat - 1;
            for _ in 0..additional {
                *noise_amount += 1;
                let noise = FbmOctave::parse(&octaves_stream.fork(), noise_amount)?;
                octaves.push(noise);
            }

            *noise_amount += 1;
            let noise = FbmOctave::parse(&octaves_stream, noise_amount)?;
            octaves.push(noise);

            if octaves_stream.parse::<Token![,]>().is_err() {
                break;
            }
        }
        Ok(Self {
            attrs,
            fbm_ident,
            settings_constructor,
            accumulator_constructor,
            octaves,
        })
    }
}

#[derive(Clone)]
struct RefOp {
    attrs: Vec<Attribute>,
    ident: Ident,
    refer: Expr,
    ops: Vec<Operation>,
}

impl RefOp {
    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        _ = input.parse::<Token![ref]>()?;
        let attrs = Attribute::parse_outer(input)?;
        let ident = input.parse()?;

        let refer = if input.peek(Token![impl]) {
            parse_quote!(input)
        } else {
            _ = input.parse::<Token![=]>()?;
            input.parse()?
        };

        _ = input.parse::<Token![impl]>()?;

        let ops = if input.peek(Brace) {
            let inner;
            _ = braced!(inner in input);
            Operation::parse_many(&inner, noise_amount)?
        } else {
            vec![Operation::parse(input, noise_amount)?]
        };
        Ok(Self {
            attrs,
            ident,
            refer,
            ops,
        })
    }
}

#[derive(Clone)]
enum Operation {
    Data(ConstructableField<Token![use]>),
    Noise(ConstructableField<Token![fn]>),
    Convert(ConversionChain),
    Morph(Morph),
    Hold(Local),
    Parallel(Box<Operation>),
    ConstructionVariable(Local),
    Mapping(Mapping),
    Fbm(FbmOp),
    RefOp(RefOp),
}

impl Operation {
    fn parse_many(input: ParseStream, noise_amount: &mut u32) -> Result<Vec<Self>> {
        let mut operations = Vec::new();
        loop {
            if input.is_empty() {
                break;
            }
            let value = Operation::parse(input, noise_amount)?;
            let needs_semi_colon = value.needs_following_semi_colon() && !input.is_empty();
            operations.push(value);
            if needs_semi_colon || input.peek(Token![;]) {
                _ = input.parse::<Token![;]>()?;
            }
        }
        Ok(operations)
    }

    fn needs_following_semi_colon(&self) -> bool {
        match self {
            Operation::Noise(_)
            | Operation::Fbm(_)
            | Operation::RefOp(_)
            | Operation::Convert(_)
            | Operation::Data(_) => true,
            Operation::ConstructionVariable(_) | Operation::Hold(_) => false,
            Operation::Morph(morph) => !matches!(&morph.block, Expr::Block(_) | Expr::TryBlock(_)),
            Operation::Parallel(op) => op.needs_following_semi_colon(),
            Operation::Mapping(mapping) => mapping.operation.needs_following_semi_colon(),
        }
    }

    fn store_fields(&self, fields: &mut Punctuated<Field, Token![,]>) {
        match self {
            Operation::Data(field) => fields.push(field.field()),
            Operation::Noise(field) => fields.push(field.field()),
            Operation::Fbm(fbm) => {
                fbm.octaves
                    .iter()
                    .flat_map(|octave| octave.ops.iter())
                    .for_each(|op| op.store_fields(fields));
                fbm.octaves.iter().for_each(|octave| {
                    fields.push(Field {
                        attrs: octave.attrs.clone(),
                        vis: Visibility::Inherited,
                        mutability: FieldMutability::None,
                        ident: Some(octave.storage_ident.clone()),
                        colon_token: Default::default(),
                        ty: octave.octave_storage.clone(),
                    });
                });
            }
            Operation::Parallel(op) => op.store_fields(fields),
            Operation::Mapping(mapping) => mapping.operation.store_fields(fields),
            Operation::RefOp(ref_op) => ref_op.ops.iter().for_each(|op| op.store_fields(fields)),
            _ => {}
        }
    }

    fn quote_construction(&self, root_name: &Ident) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(field) => field.quote_constructor(),
            Operation::Noise(field) => field.quote_constructor(),
            Operation::Fbm(FbmOp {
                attrs,
                fbm_ident,
                settings_constructor,
                accumulator_constructor: _,
                octaves,
            }) => {
                let constructing_octaves = octaves.iter().map(
                    |FbmOctave {
                         attrs: _,
                         storage_ident: _,
                         octave_ident: _,
                         octave_storage: _,
                         octave_const_ident,
                         octave_constructor,
                         ops: _,
                     }| {
                        quote! {
                            let mut #octave_const_ident = #octave_constructor;
                            #octave_const_ident.post_construction(&mut #fbm_ident);
                        }
                    },
                );

                let finalize_octaves = octaves.iter().map(
                    |FbmOctave {
                         attrs: _,
                         storage_ident,
                         octave_ident,
                         octave_storage: _,
                         octave_const_ident,
                         octave_constructor: _,
                         ops,
                     }| {
                        let ops_construction = ops.iter().map(|op| op.quote_construction(root_name));
                        quote! {
                            let (#storage_ident, mut #octave_ident) = #octave_const_ident.finalize(&#fbm_ident);
                            #(#ops_construction)*
                        }
                    },
                );

                quote! {
                    use noiz::noise::fbm::Octave as _;
                    use noiz::noise::fbm::Settings as _;
                    #(#attrs)*
                    let mut #fbm_ident = #settings_constructor;
                    #(#constructing_octaves)*
                    #(#finalize_octaves)*
                }
            }
            Operation::ConstructionVariable(binding) => binding.to_token_stream(),
            Operation::Parallel(op) => op.quote_construction(root_name),
            Operation::Mapping(mapping) => mapping.operation.quote_construction(root_name),
            Operation::RefOp(ref_op) => {
                let ops = ref_op.ops.iter().map(|op| op.quote_construction(root_name));
                quote! {#(#ops)*}
            }
            _ => quote! {},
        }
    }

    fn quote_noise(&self) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(_) | Operation::ConstructionVariable(_) => {
                quote! {}
            }
            Operation::Noise(field) => {
                let name = &field.ident;
                quote! {let mut input = #name.get(input); }
            }
            Operation::Convert(conversions) => {
                if conversions.conversions.is_empty() {
                    return quote! {};
                }

                let final_type = conversions.conversions.last().unwrap();
                let conversions = conversions.conversions.iter();
                quote! {
                    let input: #final_type = noiz::noise::convert!(input => #(#conversions),*);
                }
            }
            Operation::Morph(morph) => {
                let block = &morph.block;
                let input_name = &morph.input_name;
                let input = if morph.mutable {
                    quote! {let mut #input_name = input;}
                } else {
                    quote! {let #input_name = input;}
                };
                quote! {
                    #[allow(unused)]
                    #input
                    let mut input = #block;
                }
            }
            Operation::Hold(local) => local.to_token_stream(),
            Operation::Parallel(op) => {
                let op_code = op.quote_noise();
                quote! {
                    let mut input = input.map(|input| {
                        #op_code
                        input
                    });
                }
            }
            Operation::Mapping(Mapping { operation, mapped }) => {
                let op = operation.quote_noise();
                quote! {
                    let mut input = noiz::noise::associating::AssociationMapping::<#mapped>::map_association(input, |input| {
                        #op
                        input
                    });
                }
            }
            Operation::Fbm(FbmOp {
                attrs: _,
                fbm_ident,
                settings_constructor: _,
                accumulator_constructor,
                octaves,
            }) => {
                let mut octaves = octaves.iter();

                let Some(first) = octaves.next() else {
                    return quote! {};
                };

                let num_octaves = octaves.len();
                let first_noise = first.ops.iter().map(|op| op.quote_noise());
                let first_storage = &first.storage_ident;
                let first = quote! {
                    {
                        let mut input = #fbm_ident;
                        #(#first_noise)*
                        __fbm_acc = noiz::noise::fbm::PreAccumulator::<_, _, #num_octaves>::start_accumulate(__fbm_acc_start, input, #first_storage);
                    }
                };

                let octaves = octaves.map(
                    |FbmOctave {
                         attrs: _,
                         storage_ident,
                         octave_ident: _,
                         octave_const_ident: _,
                         octave_storage: _,
                         octave_constructor: _,
                         ops,
                     }| {
                        let ops_noise = ops.iter().map(|op| op.quote_noise());
                        quote! {
                            {
                                let mut input = #fbm_ident;
                                #(#ops_noise)*
                                __fbm_acc.accumulate(input, #storage_ident);
                            }
                        }
                    },
                );

                quote! {
                    use noiz::noise::fbm::PostAccumulator as _;
                    use noiz::noise::fbm::Accumulator as _;
                    let mut __fbm_acc_start = #accumulator_constructor;
                    let mut __fbm_acc;

                    let mut #fbm_ident = input;
                    #first
                    #(#octaves)*

                    let mut input = __fbm_acc.finish();
                }
            }
            Operation::RefOp(RefOp {
                attrs,
                ident,
                refer,
                ops,
            }) => {
                let ops = ops.iter().map(|op| op.quote_noise());
                quote! {
                    #(#attrs)*
                    let #ident = {
                        let mut input = #refer;
                        #(#ops)*
                        input
                    };
                }
            }
        }
    }

    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        *noise_amount += 1;
        if let Ok(_is_construction_variable) = input.parse::<Token![const]>() {
            match input.parse::<Stmt>() {
                Ok(Stmt::Local(var)) => Ok(Self::ConstructionVariable(var)),
                Ok(_) => {
                    Err(input
                        .error("Only local bindings are allowed to follow 'const' in a noise_op."))
                }
                Err(err) => Err(err),
            }
        } else if input.peek(Token![ref]) {
            Ok(Self::RefOp(RefOp::parse(input, noise_amount)?))
        } else if let Ok(op) = ConstructableField::<Token![use]>::parse(input, noise_amount) {
            Ok(Self::Data(op))
        } else if let Ok(op) = ConstructableField::<Token![fn]>::parse(input, noise_amount) {
            Ok(Self::Noise(op))
        } else if input.peek(Token![loop]) {
            Ok(Self::Fbm(FbmOp::parse(input, noise_amount)?))
        } else if let Ok(_is_converter) = input.parse::<Token![as]>() {
            let conversions = Punctuated::parse_separated_nonempty(input)?;
            Ok(Self::Convert(ConversionChain { conversions }))
        } else if let Ok(_is_mapper) = input.parse::<Token![mut]>() {
            Ok(Self::Mapping(Mapping {
                mapped: input.parse()?,
                operation: Box::new(Self::parse(input, noise_amount)?),
            }))
        } else if let Ok(op) = input.parse::<Morph>() {
            Ok(Self::Morph(op))
        } else if let Ok(_is_parallel) = input.parse::<Token![for]>() {
            Ok(Self::Parallel(Box::new(Self::parse(input, noise_amount)?)))
        } else if let Ok(Stmt::Local(op)) = input.parse::<Stmt>() {
            Ok(Self::Hold(op))
        } else {
            Err(input.error(
                "Unable to parse a noise operation. Expected a noise key word like 'let', '||', \
                 'as', 'use', 'for', 'fn', 'loop', 'ref', 'mut, or 'const'.",
            ))
        }
    }
}

#[derive(Clone)]
struct ConstructableField<K: Parse> {
    attrs: Vec<Attribute>,
    vis: Visibility,
    #[expect(
        unused,
        reason = "Helpful for parsing to have this. Helpful for users for little type hints."
    )]
    key_word: K,
    ident: Ident,
    colon: Colon,
    ty: Type,
    #[expect(
        unused,
        reason = "Helpful for parsing to have this. Helpful for users for little type hints."
    )]
    eq: Eq,
    constructor: Expr,
}

impl<K: Parse + Clone> ConstructableField<K> {
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

    fn parse_named_no_constructor<'a>(input: ParseStream<'a>) -> Result<(Self, ParseStream<'a>)> {
        Ok((
            Self {
                attrs: Attribute::parse_outer(input)?,
                vis: input.parse()?,
                key_word: input.parse()?,
                ident: input.parse()?,
                colon: input.parse()?,
                ty: input.parse()?,
                eq: Default::default(),
                constructor: parse_quote! {Default::default()},
            },
            input,
        ))
    }

    fn parse_unnamed_no_constructor<'a>(
        input: ParseStream<'a>,
        noise_amount: &mut u32,
    ) -> Result<(Self, ParseStream<'a>)> {
        let ident_hint = *noise_amount;
        *noise_amount += 1;
        Ok((
            Self {
                attrs: Attribute::parse_outer(input)?,
                vis: input.parse()?,
                key_word: input.parse()?,
                ident: Ident::new(&format!("val{ident_hint}"), input.span()),
                colon: Default::default(),
                ty: input.parse()?,
                eq: Default::default(),
                constructor: parse_quote! {Default::default()},
            },
            input,
        ))
    }

    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        let name_fork = input.fork();
        let unnamed_fork = input.fork();
        Self::parse_named_no_constructor(&name_fork)
            .or_else(|_| Self::parse_unnamed_no_constructor(&unnamed_fork, noise_amount))
            .and_then(|(mut result, fork)| {
                input.advance_to(fork);

                if let Ok(_found_custom_constructor) = input.parse::<Token![=]>() {
                    result.constructor = input.parse::<Expr>()?;
                }

                Ok(result)
            })
    }

    fn quote_constructor(&self) -> proc_macro2::TokenStream {
        let name = &self.ident;
        let constructor = &self.constructor;
        quote! {let #name = #constructor;}
    }
}

#[derive(Clone)]
struct ConversionChain {
    conversions: Punctuated<Type, Token![,]>,
}

#[derive(Clone)]
struct Morph {
    mutable: bool,
    input_name: Ident,
    #[expect(
        unused,
        reason = "Helpful for parsing to have this. Helpful for users for little type hints."
    )]
    input_type: Option<Type>,
    block: Expr,
}

impl Parse for Morph {
    fn parse(input: ParseStream) -> Result<Self> {
        _ = input.parse::<Token![ | ]>()?;
        let (input_name, mutable, input_type) = if !input.peek(Token![ | ]) {
            let mutable = input.parse::<Token![mut]>().is_ok();
            let input_name = input
                .parse()
                .unwrap_or_else(|_| Ident::new("input", input.span()));
            let input_type = if input.parse::<Token![:]>().is_ok() {
                Some(input.parse::<Type>()?)
            } else {
                None
            };
            (input_name, mutable, input_type)
        } else {
            (Ident::new("input", input.span()), false, None)
        };
        _ = input.parse::<Token![ | ]>()?;
        Ok(Self {
            mutable,
            input_name,
            input_type,
            block: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn noise_op(input: TokenStream) -> TokenStream {
    let noise = parse_macro_input!(input as NoiseDefinition);
    quote! {#noise}.into()
}
