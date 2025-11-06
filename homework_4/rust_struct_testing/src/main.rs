mod rectangle;

use rectangle::Rectangle;

fn main() {
    let rect1 = Rectangle::new(8, 7);
    let rect2 = Rectangle::new(5, 1);

    println!("rect1: {:?}", rect1);
    println!("rect2: {:?}", rect2);
    println!("rect1 area: {}", rect1.area());
    println!("rect2 area: {}", rect2.area());
    println!("rect1 is square? {}", rect1.is_square());
    println!("rect2 is square? {}", rect2.is_square());

    println!("rect1 can hold rect2: {}", rect1.can_hold(&rect2));
    println!("rect2 can hold rect1: {}", rect2.can_hold(&rect1));
}
