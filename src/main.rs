use std::env;
use std::error::Error;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use regex::Regex;

struct Config {
    source: String,
    destination: String
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next();

        let source = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a query string"),
        };

        let destination = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a file name"),
        };

        Ok(Config { source, destination })
    }
}


fn remove_chars(s: &str, end_directory: &str) -> String {
    let mut s = s.replace(": ", " - ").replace("?", "").replace("&", "and");
    // Replace ( ) with a hyphen so "this (text)" becomes "this - text"
    s = Regex::new(r"\((.+?)\)").unwrap().replace_all(&s, "- $1").to_string();
    // Delete filename chars tht are not alphanumeric or ; , _ -
    s = Regex::new(r"[^a-zA-Z\d\s\w;,_-]+").unwrap().replace_all(&s, "").to_string();
    // Trim off anything that isn't a word at the start & end
    s = Regex::new(r"^\W+|\W+$").unwrap().replace_all(&s, "").to_string();

    let max_length = 245 - end_directory.len();
    s.chars().take(max_length).collect()
}


fn parse_clippings(source_file: &str, end_directory: &str) -> io::Result<()> {
    if !Path::new(source_file).is_file() {
        return Err(io::Error::new(io::ErrorKind::NotFound, format!("ERROR: cannot find {}", source_file)));
    }

    if !Path::new(end_directory).exists() {
        fs::create_dir_all(end_directory)?;
    }

    let mut output_files: Vec<String> = Vec::new();

    let file_content = fs::read_to_string(source_file)?;
    let highlights = file_content.split("\n==========\n");
    for highlight in highlights {
        dbg!(highlight);
        let lines: Vec<&str> = highlight.split('\n').collect();
        dbg!(&lines);
        if lines.len() < 4 {
            continue;
        }

        let title = lines[0];
        dbg!(title);
        let outfile_name = format!("{}.txt", remove_chars(&title, end_directory));
        let path_name = format!("{}/{}", end_directory, &outfile_name);
        let path = Path::new(&path_name);

        if !path.exists() {
            File::create(&path)?;
            output_files.push(path.to_string_lossy().to_string());
        } else {
            dbg!("{} existed, skipped", title);
        };

        let clipping_text = lines[3];
        dbg!("Processing: {}", clipping_text);
        let mut outfile = gen_file(&path)?;
        writeln!(outfile, "{}", clipping_text)?;
        writeln!(outfile, "\n...\n")?;
    }

    Ok(())
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Config::new(env::args())?;
    parse_clippings(&args.source, &format!("./{}", &args.destination))?;
    Ok(())
}


pub fn gen_file(path: &Path) -> Result<File, std::io::Error> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)
}