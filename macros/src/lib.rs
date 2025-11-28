use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Derives the `AccessLevel` trait for enums.
///
/// This macro automatically implements `from_str` and `as_str` methods
/// for your access level enum, using the variant names as string representations.
///
/// # Example
///
/// ```ignore
/// use nut_shell_macros::AccessLevel;
///
/// #[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, AccessLevel)]
/// pub enum MyAccessLevel {
///     Guest = 0,
///     User = 1,
///     Admin = 2,
/// }
/// ```
///
/// This generates:
///
/// ```ignore
/// impl AccessLevel for MyAccessLevel {
///     fn from_str(s: &str) -> Option<Self> {
///         match s {
///             "Guest" => Some(Self::Guest),
///             "User" => Some(Self::User),
///             "Admin" => Some(Self::Admin),
///             _ => None,
///         }
///     }
///
///     fn as_str(&self) -> &'static str {
///         match self {
///             Self::Guest => "Guest",
///             Self::User => "User",
///             Self::Admin => "Admin",
///         }
///     }
/// }
/// ```
///
/// # Requirements
///
/// - The type must be an enum
/// - All variants must be unit variants (no fields)
/// - Variant names will be used as the string representation
#[proc_macro_derive(AccessLevel)]
pub fn derive_access_level(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    // Extract enum variants
    let variants = match &input.data {
        Data::Enum(data_enum) => &data_enum.variants,
        _ => {
            return syn::Error::new_spanned(&input, "AccessLevel can only be derived for enums")
                .to_compile_error()
                .into();
        }
    };

    // Validate that all variants are unit variants (no fields)
    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                variant,
                "AccessLevel can only be derived for enums with unit variants (no fields)",
            )
            .to_compile_error()
            .into();
        }
    }

    // Generate match arms for from_str
    let from_str_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();
        quote! {
            #variant_str => Some(Self::#variant_name)
        }
    });

    // Generate match arms for as_str
    let as_str_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();
        quote! {
            Self::#variant_name => #variant_str
        }
    });

    // Generate the impl block
    let expanded = quote! {
        impl ::nut_shell::auth::AccessLevel for #name {
            fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_arms,)*
                    _ => None,
                }
            }

            fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms,)*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
