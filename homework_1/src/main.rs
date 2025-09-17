// Assignment 1: Temperature Converter

const FREEZING_POINT_F: f64 = 32.0; // constant

fn fahrenheit_to_celsius(f: f64) -> f64 {
    (f - FREEZING_POINT_F) * 5.0 / 9.0   // return result
}

fn celsius_to_fahrenheit(c: f64) -> f64 {
    (c * 9.0 / 5.0) + FREEZING_POINT_F   // return result
}

fn assignment1() {
    let mut temp_f: f64 = 32.0; // start value

    println!("{} F = {} C", temp_f, fahrenheit_to_celsius(temp_f));

    // loop 5 times
    let mut i = 0;
    while i < 5 {
        temp_f += 1.0; // add 1
        println!("{} F = {} C", temp_f, fahrenheit_to_celsius(temp_f));
        i += 1; // counter
    }

    // extra test for Celsius → Fahrenheit
    let c: f64 = 100.0;
    println!("{} C = {} F", c, celsius_to_fahrenheit(c));
}

fn main() {
    assignment1();
}
