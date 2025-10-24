use std::fs::File;
use std::io::{Write, BufReader, BufRead};

struct Book {
    title: String,
    author: String,
    year: u16,
}

fn save_books(books: &Vec<Book>, filename: &str) {
    // Create (or overwrite) the file and write one book per line: title,author,year
    let mut file = File::create(filename).expect("Failed to create file");
    for b in books {
        // Simple CSV-style line; assumes no commas in fields per the assignment
        writeln!(file, "{},{},{}", b.title, b.author, b.year)
            .expect("Failed to write to file");
    }
}

fn load_books(filename: &str) -> Vec<Book> {
    // Open file and read line-by-line, parsing "title,author,year"
    let file = match File::open(filename) {
        Ok(f) => f,
        Err(_) => return Vec::new(), // If file missing, return empty Vec
    };
    let reader = BufReader::new(file);
    let mut books = Vec::new();

    for line in reader.lines() {
        if let Ok(text) = line {
            let parts: Vec<String> = text
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();

            if parts.len() == 3 {
                if let Ok(year) = parts[2].parse::<u16>() {
                    books.push(Book {
                        title: parts[0].clone(),
                        author: parts[1].clone(),
                        year,
                    });
                } else {
                    eprintln!("Skipping line with invalid year: {}", text);
                }
            } else {
                eprintln!("Skipping malformed line: {}", text);
            }
        }
    }
    books
}

fn main() {
    let books = vec![
        Book { title: "1984".to_string(), author: "George Orwell".to_string(), year: 1949 },
        Book { title: "To Kill a Mockingbird".to_string(), author: "Harper Lee".to_string(), year: 1960 },
        Book { title: "The Pragmatic Programmer".to_string(), author: "Andrew Hunt & David Thomas".to_string(), year: 1999 },
    ];

    let filename = "books.txt";

    save_books(&books, filename);
    println!("Books saved to file: {}", filename);

    let loaded_books = load_books(filename);
    println!("Loaded books:");
    for book in loaded_books {
        println!("{} by {}, published in {}", book.title, book.author, book.year);
    }
}
//RS