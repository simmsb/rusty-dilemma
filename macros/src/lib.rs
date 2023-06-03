use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    bracketed, parenthesized, parse::Parse, parse_macro_input, punctuated::Punctuated, token, LitInt,
};

#[allow(unused)]
struct Key {
    x: LitInt,
    y: LitInt,
    val: (u8, u8),
    paren_token: token::Paren,
    comma_token: token::Comma,
}

impl Parse for Key {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;

        let paren_token = parenthesized!(content in input);
        let x: LitInt = content.parse()?;
        let comma_token = content.parse()?;
        let y: LitInt = content.parse()?;
        let x_val = x.base10_parse()?;
        let y_val = y.base10_parse()?;
        Ok(Key {
            paren_token,
            x,
            comma_token,
            y,
            val: (x_val, y_val),
        })
    }
}

#[allow(unused)]
struct Chord {
    inputs: Punctuated<Key, token::Comma>,
    outputs: Punctuated<Key, token::Comma>,
    input_bracket_token: token::Bracket,
    outputs_bracket_token: token::Bracket,
    arrow_token: token::FatArrow,
}

impl Parse for Chord {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let inputs_content;
        let outputs_content;

        Ok(Chord {
            input_bracket_token: bracketed!(inputs_content in input),
            inputs: Punctuated::parse_separated_nonempty(&inputs_content)?,
            arrow_token: input.parse()?,
            outputs_bracket_token: bracketed!(outputs_content in input),
            outputs: Punctuated::parse_separated_nonempty(&outputs_content)?,
        })
    }
}

fn singleton(x: TokenStream) -> TokenStream {
    quote!({
        type T = impl Sized;
        static STATIC_CELL: ::static_cell::StaticCell<T> = ::static_cell::StaticCell::new();
        let (x,) = STATIC_CELL.init((#x,));
        x
    })
}

#[proc_macro]
pub fn chords(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut key_chord_map: HashMap<(u8, u8), Vec<usize>> = HashMap::new();
    let mut chords: Vec<Chord> = Vec::new();

    let p = parse_macro_input!(item with Punctuated::<Chord, token::Comma>::parse_terminated);

    for (i, chord) in p.into_iter().enumerate() {
        for key in &chord.inputs {
            key_chord_map.entry(key.val).or_default().push(i);
        }

        chords.push(chord);
    }

    let chord_defns = chords.into_iter().map(|c| {
        let mut keys_map = phf_codegen::Map::new();
        let num_keys = c.inputs.len();
        for (i, key) in c.inputs.iter().enumerate() {
            keys_map.entry([key.val.0, key.val.1], &i.to_string());
        }
        let keys_t = keys_map.build().to_string().parse::<TokenStream>().unwrap();
        let key_states_t = singleton(quote!([false; #num_keys]));
        let actions_t = c.outputs.iter().map(|Key { x, y, .. }| quote!((#x, #y)));
        let action_t = quote!([#(#actions_t),*]);

        quote!(
            {
                let key_states = #key_states_t;
                static KEY_MAP: ::phf::Map<[u8; 2], usize> = #keys_t;
                static ACTION: &'static [(u8, u8)] = &#action_t;

                crate::keys::chord::Chord {
                    key_map: &KEY_MAP,
                    key_states,
                    is_active: false,
                    action: ACTION,
                }
            }
        )
    });

    let chord_defns_t = singleton(quote!([#(#chord_defns),*]));

    let mut key_chord_map_p = phf_codegen::Map::new();
    for (key, val) in key_chord_map {
        key_chord_map_p.entry([key.0, key.1], &format!("{{ static A: &[usize] = &{:?}; A }}", val));
    }
    let key_chord_map_t = key_chord_map_p
        .build()
        .to_string()
        .parse::<TokenStream>()
        .unwrap();

    quote!({
        let chords = #chord_defns_t;
        static KEY_CHORD_MAP: ::phf::Map<[u8; 2], &[usize]> = #key_chord_map_t;

        crate::keys::chord::Chorder {
            key_chord_map: &KEY_CHORD_MAP,
            chords,
        }
    })
    .into()
}
