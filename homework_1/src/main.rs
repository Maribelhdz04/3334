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

// Assignment 2: Number Analyzer
fn is_even(n: i32) -> bool {
    n % 2 == 0 // true if divisible by 2
}

fn assignment2() {
    let numbers: [i32; 10] = [3, 5, 8, 12, 15, 20, 21, 30, 42, 55];

    // for loop → analyze each number
    for n in numbers {
        if n % 3 == 0 && n % 5 == 0 {
            println!("{} → FizzBuzz", n);
        } else if n % 3 == 0 {
            println!("{} → Fizz", n);
        } else if n % 5 == 0 {
            println!("{} → Buzz", n);
        } else if is_even(n) {
            println!("{} → Even", n);
        } else {
            println!("{} → Odd", n);
        }
    }

    // while loop → sum of numbers
    let mut sum = 0;
    let mut i = 0;
    while i < numbers.len() {
        sum += numbers[i];
        i += 1;
    }
    println!("Sum of numbers = {}", sum);

    // loop → largest number
    let mut max = numbers[0];
    let mut j = 0;
    loop {
        if numbers[j] > max {
            max = numbers[j];
        }
        j += 1;
        if j >= numbers.len() {
            break;
        }
    }
    println!("Largest number = {}", max);
}

// Assignment 3: Guessing Game
fn check_guess(guess: i32, secret: i32) -> i32 {
    if guess == secret {
        0 // correct
    } else if guess > secret {
        1 // too high
    } else {
        -1 // too low
    }
}

fn assignment3() {
    let secret: i32 = 42; // hard-coded secret
    let mut guess: i32 = 30; // start guess
    let mut attempts: i32 = 0; // track guesses

    loop {
        attempts += 1;
        let result = check_guess(guess, secret);

        if result == 0 {
            println!("Guess {} → Correct!", guess);
            break; // exit loop
        } else if result == 1 {
            println!("Guess {} → Too high", guess);
            guess -= 1; // change guess
        } else {
            println!("Guess {} → Too low", guess);
            guess += 1; // change guess
        }
    }

    println!("It took {} guesses.", attempts);
}

fn main() {
    println!("--- Assignment 1 ---");
    assignment1();

    println!("\n--- Assignment 2 ---");
    assignment2();

    println!("\n--- Assignment 3 ---");
    assignment3();
}
