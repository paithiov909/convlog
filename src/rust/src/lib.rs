//! Provides methods to transform mahjong logs from tenhou.net/6 format into
//! mjai format.

#![allow(clippy::manual_range_patterns)] // because of matches_tu8
#![deny(
    rust_2018_idioms,
    let_underscore_drop,
    clippy::assertions_on_result_states,
    clippy::bool_to_int_with_if,
    clippy::borrow_as_ptr,
    clippy::cloned_instead_of_copied,
    clippy::create_dir,
    clippy::debug_assert_with_mut_call,
    clippy::default_union_representation,
    clippy::deref_by_slicing,
    clippy::derive_partial_eq_without_eq,
    clippy::empty_drop,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::equatable_if_let,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::format_push_string,
    clippy::from_iter_instead_of_collect,
    clippy::get_unwrap,
    clippy::implicit_clone,
    clippy::implicit_saturating_sub,
    clippy::imprecise_flops,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::large_types_passed_by_value,
    clippy::let_unit_value,
    clippy::lossy_float_literal,
    clippy::macro_use_imports,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::map_unwrap_or,
    clippy::match_bool,
    clippy::match_same_arms,
    clippy::missing_const_for_fn,
    clippy::mut_mut,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::nonstandard_macro_braces,
    clippy::or_fun_call,
    clippy::path_buf_push_overwrite,
    clippy::ptr_as_ptr,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::redundant_else,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::semicolon_if_nothing_returned,
    clippy::significant_drop_in_scrutinee,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_lit_as_bytes,
    clippy::string_to_string,
    clippy::suboptimal_flops,
    clippy::suspicious_to_owned,
    clippy::trait_duplication_in_bounds,
    clippy::trivially_copy_pass_by_ref,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_join,
    clippy::unnecessary_self_imports,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unseparated_literal_suffix,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::use_self,
    clippy::used_underscore_binding,
    clippy::useless_let_if_seq
)]

mod conv;
mod kyoku_filter;
mod macros;
mod mjai;
mod mjlog;
mod tile;

mod tenhou;

use std::str::FromStr;

pub use conv::{ConvertError, tenhou_to_mjai};
pub use kyoku_filter::KyokuFilter;
pub use mjai::Event;
pub use tile::{tile_set_eq, Tile};

use quick_xml::events::Event as XmlEvent;
use quick_xml::reader::Reader as XmlReader;
use serde_json as json;

use savvy::{savvy, savvy_err};
use savvy::{OwnedListSexp, OwnedStringSexp, StringSexp, NotAvailableValue};

/// Convert 'tenhou.net/6' JSON strings into mjai log format
///
/// @param x A character vector.
/// @returns A list of character vectors
/// where each element represents one mjai event as a JSON string.
/// @noRd
#[savvy]
fn parse_tenhou6(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = OwnedListSexp::new(x.len(), false)?;

    for (i, e) in x.iter().enumerate() {
        if e.is_na() {
            let mut dummy = OwnedStringSexp::new(1)?;
            dummy.set_na(0)?;
            out.set_value(i, dummy)?;
            continue;
        }
        let tenhou_log = tenhou::Log::from_json_str(&e)?;
        let events = tenhou_to_mjai(&tenhou_log)?;

        let mut ret = OwnedStringSexp::new(events.len())?;
        for (j, event) in events.iter().enumerate() {
            let to_write = json::to_string(event)?;
            ret.set_elt(j, &to_write)?;
        }
        out.set_value(i, ret)?;
    }

    Ok(out.into())
}

/// Parse mjlog XML into mjai log format
///
/// @param x A character vector.
/// @returns A list of character vectors
/// where each element represents one mjai event as a JSON string.
/// @noRd
#[savvy]
fn parse_mjlog(x: StringSexp) -> savvy::Result<savvy::Sexp> {
    let mut out = OwnedListSexp::new(x.len(), false)?;
    let mut buf = Vec::new();

    for (i, elem) in x.iter().enumerate() {
        if elem.is_na() {
            let mut dummy = OwnedStringSexp::new(1)?;
            dummy.set_na(0)?;
            out.set_value(i, dummy)?;
            continue;
        }
        let mut reader = XmlReader::from_str(elem);

        let mut aka_flag: bool = false;
        let mut is_initialized: bool = false;
        let mut player_names: (String, String, String, String) = (
            "player1".to_string(),
            "player2".to_string(),
            "player3".to_string(),
            "player4".to_string(),
        );
        let mut last_draw: [u8; 4] = vec![Tile::from_str("?").unwrap().as_u8(); 4].try_into().unwrap();
        let mut reach_count: u8 = 0;

        let mut events: Vec<Event> = Vec::new();
        'read_event: loop {
            match reader.read_event_into(&mut buf) {
                Ok(XmlEvent::Eof) => break 'read_event,
                Ok(XmlEvent::Start(e)) => match e.name().as_ref() {
                    b"mjloggm" => {
                        let ver = mjlog::parse_mjloggm_version(&e)?;
                        if ver != "2.3" {
                            return Err(savvy_err!("mjloggm ver {} is not supported.", ver));
                        }
                    }
                    _ => (),
                },
                Ok(XmlEvent::Empty(e)) => {
                    match e.name().as_ref() {
                        b"GO" => {
                            let (flag, is_sanma) = mjlog::parse_game_type(&e)?;
                            if is_sanma {
                                return Err(savvy_err!("sanma is not supported."));
                            }
                            aka_flag = flag;
                        }
                        b"UN" => {
                            if !is_initialized {
                                player_names = mjlog::parse_names(&e)?;
                                is_initialized = true;
                            }
                        }
                        b"TAIKYOKU" => {
                            let names: [String; 4] = [
                                player_names.0.clone(),
                                player_names.1.clone(),
                                player_names.2.clone(),
                                player_names.3.clone(),
                            ];
                            events.push(Event::StartGame {
                                names,
                                kyoku_first: 0,
                                aka_flag,
                            });
                        }
                        b"SHUFFLE" => {}
                        b"INIT" => {
                            // NOTE: 手牌は並び替えされていない
                            let (bakaze, dora_marker, kyoku, honba, kyotaku, oya) =
                                mjlog::parse_init_others(&e, aka_flag)?;
                            let scores = mjlog::parse_init_scores(&e)?;
                            let tehais = mjlog::parse_init_tehais(&e, aka_flag)?;
                            events.push(Event::StartKyoku {
                                bakaze,
                                dora_marker,
                                kyoku,
                                honba,
                                kyotaku,
                                oya,
                                scores,
                                tehais
                            });
                        }
                        b"DORA" => {
                            let dora_marker = mjlog::parse_dora(&e, aka_flag)?;
                            events.push(Event::Dora { dora_marker });
                        }
                        b"N" => {
                           let (call_type, caller, callee, tiles) = mjlog::parse_n(&e, aka_flag)?;
                           match call_type.as_str() {
                               "Chi" => {
                                    events.push(Event::Chi {
                                        actor: caller,
                                        target: callee,
                                        pai: tiles[0],
                                        consumed: tiles[1..].try_into().unwrap(),
                                    });
                               }
                               "Pon" => {
                                    events.push(Event::Pon {
                                        actor: caller,
                                        target: callee,
                                        pai: tiles[0],
                                        consumed: tiles[1..].try_into().unwrap(),
                                    });
                               }
                               "Kakan" => {
                                    events.push(Event::Kakan {
                                        actor: caller,
                                        pai: tiles[0],
                                        consumed: vec![tiles[1], tiles[1], tiles[1]].try_into().unwrap(),
                                    });
                               }
                               "Ankan" => {
                                    // 5m,5p,5sは、赤ありのとき1枚赤くする
                                    let tile_0: Tile = if aka_flag & matches_tu8!(tiles[0].as_u8(), 5m | 5p | 5s) {
                                        tiles[0].akaize()
                                    } else {
                                        tiles[0]
                                    };
                                    events.push(Event::Ankan {
                                        actor: caller,
                                        consumed: vec![tiles[1], tiles[1], tiles[1], tile_0].try_into().unwrap(),
                                    });
                                }
                                "Minkan" => {
                                    events.push(Event::Daiminkan {
                                        actor: caller,
                                        target: callee,
                                        pai: tiles[0],
                                        consumed: tiles[1..].try_into().unwrap(),
                                    });
                               }
                               _ => {}
                           }
                        }
                        b"REACH" => {
                            // NOTE: 本来の`reach_accepted`は宣言牌が鳴かれた場合は次以降の巡目になるが、ここでは考慮しない
                            let (actor, step) = mjlog::parse_reach(&e)?;
                            match step {
                                1 => events.push(Event::Reach { actor }),
                                2 if reach_count < 4 => {
                                    reach_count += 1;
                                    events.push(Event::ReachAccepted { actor })
                                }
                                _ => (),
                            }
                        }
                        b"AGARI" => {
                            let (actor, target, ura_markers, deltas) =
                                mjlog::parse_agari(&e, aka_flag)?;
                            events.push(Event::Hora {
                                actor,
                                target,
                                ura_markers,
                                deltas,
                            });
                            events.push(Event::EndKyoku);
                            reach_count = 0;
                            if mjlog::check_if_owari(&e)? {
                                events.push(Event::EndGame);
                            }
                        }
                        b"RYUUKYOKU" => {
                            let deltas = mjlog::parse_ryuukyoku(&e)?;
                            events.push(Event::Ryukyoku { deltas });
                            events.push(Event::EndKyoku);
                            reach_count = 0;
                            if mjlog::check_if_owari(&e)? {
                                events.push(Event::EndGame);
                            }
                        }
                        b"BYE" => {}
                        _ => {
                            let name = e.name().into_inner();
                            let name = String::from_utf8_lossy(name).into_owned();
                            let tile_int = name[1..].parse::<u8>().unwrap();
                            let pai = mjlog::translate_mjlog_tile(tile_int, aka_flag).unwrap();
                            match name.chars().next() {
                                // [T-W]はTsumo
                                Some('T') => {
                                    last_draw[0] = pai.as_u8();
                                    events.push(Event::Tsumo { actor: 0, pai })
                                }
                                Some('U') => {
                                    last_draw[1] = pai.as_u8();
                                    events.push(Event::Tsumo { actor: 1, pai })
                                }
                                Some('V') => {
                                    last_draw[2] = pai.as_u8();
                                    events.push(Event::Tsumo { actor: 2, pai })
                                }
                                Some('W') => {
                                    last_draw[3] = pai.as_u8();
                                    events.push(Event::Tsumo { actor: 3, pai })
                                }
                                // [D-G]はDahai
                                Some('D') => events.push(Event::Dahai { actor: 0, pai, tsumogiri: last_draw[0] == pai.as_u8() }),
                                Some('E') => events.push(Event::Dahai { actor: 1, pai, tsumogiri: last_draw[1] == pai.as_u8() }),
                                Some('F') => events.push(Event::Dahai { actor: 2, pai, tsumogiri: last_draw[2] == pai.as_u8() }),
                                Some('G') => events.push(Event::Dahai { actor: 3, pai, tsumogiri: last_draw[3] == pai.as_u8() }),
                                _ => (),
                            }
                        }
                    }
                }
                _ => (),
            }
            buf.clear();
        }

        let mut ret = OwnedStringSexp::new(events.len())?;
        for (j, event) in events.iter().enumerate() {
            let to_write = json::to_string(event)?;
            ret.set_elt(j, &to_write)?;
        }
        out.set_value(i, ret)?;
    }

    Ok(out.into())
}
