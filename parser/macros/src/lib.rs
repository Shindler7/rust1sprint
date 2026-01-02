//! Поддерживающие библиотеку макросы.

use proc_macro::TokenStream;
use quote::quote;
use syn::Data::Enum;
use syn::{Data, DataStruct, DeriveInput, Fields, parse_macro_input};

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
            /// Предоставить id-поля перечисления для экземпляра.
            ///
            /// ID формируется автоматически при помощи `repr`.
            pub const fn as_u8(self) -> u8 {self as u8}

            /// Предоставить экземпляр перечисления на основе id.
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

/// Макрос `YPBankDisplay` автоматически реализует методы для работы со строковыми представлениями
/// полей структуры: проверка наличия поля по имени (в любом регистре).
///
/// ## Реализуемые методы
///
/// - `fn has_field_from_str(field: &str) -> bool` — проверяет наличие поля по строковому имени.
/// - `fn fields() -> [&'static str; N]` — возвращает массив имён полей в верхнем регистре.
///
/// ## Ограничения:
/// Работает только с именованными структурами (без tuple-structs и unit-structs).
#[proc_macro_derive(YPBankFields)]
pub fn derive_ypbank_fields(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    let fields_named = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref named_fields),
            ..
        }) => named_fields,
        _ => {
            return syn::Error::new_spanned(
                struct_name,
                "YPBankDisplay работает только с именованными структурами",
            )
            .to_compile_error()
            .into();
        }
    };

    // Собираем идентификаторы и строковые версии имён полей.
    let field_pairs: Vec<_> = fields_named
        .named
        .iter()
        .filter_map(|f| f.ident.as_ref())
        .map(|ident| {
            let field_str = ident.to_string();
            let uppercase = field_str.to_uppercase();
            (ident.clone(), field_str, uppercase)
        })
        .collect();

    // Создаём выражения (`"FIELD_NAME"`) для массива `fields()`.
    let field_names = field_pairs
        .iter()
        .map(|(_, _, uppercase)| syn::LitStr::new(uppercase, struct_name.span()));

    let field_count = field_pairs.len();
    // Генерируем реализацию
    let expanded = quote! {
        impl #struct_name {
            /// Проверяет, содержится ли поле с заданным именем (в любом регистре) в структуре.
            ///
            /// ```no_run
            /// use parser::models::YPBankCsvFormat;
            ///
            /// assert!(YPBankCsvFormat::has_field_from_str("id"));
            /// assert!(YPBankCsvFormat::has_field_from_str("ID"));
            /// assert!(!YPBankCsvFormat::has_field_from_str("not_a_field"));
            /// ```
            pub fn fields() -> [&'static str; #field_count] {
                [
                    #(#field_names),*
                ]
            }

            /// Возвращает список имён всех полей структуры в верхнем регистре.
            ///
            /// Метод полезен при автоматическом отображении или проверке допустимых значений.
            pub fn has_field_from_str(field: &str) -> bool {
                let field_upper = field.to_uppercase();
                Self::fields().contains(&field_upper.as_str())
            }

        }
    };

    TokenStream::from(expanded)
}
