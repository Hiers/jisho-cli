use colored::*;
use serde_json::Value;

use std::env;

macro_rules! JISHO_URL {
    () => {
        "https://jisho.org/api/v1/search/words?keyword={}"
    };
}
const ITEM_LIMIT: usize = 4;

fn main() -> Result<(), ureq::Error> {
    // Get all parameter into one space separated query
    let query = env::args().skip(1).collect::<Vec<String>>().join(" ");

    // Check query not being empty
    if query.is_empty() {
        println!(
            "Usage: {} [<Keywords>]",
            get_exec_name().unwrap_or_else(|| "jisho-cli".to_owned())
        );

        return Ok(());
    }

    // Do API request
    let body: Value = ureq::get(&format!(JISHO_URL!(), query))
        .call()?
        .into_json()?;

    // Try to get the data json-object
    let body = value_to_arr({
        let body = body.get("data");

        if body.is_none() {
            eprintln!("Error! Invalid response");
            return Ok(());
        }

        body.unwrap()
    });

    // Iterate over meanings and print them
    for (i, entry) in body.iter().enumerate() {
        if i > ITEM_LIMIT {
            break;
        }

        print_item(&query, entry);
        println!();
    }

    Ok(())
}

fn print_item(query: &str, value: &Value) {
    let japanese = value_to_arr(value.get("japanese").unwrap())
        .get(0)
        .unwrap()
        .to_owned();

    let reading = japanese
        .get("reading")
        .map(|i| value_to_str(i))
        .unwrap_or(query);

    let word = value_to_str(japanese.get("word").unwrap());

    println!("{}[{}] {}", word, reading, format_tags(value));

    // Print senses
    let senses = value.get("senses").unwrap();
    for (i, sense) in value_to_arr(senses).iter().enumerate() {
        let sense_str = format_sense(&sense, i);
        if sense_str.is_empty() {
            continue;
        }

        println!(" {}", sense_str);
    }
}

fn format_sense(value: &Value, index: usize) -> String {
    let english_definitons = value.get("english_definitions");
    if english_definitons.is_none() {
        return "".to_owned();
    }

    let english_definiton = value_to_arr(english_definitons.unwrap());

    format!(
        "{}. {}",
        index,
        value_to_str(english_definiton.get(0).unwrap())
    )
}

fn format_tags(value: &Value) -> String {
    let mut builder = String::new();

    let is_common_val = value.get("is_common");
    if is_common_val.is_some() && value_to_bool(is_common_val.unwrap()) {
        builder.push_str(&"(common) ".bright_green().to_string());
    }

    if let Some(jlpt) = value.get("jlpt") {
        let jlpt = value_to_arr(&jlpt);
        if !jlpt.is_empty() {
            let jlpt = value_to_str(jlpt.get(0).unwrap())
                .replace("jlpt-", "")
                .to_uppercase();
            builder.push_str(&format!("({}) ", jlpt.bright_blue().to_string()));
        }
    }

    builder
}

//
// --- Value helper
//

fn value_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        _ => unreachable!(),
    }
}

fn value_to_str(value: &Value) -> &str {
    match value {
        Value::String(s) => s,
        _ => unreachable!(),
    }
}

fn value_to_arr<'a>(value: &'a Value) -> &'a Vec<Value> {
    match value {
        Value::Array(a) => a,
        _ => unreachable!(),
    }
}

fn get_exec_name() -> Option<String> {
    std::env::current_exe()
        .ok()
        .and_then(|pb| pb.file_name().map(|s| s.to_os_string()))
        .and_then(|s| s.into_string().ok())
}
