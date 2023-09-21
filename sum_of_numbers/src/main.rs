use std::io;

fn main() {
    let input = io::stdin();

    let mut lines = input.lines()
        .map(|_line| _line.ok().unwrap());    

    let l = f64::ceil(lines
        .next().unwrap()
        .parse::<f64>().unwrap() / 2.0) ; 

    let mut nums = lines
        .next().unwrap()
        .split_whitespace()
        .map(|val| val
        .parse::<usize>().unwrap()
        ).collect::<Vec<usize>>();
    
    nums.sort();
    nums.reverse();

    let mut res : usize = 0;

    for idx in 0..l as usize {
        res += nums[idx];
    }

    println!("{res}");
}   