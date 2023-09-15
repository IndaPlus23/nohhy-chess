use std::fmt;

struct Board {
    squares : [[Option<Piece>; 12] ; 12],
    turn : Color,
    kingside_castle_white : bool,
    queenside_castle_white : bool,
    kingside_castle_black : bool,
    queenside_castle_black : bool,
    en_passant_square : Option<[usize ; 2]>,
    half_moves : u32,
    full_moves : u32,
}

impl fmt::Debug for Board {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();

        for i in 2..10 {
            for j in 2..10 {
                match self.squares[i][j] {
                    Some(piece) => str.push(get_repr(piece)),
                    None => str.push('.'),
                }
            }
            str.push('\n');
        }

        write!(f, "{}", str)
    }
}

impl Board {
    fn new() -> Board {
        Board {
            squares : [[Option::None ; 12] ; 12], // >:(
            turn : Color::White,
            kingside_castle_white : true,
            queenside_castle_white : true,
            kingside_castle_black : true,
            queenside_castle_black : true,
            en_passant_square : None,
            half_moves : 0,
            full_moves : 0,
        }
    }

    fn from(fen_str : &str) -> Result<Board, String> {
        //Splits up FEN string to the seprate fields
        //For FEN format see https://en.wikipedia.org/wiki/Forsyth–Edwards_Notation
        let fen_fields = fen_str
            .split_whitespace()
            .collect::<Vec<&str>>();

        let mut board = Board::new();

        //Map piece placement string to Board
        let mut j : usize = 2;
        
        for (i, row) in fen_fields[0].split("/").enumerate() {
            for chr in row.chars() {
                match chr.to_digit(10) {
                    Some(num) => {
                        j += num as usize;
                        continue;
                    },
                    None => {
                        board.squares[i + 2][j] = Some(get_piece(chr)?);
                    },
                }
                j += 1;
            }
            j = 2;
        }

        //Map active turn string to Board
        board.turn = match fen_fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _c => return Err(format!("Invalid active field {}", _c)),
        };

        //Map Casteling rights string to Board
        for c in fen_fields[2].chars() {
            match c {
                'K' => board.kingside_castle_white = true,
                'Q' => board.queenside_castle_white = true,
                'k' => board.kingside_castle_black = true,
                'q' => board.queenside_castle_black = true,
                '-' => continue,
                _c => return Err(format!("Invalid casteling field {}", _c)),
            } 
        }

        //Map en passant string to Board
        match fen_fields[3] {
            "-" => {},
            _ => {
                    board.en_passant_square = match alg_notation_to_indx(fen_fields[3]) {
                    Ok(indx) => Some(indx),
                    Err(e) => return Err(e.to_string()),
                };
            }
        }

        //Parse number of half-moves to Board
        board.half_moves = match fen_fields[4].parse::<u32>() {
            Ok(num) => num,
            Err(e) => return Err(e.to_string()),
        };

        //Parse number of full-moves to Board
        board.full_moves = match fen_fields[5].parse::<u32>() {
            Ok(num) => num,
            Err(e) => return Err(e.to_string()),
        };

        return Result::Ok(board);
    }
}

#[derive(Clone, Copy, Debug)]
struct Piece {
    piece_type : PieceType,
    color : Color,
}

impl Piece {
    fn new(piece_type : PieceType, color : Color) -> Piece{
        Piece {
            piece_type,
            color,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
#[derive(Clone, Copy, Debug)]
enum Color {
    White,
    Black,
}

fn get_piece(chr : char) -> Result<Piece, String> {
    match chr {
        'P' => Ok(Piece::new(PieceType::Pawn, Color::White)),
        'N' => Ok(Piece::new(PieceType::Knight, Color::White)),
        'B' => Ok(Piece::new(PieceType::Bishop, Color::White)),
        'R' => Ok(Piece::new(PieceType::Rook, Color::White)),
        'Q' => Ok(Piece::new(PieceType::Queen, Color::White)),
        'K' => Ok(Piece::new(PieceType::King, Color::White)),
        'p' => Ok(Piece::new(PieceType::Pawn, Color::Black)),
        'n' => Ok(Piece::new(PieceType::Knight, Color::Black)),
        'b' => Ok(Piece::new(PieceType::Bishop, Color::Black)),
        'r' => Ok(Piece::new(PieceType::Rook, Color::Black)),
        'q' => Ok(Piece::new(PieceType::Queen, Color::Black)),
        'k' => Ok(Piece::new(PieceType::King, Color::Black)),
        e => Err(e.to_string())
    }       
}

fn get_repr(piece : Piece) -> char {
    match piece.color {
        Color::White => match piece.piece_type {
            PieceType::Pawn => 'P', 
            PieceType::Knight => 'N', 
            PieceType::Bishop => 'B', 
            PieceType::Rook => 'R', 
            PieceType::Queen => 'Q', 
            PieceType::King => 'K', 
        }
        Color::Black => match piece.piece_type {
            PieceType::Pawn => 'p', 
            PieceType::Knight => 'n', 
            PieceType::Bishop => 'b', 
            PieceType::Rook => 'r', 
            PieceType::Queen => 'q', 
            PieceType::King => 'k', 
        }
    }
}

fn alg_notation_to_indx(notation : &str) -> Result<[usize ; 2], String> {
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
        _c => return Err(format!("Invalid column {}", _c)),
    };

    let row : usize = chr_vec[0].to_digit(10).unwrap() as usize - 1;

    return Ok([row, col]);
}

fn main() {
    let board = Board::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    println!("{:?}", board);
}
