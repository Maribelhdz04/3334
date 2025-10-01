#[derive(Debug, Clone)]
struct Student {
    name: String,
    major: String,
}

impl Student {
    fn new(name: &str, major: &str) -> Self {
        Self { name: name.to_string(), major: major.to_string() }
    }

    fn set_major(&mut self, new_major: &str) {
        self.major = new_major.to_string();
    }

    fn get_major(&self) -> &str {
        &self.major
    }

    // NEW: read the name so the field isn't “dead”
    fn get_name(&self) -> &str {
        &self.name
    }
}

fn main() {
    let mut s = Student::new("Maribel Hernandez", "Undeclared");

    println!("Student name: {}", s.get_name()); // <— uses `name`
    println!("Before: {s:?}");
    println!("Current major: {}", s.get_major());

    s.set_major("Computer Science");

    println!("After:  {s:?}");
    println!("Updated major: {}", s.get_major());
}
