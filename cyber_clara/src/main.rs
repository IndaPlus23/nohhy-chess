use std::collections::HashSet;
fn main() {
    
    // let lines = io::stdin()
    //     .lines()
    //     .map(|_line| _line.ok().unwrap())
    //     .collect::<Vec<String>>();

    let lines = vec![
        String::from("3"),
        String::from("calle"),
        String::from("gustav"),
        String::from("calle"),
        String::from("svensson"),
        String::from("nilsson"),
        String::from("svensson"),
    ];

    let num_of_names = lines[0]
        .parse::<usize>()
        .unwrap();

    let first_names : Vec<String> = lines[1..=num_of_names].to_vec();
    let last_names : Vec<String> = lines[num_of_names+1..].to_vec();

    println!("{:?}", first_names);
    println!("{:?}", last_names);

    // let num_of_names = 3;

    // let first_names : Vec<String> = [
    //     String::from("calle"),
    //     String::from("gustav"),
    //     String::from("calle"),
    // ].to_vec();

    // let last_names : Vec<String> = [
    //     String::from("svensson"),
    //     String::from("nilsson"),
    //     String::from("svensson"),
    // ].to_vec();

    let mut name_set = HashSet::new();

    for i in 0..num_of_names {
        name_set.insert(format!("{} {}", first_names[i], last_names[i]));
    }

    println!("{}", name_set.len());
}
