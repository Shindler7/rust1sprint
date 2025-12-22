//! Поддерживающие библиотеку макросы.

use proc_macro::TokenStream;
use quote::quote;
use syn::Data::Enum;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

/// Derive-макрос, который генерирует методы для enum категории `TxType` и `TxStatus`, позволяющие
/// динамически взаимодействовать с перечислениями, получать их текстовые представления.
///
/// ## Доступные методы:
///
/// - `pub const fn as_u8(self) -> u8 {self as u8}`
///
/// Предоставляет возможность получить id заданного перечисления.
///
/// ```ignore
/// use parser::models::TxType;
///
/// let w = TxType::Withdraw;
/// println!("{}", w.as_u8());
/// ```
///
/// - `pub const fn from_u8(value: u8) -> Option<Self>`
///
/// Предоставляет возможность получить перечисление по его значению.
///
/// ```ignore
/// use parser::models::TxType;
///
/// let n = TxType::from_u8(0).unwrap();
/// println!("{}", n);
/// ```
///
/// Два других метода: реализация `Display` и возможность получить экземпляр перечисления на основе
/// его текстового представления (`FromStr`).
#[proc_macro_derive(TxDisplay)]
pub fn derive_tx_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let variants = match input.data {
        Enum(data) => data.variants,
        _ => panic!("#[derive(TxDisplay)] может применяться только с Enum"),
    };

    let variant_data: Vec<_> = variants
        .iter()
        .map(|variant| {
            let ident = &variant.ident;
            let value = match &variant.discriminant {
                Some(d) => d.1.clone(),
                None => panic!("Элементам Enum не присвоены значения"),
            };
            let name_uppercase = ident.to_string().to_uppercase();
            (ident, value, name_uppercase)
        })
        .collect();

    // from_u8
    let match_arms = variant_data.iter().map(|(ident, value, _)| {
        quote! { #value => Some(Self::#ident), }
    });

    // fmt::Display
    let display_arms = variant_data.iter().map(|(ident, _, uppercase)| {
        quote! { Self::#ident => #uppercase, }
    });

    // FromStr
    let from_str_arms = variant_data.iter().map(|(ident, _, uppercase)| {
        quote! { #uppercase => Ok(Self::#ident), }
    });

    // Сборка комплекта.
    let expanded = quote! {
        impl #name {
            pub const fn as_u8(self) -> u8 {self as u8}

            pub const fn from_u8(value: u8) -> Option<Self> {
                match value {
                    #(#match_arms)*
                    _ => None,
                }
            }
        }

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
                let result = match self { #(#display_arms)* };
                f.write_str(result)
            }
        }

        impl std::str::FromStr for #name {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.to_uppercase().as_str() {
                    #(#from_str_arms)*
                    _ => Err("Неизвестное значение"),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

/// Derive-макрос, который собирает методы, позволяющие обрабатывать поля структур, для их
/// отображения (`Display`), а также использование в текстовых данных.
///
/// ## Доступные методы
///
/// * `fn has_field_from_str(field: &str) -> bool`
///
/// Метод для структуры, который позволяет проверить наличие поля структуры через строковое
/// представление поля.
#[proc_macro_derive(YPBankDisplay)]
pub fn derive_ypbank_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let fields_named = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields_named),
            ..
        }) => fields_named,
        _ => panic!("YPBankDisplay работает только с именованными структурами"),
    };

    // Собираем имена полей и их UPPERCASE
    let field_pairs: Vec<_> = fields_named
        .named
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .map(|ident| {
            let field_name = ident.to_string();
            let uppercase = field_name.to_uppercase();
            (ident, field_name, uppercase)
        })
        .collect();

    // Литерные (utf-8) названия полей в UPPERCASE.
    let liter_fields = field_pairs
        .iter()
        .map(|(_, _, uppercase)| syn::LitStr::new(uppercase, name.span()));

    // Display::fmt - просто перечисляем поля
    let display_fields = field_pairs.iter().map(|(ident, field_name, _)| {
        quote! {
            write!(f, "{}: {:?}, ", #field_name, self.#ident)?;
        }
    });

    let expanded = quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{} {{ ", stringify!(#name))?;
                #(#display_fields)*
                write!(f, "}}")
            }
        }

        impl #name {
            pub fn has_field_from_str(field: &str) -> bool {
                matches!(
                    field.to_uppercase().as_str(),
                    #(#liter_fields)|*
                )
            }
        }
    };

    TokenStream::from(expanded)
}
