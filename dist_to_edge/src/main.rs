use std::io;
use std::cmp;

fn main() {
    let mut ab_str = String::new();

    println!("Input width and height: ");

    io::stdin()
        .read_line(&mut ab_str)
        .expect("Invalid input");


    let ab_vec : Vec<&str> = ab_str
        .trim()
        .split_whitespace()
        .collect();

    if ab_vec.len() == 2{
        let a = ab_vec[0].parse::<u32>().expect("Invalid input for width");
        let b = ab_vec[1].parse::<u32>().expect("Invalid input for height");

        for row in 1..=a{
            for col in 1..=b{
                let dist_to_edge = cmp::min(
                    cmp::min(row, a - row + 1),
                    cmp::min(col, b - col + 1)
                );
                
                if dist_to_edge > 9 {
                    print!(".");
                } else {
                    print!("{}", dist_to_edge);
                }
            }
            println!("");
        }
    }

}
