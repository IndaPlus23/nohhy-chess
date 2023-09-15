use std::io;
use std::collections::HashSet;
fn main() {
    
    let lines = io::stdin()
        .lines()
        .map(|_line| _line.ok().unwrap())
        .collect::<Vec<String>>();

    let num_of_names = lines[0]
        .parse::<usize>()
        .unwrap();
        
    let first_names : Vec<String> = lines[1..=num_of_names].to_vec();
    let last_names : Vec<String> = lines[num_of_names+1..].to_vec();

    let mut name_set = HashSet::new();

    for i in 0..num_of_names {
        name_set.insert(format!("{} {}", first_names[i], last_names[i]));
    }

    println!("{}", name_set.len());
}
