use std::cell::LazyCell;

use proc_macro::{Ident, Punct, TokenStream, TokenTree};
use quote::{format_ident, quote};

#[inline(always)]
fn note(n: f32) -> f32 {
    440. * 2.0_f32.powf((n - 49.) / 12.)
}

// #[proc_macro]
// pub fn n(ts: TokenStream) -> TokenStream {
//     let mut out = TokenStream::new();
//     let mut ts_iter = ts.into_iter();
//     let Some(tt) = ts_iter.next() else {
//         panic!("invalid note name")
//     };
//     let TokenTree::Ident(ident) = tt else {
//         panic!("invalid note name")
//     };
//     let chars = ident.to_string();
//     let mut chars = chars.chars();
//     let Some(name) = chars.next() else {
//         panic!("invalid note name")
//     };
//     if matches!(name, 'A'..'G') {
//         let Some(register) = chars.next() else {
//             panic!("invalid note name")
//         };
//         if register.is_numeric() {
//             let note_name = format_ident!(
//                 "{}{}{}",
//                 name,
//                 register,
//                 match ts_iter.next() {
//                     None => "",
//                     Some(TokenTree::Punct(p)) => match p.as_char() {
//                         '+' => "_sharp",
//                         '-' => "_flat",
//                         _ => "",
//                     },
//                     _ => "",
//                 }
//             );
//             out = quote! { #note_name }.into();
//         };
//     }
//     out
// }

#[proc_macro]
pub fn midi_notes(_: TokenStream) -> TokenStream {
    let major_scale = vec![2, 2, 1, 2, 2, 2, 1];
    let harmonic_minor_scale = vec![2, 1, 2, 2, 1, 3, 1];
    let melodic_minor_scale_ascending = vec![2, 1, 2, 2, 2, 2, 1];
    let melodic_minor_scale_descending = vec![2, 2, 1, 2, 2, 1, 2];
    let order_of_sharps = vec!["", "F", "C", "G", "D", "A", "E", "B"];
    let order_of_flats = order_of_sharps
        .iter()
        .cloned()
        .rev()
        .cycle()
        .take(7)
        .collect::<Vec<&str>>();
    let note_names_sharp = vec![
        "A", "A#", "B", "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#",
    ];
    let note_names_flat = [
        "A", "Bb", "B", "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab",
    ];
    let major_sharp_keys = note_names_sharp
        .iter()
        .cycle()
        .skip(4)
        .step_by(7)
        .take(7)
        .map(|n| format!("{n} maj"))
        .collect::<Vec<String>>();
    let major_flat_keys = note_names_flat
        .iter()
        .cycle()
        .skip(8)
        .step_by(5)
        .take(7)
        .map(|n| format!("{n} maj"))
        .collect::<Vec<String>>();
    let minor_sharp_keys = note_names_sharp
        .iter()
        .rev()
        .cycle()
        .step_by(5)
        .take(7)
        .map(|n| format!("{n} min"))
        .collect::<Vec<String>>();
    let minor_flat_keys = note_names_flat
        .iter()
        .rev()
        .cycle()
        .skip(7)
        .step_by(7)
        .take(7)
        .map(|n| format!("{n} min"))
        .collect::<Vec<String>>();

    TokenStream::new()
}

// #[proc_macro]
// pub fn midi(_: TokenStream) -> TokenStream {
//     quote! {
//       fn note(n: f32) -> f32 {
//           440. * 2.0_f32.powf((n - 49.) / 12.)
//       }
//       let mut midi = [0.0_f32; 89];
//       midi.resize(89, 0.0);
//       for n in 1..=88 {
//           midi[n] = note(n as f32);
//       }
//     }
//     .into()
// }

#[proc_macro]
pub fn keys(_ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let variants: Vec<_> = [
        "C", "CSh", "Db", "D", "DSh", "E", "F", "FSh", "Gb", "G", "Ab", "A", "Bb",
    ]
    .iter()
    // .map(|n| n.to_string())
    .flat_map(|n| {
        let n = *n;
        [format_ident!("{n}Maj"), format_ident!("{n}Min")]
    })
    .collect();

    quote! {
      #[derive(Clone, Debug)]
      enum Key {
        #(#variants),*
      }
    }
    .into()
}
