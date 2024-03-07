use std::error::Error;
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;

#[derive(Debug)]
pub struct BookInfo {
    pub title: String,
    pub author: String,
    pub series: Option<String>,
    pub genre: String,
    pub reader: String,
    pub cover_url: String,
}

fn get_node_content(doc: &Html, selector: &'static str) -> Result<Option<String>, Box<dyn Error>> {
    let sel = Selector::parse(selector)?;
    let val = doc.select(&sel)
        .next()
        .map(|s| s.inner_html().trim().to_string());
    Ok(val)
}

fn get_node_attribute(doc: &Html, selector: &'static str, attribute: &str) -> Result<Option<String>, Box<dyn Error>> {
    let sel = Selector::parse(selector)?;
    let val = doc.select(&sel)
        .next()
        .and_then(|s| s.value().attr(attribute))
        .map(|s| s.to_string());
    Ok(val)
}

pub fn extract_book_info(response: &str) -> Result<BookInfo, Box<dyn Error>> {
    let doc = Html::parse_document(&response);
    Ok(BookInfo {
        title: get_node_content(&doc, "span.book_title_name")?.ok_or("No title")?,
        author: get_node_content(&doc, "span.book_title_elem > span[itemprop=author] > a")?.ok_or("No author")?,
        series: Some(get_node_content(&doc, "div.book_serie_block_title > a")?.ok_or("No series")?),
        genre: get_node_content(&doc, "div.book_genre_pretitle > a")?.ok_or("No genre")?,
        reader: get_node_content(&doc, "span.book_title_elem > a")?.ok_or("No reader")?,
        cover_url: get_node_attribute(&doc, "div.book_cover > img", "src")?.ok_or("No cover")?,
    })
}

#[derive(Deserialize, Debug)]
pub struct BookChapter {
    pub duration_float: f32,
    pub title: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
struct BookPlayerArgs(
    serde_json::Value,
    Vec<BookChapter>,
    Vec<serde_json::Value>,
    Vec<serde_json::Value>,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
    serde_json::Value,
);

fn extract_book_player_args(html: &str) -> Result<BookPlayerArgs, Box<dyn Error>> {
    let re = Regex::new(r"var player = new BookPlayer\((.*)\);")?;
    let captures = re.captures(html).ok_or("No match found")?;
    let js_snippet = captures.get(1).ok_or("No match found")?;
    let json_snippet = format!("[{}]", js_snippet.as_str());
    let result: BookPlayerArgs = serde_json::from_str(&json_snippet)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_book_player_args() {
        assert!(super::extract_book_player_args("").is_err());
        assert!(super::extract_book_player_args("var player = new BookPlayer(1, [], [], [], 2, 3, 4, 5);").is_ok());
        let res = super::extract_book_player_args(r#"var player = new BookPlayer(1, [{"duration_float":1, "title":"ttl", "url":"url://"}], [], [], 2, 3, 4, 5);"#);
        assert!(res.is_ok());
        let res = res.unwrap().1;
        assert_eq!(res.len(), 1);
        let res = &res[0];
        assert_eq!(res.duration_float, 1.0);
        assert_eq!(res.title, "ttl");
        assert_eq!(res.url, "url://");
    }
}

pub fn extract_book_chapters(html: &str) -> Result<Vec<BookChapter>, Box<dyn Error>> {
    Ok(extract_book_player_args(html)?.1)
}