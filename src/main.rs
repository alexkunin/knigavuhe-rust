mod book_parser;

use std::error::Error;
use crate::book_parser::{extract_book_chapters, extract_book_info};

fn main() -> Result<(), Box<dyn Error>> {
    let book_url = "https://knigavuhe.org/book/barliona/";
    println!("Book URL: {}", book_url);
    let html = reqwest::blocking::get(book_url)?.text()?;
    println!("Book Info: {:#?}", extract_book_info(&html)?);
    println!("Book Chapters: {:#?}", extract_book_chapters(&html)?);
    for chapter in extract_book_chapters(&html)? {
        println!("{}", chapter.url);
    }
    Ok(())
}
