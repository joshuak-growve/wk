use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{self, Write};

const HIRAGANA: &[(&str, &str)] = &[
    ("あ", "a"), ("い", "i"), ("う", "u"), ("え", "e"), ("お", "o"),
    ("か", "ka"), ("き", "ki"), ("く", "ku"), ("け", "ke"), ("こ", "ko"),
    ("さ", "sa"), ("し", "shi"), ("す", "su"), ("せ", "se"), ("そ", "so"),
    ("た", "ta"), ("ち", "chi"), ("つ", "tsu"), ("て", "te"), ("と", "to"),
    ("な", "na"), ("に", "ni"), ("ぬ", "nu"), ("ね", "ne"), ("の", "no"),
    ("は", "ha"), ("ひ", "hi"), ("ふ", "fu"), ("へ", "he"), ("ほ", "ho"),
    ("ま", "ma"), ("み", "mi"), ("む", "mu"), ("め", "me"), ("も", "mo"),
    ("や", "ya"), ("ゆ", "yu"), ("よ", "yo"),
    ("ら", "ra"), ("り", "ri"), ("る", "ru"), ("れ", "re"), ("ろ", "ro"),
    ("わ", "wa"), ("を", "wo"), ("ん", "n"),
    // Dakuten
    ("が", "ga"), ("ぎ", "gi"), ("ぐ", "gu"), ("げ", "ge"), ("ご", "go"),
    ("ざ", "za"), ("じ", "ji"), ("ず", "zu"), ("ぜ", "ze"), ("ぞ", "zo"),
    ("だ", "da"), ("ぢ", "ji"), ("づ", "zu"), ("で", "de"), ("ど", "do"),
    ("ば", "ba"), ("び", "bi"), ("ぶ", "bu"), ("べ", "be"), ("ぼ", "bo"),
    ("ぱ", "pa"), ("ぴ", "pi"), ("ぷ", "pu"), ("ぺ", "pe"), ("ぽ", "po"),
    // Small ya combos
    ("きゃ", "kya"), ("きゅ", "kyu"), ("きょ", "kyo"),
    ("しゃ", "sha"), ("しゅ", "shu"), ("しょ", "sho"),
    ("ちゃ", "cha"), ("ちゅ", "chu"), ("ちょ", "cho"),
    ("にゃ", "nya"), ("にゅ", "nyu"), ("にょ", "nyo"),
    ("ひゃ", "hya"), ("ひゅ", "hyu"), ("ひょ", "hyo"),
    ("みゃ", "mya"), ("みゅ", "myu"), ("みょ", "myo"),
    ("りゃ", "rya"), ("りゅ", "ryu"), ("りょ", "ryo"),
    ("ぎゃ", "gya"), ("ぎゅ", "gyu"), ("ぎょ", "gyo"),
    ("じゃ", "ja"), ("じゅ", "ju"), ("じょ", "jo"),
    ("びゃ", "bya"), ("びゅ", "byu"), ("びょ", "byo"),
    ("ぴゃ", "pya"), ("ぴゅ", "pyu"), ("ぴょ", "pyo"),
];

fn to_katakana(s: &str) -> String {
    s.chars()
        .map(|ch| {
            let cp = ch as u32;
            if (0x3041..=0x3096).contains(&cp) {
                std::char::from_u32(cp + 0x60).unwrap_or(ch)
            } else {
                ch
            }
        })
        .collect()
}

fn input(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut s = String::new();
    io::stdin().read_line(&mut s).ok();
    s.trim().to_string()
}

fn normalize_answer(s: &str) -> String {
    s.trim().to_lowercase()
}

fn run_rounds(pairs: &[(String, String)], kana_to_romaji: bool, rounds: Option<usize>) {
    let mut rng = thread_rng();
    let mut score = 0usize;
    let mut asked = 0usize;

    loop {
        if let Some(max) = rounds {
            if asked >= max {
                break;
            }
        }

        let (k, r) = pairs.choose(&mut rng).unwrap();
        asked += 1;

        if kana_to_romaji {
            let ans = input(&format!("{} -> ", k));
            if normalize_answer(&ans) == normalize_answer(r) {
                println!("Correct!");
                score += 1;
            } else {
                println!("Wrong — answer: {}", r);
            }
        } else {
            let ans = input(&format!("{} -> ", r));
            if normalize_answer(&ans) == normalize_answer(k) {
                println!("Correct!");
                score += 1;
            } else {
                println!("Wrong — answer: {}", k);
            }
        }

        if rounds.is_none() {
            // infinite mode: allow quit with q
            println!("Type `q` to quit, Enter to continue.");
            let cont = input(": ");
            if cont.trim().eq_ignore_ascii_case("q") {
                break;
            }
        }
    }

    println!("Finished — score: {}/{}", score, asked);
}

pub fn run() {
    println!("Kana practice — Hiragana / Katakana CLI");

    let mode = loop {
        let m = input("Mode: (h)iragana, (k)atakana, (m)mixed: ");
        match m.as_str() {
            "h" | "H" => break 'h',
            "k" | "K" => break 'k',
            "m" | "M" => break 'm',
            _ => println!("Please choose h, k, or m."),
        }
    };

    let dir = loop {
        let d = input("Direction: (1) kana -> romaji, (2) romaji -> kana: ");
        match d.as_str() {
            "1" => break 1,
            "2" => break 2,
            _ => println!("Please choose 1 or 2."),
        }
    };

    let rounds = loop {
        let r = input("Rounds (0 = infinite): ");
        match r.parse::<usize>() {
            Ok(0) => break None,
            Ok(n) => break Some(n),
            Err(_) => println!("Enter a number."),
        }
    };

    // Build selected pairs
    let mut pairs: Vec<(String, String)> = Vec::new();
    for &(h, romaji) in HIRAGANA.iter() {
        pairs.push((h.to_string(), romaji.to_string()));
    }

    if mode == 'k' {
        // convert to katakana
        let kat_pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, r)| (to_katakana(k), r.clone()))
            .collect();
        pairs = kat_pairs;
    } else if mode == 'm' {
        // add katakana variants alongside hiragana
        let mut extra: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, r)| (to_katakana(k), r.clone()))
            .collect();
        pairs.append(&mut extra);
    }

    let kana_to_romaji = dir == 1;

    run_rounds(&pairs, kana_to_romaji, rounds);
}