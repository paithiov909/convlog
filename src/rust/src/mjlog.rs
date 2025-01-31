// these logics are taken from https://github.com/fstqwq/mjlog2mjai/blob/master/parse.py
use crate::Tile;
use crate::mjlog;

use quick_xml::encoding::Decoder;
use quick_xml::events::attributes::Attribute;
use quick_xml::events::BytesStart;
use std::str::FromStr;
use std::vec;
use urlencoding::decode;

const TRANSLATION: [&str; 35] = [
    "1m", "2m", "3m", "4m", "5m", "6m", "7m", "8m", "9m",
    "1p", "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p",
    "1s", "2s", "3s", "4s", "5s", "6s", "7s", "8s", "9s",
    "E", "S", "W", "N", "P", "F", "C",
    "?", // 稀にID=136のタイルがある
];

pub fn translate_mjlog_tile(tile: u8, red: bool) -> Option<Tile> {
    let ret = String::from(TRANSLATION[(tile >> 2) as usize]);
    if red && ret.starts_with('5') && (tile & 3) == 0 {
        Tile::from_str(&format!("{}r", ret)).ok()
    } else {
        Tile::from_str(&ret).ok()
    }
}

fn parse_chi(meld: u16) -> Vec<u8> {
    let mut base_tile = (meld & 0xfc00) >> 10;
    let rotation = base_tile % 3;
    base_tile /= 3;
    base_tile = 9 * (base_tile / 7) + (base_tile % 7);
    base_tile *= 4;

    let tiles: Vec<u8> = vec![
        base_tile + 4 * 0 + ((meld & 0x0018) >> 3),
        base_tile + 4 * 1 + ((meld & 0x0060) >> 5),
        base_tile + 4 * 2 + ((meld & 0x0180) >> 7),
    ]
        .iter()
        .map(|&x| x as u8)
        .collect();

    if rotation == 1 {
        return vec![tiles[1], tiles[0], tiles[2]]
    } else if rotation == 2 {
        return vec![tiles[2], tiles[0], tiles[1]]
    }
    tiles
}

fn parse_pon(meld: u16) -> Vec<u8> {
    let mut base_tile = (meld & 0xfe00) >> 9;
    let rotation = base_tile % 3;
    base_tile /= 3;
    base_tile *= 4;

    let mut tiles: Vec<u8> = vec![base_tile, base_tile, base_tile]
        .iter()
        .map(|&x| x as u8)
        .collect();

    match (meld & 0x0060) >> 5 {
        0 => {
            tiles[0] += 1;
            tiles[1] += 2;
            tiles[2] += 3;
        }
        1 => {
            tiles[1] += 2;
            tiles[2] += 3;
        }
        2 => {
            tiles[1] += 1;
            tiles[2] += 3;
        }
        3 => {
            tiles[1] += 1;
            tiles[2] += 2;
        }
        _ => {},
    }

    if rotation == 1 {
        tiles.rotate_left(1);
    } else if rotation == 2 {
        tiles.rotate_right(1);
    }
    tiles
}

fn parse_kakan(meld: u16) -> Vec<u8> {
    let added_tile = (meld & 0x0060) >> 5;
    let mut base_tile = (meld & 0xfe00) >> 9;
    let rotation = base_tile % 3;
    base_tile /= 3;
    base_tile *= 4;

    let mut tiles: Vec<u8> = vec![base_tile, base_tile, base_tile]
        .iter()
        .map(|&x| x as u8)
        .collect();

    match added_tile {
        0 => {
            tiles[0] += 1;
            tiles[1] += 2;
            tiles[2] += 3;
        }
        1 => {
            tiles[0] += 0;
            tiles[1] += 2;
            tiles[2] += 3;
        }
        2 => {
            tiles[0] += 0;
            tiles[1] += 1;
            tiles[2] += 3;
        }
        3 => {
            tiles[0] += 0;
            tiles[1] += 1;
            tiles[2] += 2;
        }
        _ => {}
    }

    if rotation == 1 {
        vec![tiles[1], tiles[0], tiles[2]]
    } else if rotation == 2 {
        vec![tiles[2], tiles[0], tiles[1]]
    } else {
        let insert: u8 = (base_tile + added_tile).try_into().unwrap();
        tiles.insert(0, insert);
        tiles
    }
}

fn parse_kan(meld: u16) -> Vec<u8> {
    let tile = ((meld & 0xff00) >> 8) as u8;
    let kui = (meld & 0x3) as u8;

    let mut tiles = vec![tile, tile, tile];
    let remainder = tile % 4;

    match remainder {
        0 => {
            tiles[0] += 1;
            tiles[1] += 2;
            tiles[2] += 3;
        }
        1 => {
            tiles[0] += 0;
            tiles[1] += 2;
            tiles[2] += 3;
        }
        2 => {
            tiles[0] += 0;
            tiles[1] += 1;
            tiles[2] += 3;
        }
        3 => {
            tiles[0] += 0;
            tiles[1] += 1;
            tiles[2] += 2;
        }
        _ => {}
    }

    if kui == 0 {
        tiles[0..2].to_vec()
    } else {
        let mut ret = vec![tile];
        ret.extend(tiles);
        ret
    }
}

fn parse_deltas<'a>(sc: Option<Attribute<'a>>) -> Result<Option<[i32; 4]>, quick_xml::Error> {
    let deltas = sc.map(|a| {
        a.decode_and_unescape_value(Decoder {})
    })
    .transpose()?;

    let deltas: Vec<i32> = deltas
        .iter()
        .flat_map(|s| s.split(",").enumerate())
        .filter(|&(i, _)| i % 2 != 0)
        .map(|(_, s)| s.parse::<i32>().unwrap() * 100)
        .collect();

    let deltas: Option<[i32; 4]> = match deltas.len() {
        4 => Some(deltas.try_into().unwrap()),
        _ => None,
    };

    Ok(deltas)
}

pub fn parse_mjloggm_version<'a>(e: &BytesStart<'a>) -> Result<String, quick_xml::Error> {
    let version = e.try_get_attribute("ver")?
        .expect("Failed to parse 'ver' attribute.")
        .decode_and_unescape_value(Decoder {})?
        .into_owned();
    Ok(version)
}

pub fn parse_game_type<'a>(e: &BytesStart<'a>) -> Result<(bool, bool), quick_xml::Error> {
    let game_type = e.try_get_attribute("type")?
        .expect("Failed to parse 'type' attribute.")
        .decode_and_unescape_value(Decoder {})?
        .parse::<u16>()
        .unwrap();
    let aka_flag: bool = (game_type & 0x02) >> 2 != 0;
    let is_sanma: bool = (game_type & 0x10) >> 4 != 0;
    Ok((!aka_flag, is_sanma))
}

pub fn parse_names<'a>(e: &BytesStart<'a>) -> Result<(String, String, String, String), quick_xml::Error> {
    let mut names = vec![String::new(); 4];
    for i in 0..4 {
        let name = e
            .try_get_attribute(&format!("n{}", i))?
            .expect("Failed to parse player names.")
            .decode_and_unescape_value(Decoder {})?
            .into_owned();
        let name = decode(&name).unwrap();
        names[i] = name.into_owned();
    }
    Ok((names[0].clone(), names[1].clone(), names[2].clone(), names[3].clone()))
}

pub fn parse_init_others<'a>(e: &BytesStart<'a>, aka_flag: bool) -> Result<(Tile, Tile, u8, u8, u8, u8), quick_xml::Error> {
    let seed_values = e.try_get_attribute("seed")?
        .expect("Failed to parse 'seed' attribute.")
        .decode_and_unescape_value(Decoder {})?
        .split(',')
        .map(|s| s.parse::<u8>().unwrap())
        .collect::<Vec<u8>>();

    let bakaze_index = (seed_values[0] / 4) % 4;
    let bakaze_tile = [109, 113, 117, 121][bakaze_index as usize];
    let bakaze = translate_mjlog_tile(bakaze_tile, aka_flag).unwrap();

    let dora_tile = seed_values[5];
    let dora = translate_mjlog_tile(dora_tile, aka_flag).unwrap();

    let kyoku = (seed_values[0] % 4) + 1;
    let honba = seed_values[1];
    let kyotaku = seed_values[2];

    let oya = e.try_get_attribute("oya")?
        .expect("Failed to parse 'oya' attribute.")
        .decode_and_unescape_value(Decoder {})?
        .parse::<u8>().unwrap();

    Ok((bakaze, dora, kyoku, honba, kyotaku, oya))
}

pub fn parse_init_scores(e: &BytesStart<'_>) -> Result<[i32; 4], quick_xml::Error> {
    // NOTE: Old logs do not have 'ten' values.
    let scores = e
        .try_get_attribute("ten")?
        .expect("Failed to parse 'ten' attribute. Possibly this mjlog file is too old to parse.")
        .decode_and_unescape_value(Decoder {})?;
    let scores: [i32; 4] = scores
        .split(',')
        .map(|s| s.parse::<i32>().unwrap() * 100)
        .collect::<Vec<i32>>()
        .try_into()
        .unwrap();
    Ok(scores)
}

pub fn parse_init_tehais<'a>(e: &BytesStart<'a>, aka_flag: bool) -> Result<[[Tile; 13]; 4], quick_xml::Error> {
    let mut tehais = [[Tile::default(); 13]; 4];
    for i in 0..4 {
        let attr_name = format!("hai{}", i);
        if let Some(attribute) = e.try_get_attribute(attr_name.as_str())? {
            let tiles = attribute.decode_and_unescape_value(Decoder {})?
                .split(',')
                .map(|s| {
                    let tile = s.parse::<u8>().unwrap();
                    mjlog::translate_mjlog_tile(tile, aka_flag).unwrap()
                })
                .collect::<Vec<Tile>>();
            tehais[i] = tiles.try_into().unwrap();
        }
    }
    Ok(tehais)
}

pub fn parse_dora<'a>(e: &BytesStart<'a>, aka_flag: bool) -> Result<Tile, quick_xml::Error> {
    let dora_tile = e
        .try_get_attribute("hai")?
        .expect("Failed to parse 'hai' attribute.")
        .decode_and_unescape_value(Decoder {})?;
    let dora_tile = dora_tile.parse::<u8>().unwrap();
    let dora = mjlog::translate_mjlog_tile(dora_tile, aka_flag).unwrap();
    Ok(dora)
}

pub fn parse_n<'a>(e: &BytesStart<'a>, aka_flag: bool) -> Result<(String, u8, u8, Vec<Tile>), quick_xml::Error> {
    let caller = e
        .try_get_attribute("who")?
        .expect("Failed to parse 'who' attribute.")
        .decode_and_unescape_value(Decoder {})?
        .parse::<u8>()
        .unwrap();
    let meld = e
        .try_get_attribute("m")?
        .expect("Failed to parse 'm' attribute.")
        .decode_and_unescape_value(Decoder {})?
        .parse::<u16>()
        .unwrap();
    let callee_rel = (meld & 0x3) as u8;
    let callee = (caller + callee_rel) % 4;

    let (call_type, mianzi): (&str, Vec<u8>) =
        if meld & (1 << 2) != 0 {
            ("Chi", parse_chi(meld))
        } else if meld & (1 << 3) != 0 {
            ("Pon", parse_pon(meld))
        } else if meld & (1 << 4) != 0 {
            ("Kakan", parse_kakan(meld))
        } else {
            (if callee_rel == 1 { "Minkan" } else { "Ankan" }, parse_kan(meld))
        };

    let tiles = mianzi
        .iter()
        .map(|&x| mjlog::translate_mjlog_tile(x, aka_flag).unwrap())
        .collect::<Vec<Tile>>();

    Ok((call_type.to_string(), caller, callee, tiles))
}

pub fn parse_reach<'a>(e: &BytesStart<'a>) -> Result<(u8, u8), quick_xml::Error> {
    let who = e
        .try_get_attribute("who")?
        .expect("Failed to parse 'who' attribute.")
        .decode_and_unescape_value(Decoder {})?;
    let who = who.parse::<u8>().unwrap();
    let step = e
        .try_get_attribute("step")?
        .expect("Failed to parse 'step' attribute.")
        .decode_and_unescape_value(Decoder {})?;
    let step = step.parse::<u8>().unwrap();
    Ok((who, step))
}

pub fn parse_agari<'a>(e: &BytesStart<'a>, aka_flag: bool) -> Result<(u8, u8, Option<Vec<Tile>>, Option<[i32; 4]>), quick_xml::Error> {
    let who = e
        .try_get_attribute("who")?
        .expect("Failed to parse 'who' attribute.")
        .decode_and_unescape_value(Decoder {})?;
    let who = who.parse::<u8>().unwrap();

    let from_who = e
        .try_get_attribute("fromWho")?
        .expect("Failed to parse 'fromWho' attribute.")
        .decode_and_unescape_value(Decoder {})?;
    let from_who = from_who.parse::<u8>().unwrap();

    let dora_ura_attribute = e.try_get_attribute("doraHaiUra")?;
    let ura_markers: Option<Vec<Tile>> = if let Some(dora_ura_attribute) = dora_ura_attribute {
        let dora_ura_str = dora_ura_attribute.decode_and_unescape_value(Decoder {})?;
        let ura_markers: Vec<Tile> = dora_ura_str
            .split(",")
            .map(|s| -> Tile {
                let tile = s.parse::<u8>().unwrap();
                mjlog::translate_mjlog_tile(tile, aka_flag).unwrap()
            })
            .collect::<Vec<Tile>>();
        Some(ura_markers)
    } else {
        None
    };

    let sc_attribute = e.try_get_attribute("sc")?;
    let deltas = parse_deltas(sc_attribute)?;
    Ok((who, from_who, ura_markers, deltas))
}

pub fn check_if_owari<'a>(e: &BytesStart<'a>) -> Result<bool, quick_xml::Error> {
    let owari_attribute = e.try_get_attribute("owari")?;
    Ok(owari_attribute.is_some())
}

pub fn parse_ryuukyoku<'a>(e: &BytesStart<'a>) -> Result<Option<[i32; 4]>, quick_xml::Error> {
    let sc_attribute = e.try_get_attribute("sc")?;
    let deltas = parse_deltas(sc_attribute)?;
    Ok(deltas)
}
