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
        let operations = Operation::parse_many(input)?;
        for op in operations.iter() {
            op.store_fields(&mut noise.data, &noise.name);
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

        let ops = operations.iter().map(|op| op.quote_external(self));

        tokens.extend(quote! {
            #noise

            #source

            #(#ops)*

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
enum NoiseType {
    Vanila(Type),
    Lambda(Lambda),
}

impl NoiseType {
    fn get_type(&self, root_name: &Ident) -> Type {
        match self {
            NoiseType::Vanila(ty) => ty.clone(),
            NoiseType::Lambda(Lambda {
                ops,
                input,
                output,
                source: _,
                id,
            }) => {
                let mut fields = Punctuated::default();
                for op in ops {
                    op.store_fields(&mut fields, root_name);
                }
                let field_types = fields.iter().map(|field| &field.ty);
                parse_quote!(noiz::noise::lambda::Lambda<(#(#field_types),*), #input, #output, #id, #root_name>)
            }
        }
    }

    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        Ok(if input.peek(Token![type]) {
            Self::Lambda(Lambda::parse(input, noise_amount)?)
        } else {
            Self::Vanila(input.parse()?)
        })
    }

    fn quote_external(&self, full: &NoiseDefinition) -> proc_macro2::TokenStream {
        match self {
            Self::Vanila(_) => quote! {},
            Self::Lambda(Lambda {
                ops,
                input,
                output,
                source,
                id,
            }) => {
                let def_name = &full.noise.name;
                let mut fields = Punctuated::default();
                for op in ops {
                    op.store_fields(&mut fields, def_name);
                }
                let field_types = fields.iter().map(|field| &field.ty);
                let field_names = fields
                    .iter()
                    .map(|field| field.ident.as_ref().expect("Fields must be named."))
                    .collect::<Vec<_>>();
                let constructions = ops.iter().map(|op| op.quote_construction(def_name));
                let noise_impls = ops.iter().map(|op| op.quote_noise());
                quote! {
                    impl From<#source> for noiz::noise::lambda::Lambda<(#(#field_types),*), #input, #output, #id, #def_name> {
                        fn from(value: #source) -> Self {
                            let mut args = value;

                            #(#constructions)*

                            let data = (#(#field_names),*);
                            Self::new(data, |(#(#field_names),*), input| {

                                #(#noise_impls)*

                                input
                            })
                        }
                    }
                }
            }
        }
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
    vis: Visibility,
    ident: Ident,
    settings: Expr,
    octaves: Vec<NoiseType>,
}

impl FbmOp {
    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        let _kw = input.parse::<Token![loop]>()?;
        let is_field = input.parse::<Token![use]>().is_ok();
        let attrs = Attribute::parse_outer(input)?;
        let vis = input.parse()?;
        let ident = if is_field {
            let name = input.parse::<Ident>()?;
            _ = input.parse::<Token![=]>()?;
            name
        } else {
            Ident::new(&format!("fbm{}", *noise_amount), input.span())
        };
        let settings = input.parse()?;
        _ = input.parse::<Token![enum]>()?;

        let octaves_stream;
        let _ = bracketed!(octaves_stream in input);
        let mut octaves = Vec::new();
        while !octaves_stream.is_empty() {
            let repeat = match octaves_stream.parse::<LitInt>() {
                Ok(lit) => lit.base10_parse::<u8>()?,
                Err(_) => 1,
            };

            *noise_amount += 1;
            let noise = NoiseType::parse(&octaves_stream, noise_amount)?;
            for _ in 0..repeat {
                octaves.push(noise.clone());
            }

            if octaves_stream.parse::<Token![,]>().is_err() {
                break;
            }
        }
        Ok(Self {
            ident,
            octaves,
            settings,
            vis,
            attrs,
        })
    }
}

#[derive(Clone)]
struct Lambda {
    ops: Vec<Operation>,
    input: Type,
    output: Type,
    source: Type,
    id: u32,
}

impl Lambda {
    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        _ = input.parse::<Token![type]>()?;
        let input_type = input.parse()?;
        _ = input.parse::<Token![->]>()?;
        let output = input.parse()?;
        _ = input.parse::<Token![for]>()?;
        let source = input.parse()?;
        _ = input.parse::<Token![impl]>()?;
        let lambda;
        _ = braced!(lambda in input);
        let ops = Operation::parse_many(&lambda)?;

        Ok(Self {
            ops,
            input: input_type,
            output,
            source,
            id: *noise_amount,
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
}

impl Operation {
    fn parse_many(input: ParseStream) -> Result<Vec<Self>> {
        let mut operations = Vec::new();
        let mut noise_ops = 0u32;
        loop {
            if input.is_empty() {
                break;
            }
            let value = Operation::parse(input, &mut noise_ops)?;
            let needs_semi_colon = value.needs_following_semi_colon() && !input.is_empty();
            operations.push(value);
            if needs_semi_colon || input.peek(Token![;]) {
                _ = input.parse::<Token![;]>()?;
            }
        }
        Ok(operations)
    }

    fn quote_external(&self, full: &NoiseDefinition) -> proc_macro2::TokenStream {
        match self {
            Operation::Parallel(op) => op.quote_external(full),
            Operation::Mapping(mapping) => mapping.operation.quote_external(full),
            Operation::Noise(noise) => noise.ty.quote_external(full),
            Operation::Fbm(fbm) => {
                let fbm = fbm.octaves.iter().map(|op| op.quote_external(full));
                quote! {#(#fbm)*}
            }
            _ => quote! {},
        }
    }

    fn needs_following_semi_colon(&self) -> bool {
        match self {
            Operation::Noise(_)
            | Operation::Fbm(_)
            | Operation::Convert(_)
            | Operation::Data(_) => true,
            Operation::ConstructionVariable(_) | Operation::Hold(_) => false,
            Operation::Morph(morph) => !matches!(&morph.block, Expr::Block(_) | Expr::TryBlock(_)),
            Operation::Parallel(op) => op.needs_following_semi_colon(),
            Operation::Mapping(mapping) => mapping.operation.needs_following_semi_colon(),
        }
    }

    fn store_fields(&self, fields: &mut Punctuated<Field, Token![,]>, root_name: &Ident) {
        match self {
            Operation::Data(field) => fields.push(field.field(root_name)),
            Operation::Noise(field) => fields.push(field.field(root_name)),
            Operation::Fbm(fbm) => {
                let types = fbm.octaves.iter().map(|ty| ty.get_type(root_name));
                fields.push(Field {
                    attrs: fbm.attrs.clone(),
                    vis: fbm.vis.clone(),
                    mutability: FieldMutability::None,
                    ident: Some(fbm.ident.clone()),
                    colon_token: Default::default(),
                    ty: parse_quote!( noiz::noise::fbm::Fbm<( #(noiz::noise::fbm::FbmOctave<#types>),* )> ),
                });
            }
            Operation::Parallel(op) => op.store_fields(fields, root_name),
            Operation::Mapping(mapping) => mapping.operation.store_fields(fields, root_name),
            _ => {}
        }
    }

    fn quote_construction(&self, root_name: &Ident) -> proc_macro2::TokenStream {
        match self {
            Operation::Data(field) => field.quote_constructor(),
            Operation::Noise(field) => field.quote_constructor(),
            Operation::Fbm(fbm) => {
                let types = fbm.octaves.iter().map(|ty| ty.get_type(root_name));
                let settings = &fbm.settings;
                let name = &fbm.ident;
                quote! { let #name = noiz::noise::fbm::Fbm::<( #(noiz::noise::fbm::FbmOctave<#types>),* )>::new_fbm(#settings); }
            }
            Operation::ConstructionVariable(binding) => binding.to_token_stream(),
            Operation::Parallel(op) => op.quote_construction(root_name),
            Operation::Mapping(mapping) => mapping.operation.quote_construction(root_name),
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
                quote! {let input = #name.get(input); }
            }
            Operation::Convert(conversions) => {
                let conversions = conversions.conversions.iter();
                quote! {
                    let input = noiz::noise::convert!(input => #(#conversions),*);
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
                    let input = #block;
                }
            }
            Operation::Hold(local) => local.to_token_stream(),
            Operation::Parallel(op) => {
                let op_code = op.quote_noise();
                quote! {
                    let input = input.map(|input| {
                        #op_code
                        input
                    });
                }
            }
            Operation::Mapping(Mapping { operation, mapped }) => {
                let op = operation.quote_noise();
                quote! {
                    let input = noiz::noise::associating::AssociationMapping::<#mapped>::map_association(input, |input| {
                        #op
                        input
                    });
                }
            }
            Operation::Fbm(fbm_op) => {
                let name = &fbm_op.ident;
                quote! {let input = #name.get(input); }
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
                 'as', 'use', 'for', 'fn', 'loop', 'mut, or 'const'.",
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
    ty: NoiseType,
    #[expect(
        unused,
        reason = "Helpful for parsing to have this. Helpful for users for little type hints."
    )]
    eq: Eq,
    constructor: Expr,
}

impl<K: Parse + Clone> ConstructableField<K> {
    fn field(&self, root_name: &Ident) -> Field {
        Field {
            attrs: self.attrs.clone(),
            vis: self.vis.clone(),
            mutability: FieldMutability::None,
            ident: Some(self.ident.clone()),
            colon_token: Some(self.colon.clone()),
            ty: self.ty.get_type(root_name),
        }
    }

    fn parse_named_no_constructor<'a>(
        input: ParseStream<'a>,
        noise_amount: &mut u32,
    ) -> Result<(Self, ParseStream<'a>)> {
        Ok((
            Self {
                attrs: Attribute::parse_outer(input)?,
                vis: input.parse()?,
                key_word: input.parse()?,
                ident: input.parse()?,
                colon: input.parse()?,
                ty: NoiseType::parse(input, noise_amount)?,
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
                ty: NoiseType::parse(input, noise_amount)?,
                eq: Default::default(),
                constructor: parse_quote! {Default::default()},
            },
            input,
        ))
    }

    fn parse(input: ParseStream, noise_amount: &mut u32) -> Result<Self> {
        let name_fork = input.fork();
        let unnamed_fork = input.fork();
        Self::parse_named_no_constructor(&name_fork, noise_amount)
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

/// Creates a noise operation using smaller operations with key words `use`, `do`, `let`, `as`,
/// `fn`, and `for`.
///
/// # Example
///
/// ```
/// noise_op! {
///     // declare the name of the noise and what type it is for
///     /// Attributes work!
///     pub struct MyNoise for Vec2 =
///
///     // declare the data that is used to make the noise operation
///     /// Attributes work!
///     pub(crate) struct MyNoiseArgs {seed: u32, period: f32,}
///
///     impl // specifies the start of the noise implementation.
///
///     // `use` adds custom data to the noise struct. Visibility works too.
///     /// Attributes work!
///     #[allow(unused)]
///     pub use custom_data: f32 = period;
///
///     // 'do' is the same as 'use', but the value participates as a noise operation automatically.
///     pub do fist_noise: GridNoise = GridNoise::new_period(period);
///
///     // If you don't give a 'do' a name, it will make one for you.
///     /// Attributes work!
///     do Seeding = Seeding(seed);
///
///     // 'let' holds a temporary value during the noise calculation.
///     #[allow(unused)]
///     let GridPoint2{ base, offset } = input.value;
///
///     // If you don't provide a constructor for a 'do' value, the default will be used.
///     do MetaOf;
///
///     // 'as' performs a conversion chain through the types listed.
///     as UNorm, f32, UNorm;
///
///     // 'fn' performs a custom noise function. You must name the return type.
///     fn (mut x: UNorm) -> [UNorm; 3] {
///         // You can name the parameter and its type if you want.
///         // You can use the values of 'use' 'do' 'let' operations here.
///         x = UNorm::new_clamped(*custom_data * offset.x);
///         // You can't use return, but whatever value is left here is passed out as the result.
///         [x, x, x]
///     }
///
///     // 'for' operates on inner values of an array for this operation only.
///     // The next operation will be on the resulting mapped array.
///     for as f32;
///
///     // 'fn' operations don't need to specify their type,
///     // and if they don't specify a name, `input` is the default
///     fn () -> f32 {input[2]}
///
///     // whatever value is left here is returned for the noise operation.
/// }
/// ```
#[proc_macro]
pub fn noise_op(input: TokenStream) -> TokenStream {
    let noise = parse_macro_input!(input as NoiseDefinition);
    quote! {#noise}.into()
}
