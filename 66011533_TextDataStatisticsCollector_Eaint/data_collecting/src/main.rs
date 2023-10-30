use std::env;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Write};
use csv::Writer;
use std::error::Error;

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Document {
    name: String,
    content: String,
    word_count: usize,
    line_count: usize,
    avg_wordlen: f64,
    com_words: HashMap<String, usize>,
    cha_count: usize,
}

impl Document {
    fn make_document(file: &str, text: &str) -> Document {

        Document {
            name: file.to_string(),
            content: text.to_string(),
            word_count: Self::count_words(text),
            line_count: Self::line_count(text),
            cha_count: Self::count_characters(text),
            avg_wordlen: Self::calculate_average_word_length(text),
            com_words: Self::find_most_common_words(text),
        }
    }
    
    fn count_words(text: &str) -> usize {
        let mut count = 0;
        let mut in_word = false;
    
        for c in text.chars() {
            if c.is_whitespace() {
                in_word = false;
            } else if !in_word {
                count += 1;
                in_word = true;
            }
        }
    
        count
    }

    fn count_characters(text: &str) -> usize {
        let mut count = 0;
    
        for c in text.chars() {
            if !c.is_whitespace() {
                count += 1;
            }
        }
    
        count
    }
    
    
    fn line_count(text: &str) -> usize {
        let mut count = 0;
    
        for c in text.chars() {
            if c == '\n' {
                count += 1;
            }
        }
    
        // Add 1 to count the last line if it doesn't end with a newline character
        if !text.is_empty() && !text.ends_with('\n') {
            count += 1;
        }
    
        count
    }

    fn calculate_average_word_length(text: &str) -> f64 {

        let words: Vec<&str> = text.split_whitespace().collect();
        let total_word_length: usize = words.iter().map(|&word| word.chars().count()).sum();
        if words.is_empty() {
            0.0
        }
        
        else {
            total_word_length as f64 / words.len() as f64
        }
    }

    fn is_valid_word(word: &str) -> bool {
        // Define a function to check if a word is valid (e.g., no special characters)
        word.chars().all(char::is_alphanumeric)
    }
    
    fn clean_word(word: &str) -> String {
        // Define a function to clean a word by removing non-alphanumeric characters
        word.chars().filter(|&c| c.is_alphanumeric()).collect()
    }
    
    fn find_most_common_words(text: &str) -> HashMap<String, usize> {
        let mut word_count = HashMap::new();
    
        // Split the text into words, clean them, and count
        for word in text.split_whitespace() {
            let cleaned_word = Self::clean_word(word);
            if Self::is_valid_word(&cleaned_word) {
                let count = word_count.entry(cleaned_word.clone()).or_insert(0);
                *count += 1;
            }
        }
    
        let mut sorted_counts: Vec<(String, usize)> = word_count.into_iter().collect();
        sorted_counts.sort_by(|a, b| b.1.cmp(&a.1));
    
        sorted_counts.into_iter().take(5).collect()
    }

    fn rank_by_word_count(unranked_docs: &[Document]) -> Vec<Document> {

        let mut ranked_docs = unranked_docs.to_vec();
        ranked_docs.sort_by(|a, b| b.word_count.cmp(&a.word_count));
        ranked_docs
    }

    fn rank_by_line_count(unranked_docs: &[Document]) -> Vec<Document> {

        let mut ranked_docs = unranked_docs.to_vec();
        ranked_docs.sort_by(|a, b| b.line_count.cmp(&a.line_count));
        ranked_docs
    }

    fn rank_by_cha_count(unranked_docs: &[Document]) -> Vec<Document> {

        let mut ranked_docs = unranked_docs.to_vec();
        ranked_docs.sort_by(|a, b| b.cha_count.cmp(&a.cha_count));
        ranked_docs
    }

    fn calculate_aggregate_stats(documents: &[Document]) -> (usize, usize, usize, f64, HashMap<String, usize>) {

        let total_word_count: usize = documents.iter().map(|doc| doc.word_count).sum();
        let total_character_count: usize = documents.iter().map(|doc| doc.cha_count).sum();
        let total_line_count: usize = documents.iter().map(|doc| doc.line_count).sum();
    
        // Calculate the total sum of average word lengths across all documents
        let total_avg_word_length: f64 = documents.iter().map(|doc| doc.avg_wordlen).sum();

        // Calculate the average of average word lengths by dividing the total by the number of documents
        let avg_word_length = if !documents.is_empty() {
            total_avg_word_length / documents.len() as f64
        } 
        else {
            0.0
        };

        let mut aggregate_common_words = HashMap::new();
        for doc in documents {
            for (word, count) in &doc.com_words {
                *aggregate_common_words.entry(word.to_string()).or_insert(0) += count;
            }
        }

        let mut sorted_counts: Vec<(String, usize)> = aggregate_common_words.into_iter().collect();
        sorted_counts.sort_by(|a,b| b.1.cmp(&a.1));
        sorted_counts.truncate(10);

        (total_word_count, total_character_count, total_line_count, avg_word_length, sorted_counts.into_iter().collect())
    }


    fn generate_html(ranked: &[Document], sort_method: &str) -> String {
        let mut html_table = String::new();

        html_table.push_str(&format!("<h3>Ranked Documents by {}</h3>", sort_method));
        html_table.push_str("<table>\n");
        html_table.push_str("<table border=\"1\" style=\"text-align: right;\">\n");
        html_table.push_str("<tr><th>File</th><th>Word count</th><th>Character count</th><th>Line count</th><th>Average word length</th><th>Most common words</th></tr>\n");
        
        for doc in ranked {
            html_table.push_str("<tr>");
            html_table.push_str(&format!("<td>{}</td>", doc.name));
            html_table.push_str(&format!("<td>{}</td>", doc.word_count));
            html_table.push_str(&format!("<td>{}</td>", doc.cha_count));
            html_table.push_str(&format!("<td>{}</td>", doc.line_count));
            html_table.push_str(&format!("<td>{:.2}</td>", doc.avg_wordlen));
            html_table.push_str("<td>");

            for (word, count) in &doc.com_words {
                html_table.push_str(&format!("{}: {}<br>", word, count));
            }
            html_table.push_str("</td>");
            html_table.push_str("</tr>\n");
        }

        let (total_word_count, total_character_count, total_line_count, total_word_length, sorted_counts) =
            Self::calculate_aggregate_stats(ranked);

        html_table.push_str("<tr>");
        html_table.push_str(&format!("<td><b>Total</b></td>"));
        html_table.push_str(&format!("<td>{}</td>", total_word_count));
        html_table.push_str(&format!("<td>{}</td>", total_character_count));
        html_table.push_str(&format!("<td>{}</td>", total_line_count));
        html_table.push_str(&format!("<td>{:.2}</td>", total_word_length));
        html_table.push_str("</tr>");
        html_table.push_str("</table>\n");
        html_table.push_str("<br/><br/><br/>");
        html_table.push_str("<h1>Most Common Words in Folder</h1>");

        
        let mut html_histogram= String::new();
        
        html_histogram.push_str("<style>");
        html_histogram.push_str(".bar {width: 25px; background-color: black;display: inline-block;margin:30px;}");
        html_histogram.push_str("span {display: inline;margin-left: -40px;margin-top: 5px;width:25px;}");
        
        for (word, count) in &sorted_counts {
        html_histogram.push_str( format!(".hist-{}",word).as_str());
        html_histogram.push_str("{");
        html_histogram.push_str(format!("height: {}px;", count*5).as_str());
        html_histogram.push_str("}");
       }

       html_histogram.push_str("</style>");
       for (word,_count) in &sorted_counts {

        html_histogram.push_str("<div class='bar ");
        html_histogram.push_str(format!("hist-{}",word).as_str());
        html_histogram.push_str("'></div>");
        html_histogram.push_str(format!("<span>{}</span>", word).as_str());
       }

        html_table.push_str(html_histogram.as_str());

        html_table
    }
}

fn find_word_in_folder(folder_path: &str, search_word: &str) -> Result<(), Box<dyn Error>> {
    let folder = Path::new(folder_path);
    if !folder.is_dir() {
        eprintln!("The provided path is not a directory.");
        std::process::exit(1);
    }

    let mut results = Vec::new();

    for entry in fs::read_dir(folder)? {
        if let Ok(entry) = entry {
            let file_path: PathBuf = entry.path();
            if file_path.is_dir() {
                continue;
            }
            if let Some(extension) = file_path.extension() {
                if extension.to_string_lossy().to_lowercase() == "txt" {
                    println!("Found text");
                    let file_contents = fs::read_to_string(&file_path)?;
                    let mut line_number = 1;

                    for line in file_contents.lines() {
                        if line.contains(search_word) {
                            results.push((file_path.to_string_lossy().to_string(), line_number));
                        }
                        line_number += 1;
                    }
                }
            }
        }
    }

    if results.is_empty() {
        println!("The word '{}' was not found in any file in the folder.", search_word);
    } else {
        let csv_file = "word_location.csv";
        let mut csv_writer = Writer::from_path(csv_file)?;
        csv_writer.write_record(&["File Name", "Line Number"])?;
        for (file_name, line) in &results {
            csv_writer.write_record(&[file_name, &line.to_string()])?;
        }
        println!("Search results have been written to 'word_location.csv'");
        return Ok(());
    }

    Ok(())
}

fn main() {
    
    let args: Vec<String> = env::args().collect();
    let folder_path = if args.len() < 2 { "" } else { &args[1] };
    let sort_method = if args.len() < 3 { "" } else { &args[2] };
    if folder_path.is_empty() {
        eprintln!("Usage: {} <folder_path> <sort_method>", args[0]);
        eprintln!("Available sort methods: word, line, cha");
        std::process::exit(1);
    }

    if args.len() >= 3 && args[2] == "find_word" {
        let search_word = if args.len() >= 4 { &args[3] } else { "" };
        if search_word.is_empty() {
            eprintln!("Usage: {} <folder_path> find_word <search_word>", args[0]);
            std::process::exit(1);
        }
        match find_word_in_folder(folder_path, search_word) {
            Ok(_) => {
                println!("Search completed.");
                std::process::exit(0);
            }
            Err(err) => {
                eprintln!("Error during search: {}", err);
                std::process::exit(1);
            }
        }
    }

    let folder = Path::new(folder_path);
    if !folder.is_dir() {
        eprintln!("The provided path is not a directory.");
        std::process::exit(1);
    }

    let mut unranked_docs = Vec::new();
    for entry in fs::read_dir(folder).expect("Failed to read directory") {
        if let Ok(entry) = entry {
            let file_path: PathBuf = entry.path();
            if file_path.is_dir() {
                continue;
            }
            if let Some(extension) = file_path.extension() {
                if extension == "txt" {
                    let file_contents = fs::read_to_string(&file_path).expect("Failed to read file");
                    let document = Document::make_document(file_path.to_string_lossy().as_ref(), &file_contents);
                    unranked_docs.push(document);
                }
            }
        }
    }

    let ranked_docs = match sort_method {
        "word" => Document::rank_by_word_count(&unranked_docs),
        "line" => Document::rank_by_line_count(&unranked_docs),
        "cha" => Document::rank_by_cha_count(&unranked_docs),
        _ => {
            eprintln!("Invalid sorting method: {}. Available sort methods: word, line, cha", sort_method);
            std::process::exit(1);
        }
    };

    // Generate and save HTML file
    let html_table = Document::generate_html(&ranked_docs, sort_method);
    let output_file_name = format!("{}count_ranked_docs.html", sort_method);
    let mut output_file = File::create(output_file_name).expect("Failed to create the output file");
    write!(output_file, "{}", html_table).expect("Failed to write to the output file");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_count_words() {
        assert_eq!(Document::count_words("This is a test 1"), 5);
        assert_eq!(Document::count_words("This is a test"), 4);
        assert_eq!(Document::count_words("Word1, Word2, Word3"), 3);
        assert_eq!(Document::count_words("  Multiple    spaces   "), 2);
        assert_eq!(Document::count_words(""), 0);
    }

    #[test]
    fn test_count_characters() {
        assert_eq!(Document::count_characters("This is a test"), 11);
        assert_eq!(Document::count_characters("Word1, Word2, Word3"), 17);
        assert_eq!(Document::count_characters("  Multiple    spaces   "), 14);
        assert_eq!(Document::count_characters(""), 0);
}


    #[test]
    fn test_line_count() {
        assert_eq!(Document::line_count("This is a test\nWith multiple lines\nis it correct\n"), 3);
        assert_eq!(Document::line_count("This is a test\nWith multiple lines\n"), 2);
        assert_eq!(Document::line_count("No newline character"), 1);
        assert_eq!(Document::line_count(""), 0);
    }

    #[test]
    fn test_calculate_average_word_length() {
        assert_eq!(Document::calculate_average_word_length("This is a test"), 2.75);
        assert_eq!(Document::calculate_average_word_length("Word1, Word2, Word3"), 5.666666666666667);
        assert_eq!(Document::calculate_average_word_length("  Multiple    spaces   "), 7.0);
        assert_eq!(Document::calculate_average_word_length(""), 0.0);
    }


    #[test]
    fn test_is_valid_word() {
        assert!(Document::is_valid_word("ValidWord"));
        assert!(!Document::is_valid_word("Invalid@Word"));
        assert!(Document::is_valid_word(""));
    }


    #[test]
    fn test_clean_word() {
        assert_eq!(Document::clean_word("CleanWord123"), "CleanWord123");
        assert_eq!(Document::clean_word("Word#1"), "Word1");
        assert_eq!(Document::clean_word(""), "");
    }


}
