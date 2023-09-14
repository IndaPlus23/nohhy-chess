struct Board {
    squares : [[Option<Piece>; 12] ; 12],
    turn : Color,
    kingside_castle_white : bool,
    queenside_castle_white : bool,
    kingside_castle_black : bool,
    queenside_castle_black : bool,
    en_passant_square : [usize ; 2],
    half_moves : u32,
    full_moves : u64,
}

impl Board {
    fn from(&mut self, fen_str : &str) -> Result<Board, &str> {
        let fen_fields = fen_str
            .split_whitespace()
            .collect::<Vec<&str>>();

        self.turn = match fen_fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _c => panic!("No color match found for {_c}"),
        };

        for c in fen_fields[2].chars() {
            match c {
                'K' => self.kingside_castle_white = true,
                'Q' => self.queenside_castle_white = true,
                'k' => self.kingside_castle_black = true,
                'q' => self.queenside_castle_black = true,
                '-' => continue,
                _c => return Result::Err("Invalid casteling field"),
            } 
        }

        Board()
    }
}

struct Piece {
    piece_type : PieceType,
    color : Color,
}

enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

enum Color {
    White,
    Black,
}

fn alg_notation_to_indx(notation : &str) -> Result<[usize ; 2], &str> {
    let chr_vec = notation
        .chars()
        .collect::<Vec<char>>();

    let col : usize = match chr_vec[0] {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _c => return Result::Err("Invalid column"),
    };

    let row : usize = chr_vec[0].to_digit(10).unwrap() as usize - 1;

    return Result::Ok([row, col]);
}

fn main() {
    println!("Hello, world!");
}
