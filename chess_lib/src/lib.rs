use core::num;
use std::fmt;

pub struct Game {
    board : [[Option<Piece>; 8] ; 8],
    turn : Color,
    kingside_castle_white : bool,
    queenside_castle_white : bool,
    kingside_castle_black : bool,
    queenside_castle_black : bool,
    en_passant_square : Option<(usize, usize)>,
    half_moves : u32,
    full_moves : u32,
    rook_move_directions : Vec<(i32, i32)>,
    bishop_move_directions : Vec<(i32, i32)>,
    queen_move_directions : Vec<(i32, i32)>,
    knight_move_directions : Vec<(i32, i32)>
}

impl fmt::Debug for Game {
    fn fmt(&self, f : &mut fmt::Formatter) -> fmt::Result {
        let mut str = String::new();

        for row in self.board {
            for piece in row {
                match piece {
                    Some(piece) => str.push(get_repr(piece)),
                    None => str.push('.'),
                }
                str.push(' ');
            }
            str.push('\n');
        }

        write!(f, "{}", str)
    }
}

impl Game {

                                                                            //FOR TESTING
    fn print_with_highlights(&self, indx_vec : Vec<(usize, usize)>){
        let mut str = String::new();

        let mut i : usize = 0;
        let mut j : usize = 0;

        for row in self.board {
            j = 0;
            for piece in row {
                match piece {
                    Some(piece) => {
                        if indx_vec.contains(&(i, j)){
                            str.push('*')
                        } else { 
                            str.push(get_repr(piece))
                        }
                    },
                    None => {
                        if indx_vec.contains(&(i, j)){
                            str.push('*')
                        } else {
                            str.push('.')
                        }
                    },
                }
                str.push(' ');
                j += 1;
            }
            str.push('\n');
            i += 1;
        }

        println!("{}", str)
    }
                                                                            //FOR TESTING
    fn new_empty() -> Game {
        let rook_move_directions : Vec<(i32, i32)> = Vec::from(
            [(1, 0), (-1, 0), (0, 1), (0, -1)]
        );

        let bishop_move_directions : Vec<(i32, i32)> = Vec::from(
            [(1, 1), (1, -1), (-1, 1), (-1, -1)]
        );

        let queen_move_directions : Vec<(i32, i32)> = Vec::from(
            [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)]
        );

        let knight_move_directions : Vec<(i32, i32)> = Vec::from(
            [(2, 1), (2, -1), (-2, 1), (-2, -1), (1, 2), (-1, 2), (1, -2), (-1, -2)]
        );

        Game {
            board : [[None ; 8] ; 8], // 12x12 to simplifying move validation
            turn : Color::White,
            kingside_castle_white : true,
            queenside_castle_white : true,
            kingside_castle_black : true,
            queenside_castle_black : true,
            en_passant_square : None,
            half_moves : 0,
            full_moves : 0,
            rook_move_directions,
            bishop_move_directions,
            queen_move_directions,
            knight_move_directions,
        }
    }

    pub fn new_starting_pos() -> Game {
        Game::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Create board from FEN string
    /// For FEN format see https://en.wikipedia.org/wiki/Forsyth–Edwards_Notation
    pub fn from(fen_str : &str) -> Result<Game, String> {
        // Splits up FEN string to the seprate fields
        
        let fen_fields = fen_str
            .split_whitespace()
            .collect::<Vec<&str>>();

        let mut board = Game::new_empty();

        let mut j = 0;

        // Map piece placement string to Board
        for (i, row) in fen_fields[0].split("/").enumerate() {
            for chr in row.chars() {
                match chr.to_digit(10) {
                    Some(number) => {
                        j  += number - 1;
                    },
                    None => {
                        board.board[i][j as usize] = Some(get_piece(chr)?);
                    },
                }
                j += 1;
            }
            j = 0;
        }

        // Map active turn string to Board
        board.turn = match fen_fields[1] {
            "w" => Color::White,
            "b" => Color::Black,
            _c => return Err(format!("Invalid active field {}", _c)),
        };

        // Map Casteling rights string to Board
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

        // Map en passant string to Board
        match fen_fields[3] {
            "-" => {},
            _ => {
                    board.en_passant_square = match alg_notation_to_indx(fen_fields[3]) {
                    Ok(indx) => Some(indx),
                    Err(e) => return Err(e.to_string()),
                };
            }
        }

        // Parse number of half-moves to Board
        board.half_moves = match fen_fields[4].parse::<u32>() {
            Ok(num) => num,
            Err(e) => return Err(e.to_string()),
        };

        // Parse number of full-moves to Board
        board.full_moves = match fen_fields[5].parse::<u32>() {
            Ok(num) => num,
            Err(e) => return Err(e.to_string()),
        };

        return Result::Ok(board);
    }

    /// Move piece on the board. "from" and "to" are in algebraic notation, with turn number and piece type omitted
    /// eg from = e2, to = e4
    /// For algebraic notation see https://www.chess.com/terms/chess-notation#readalgebraic
    /// Returns Ok(true) if move is valid, Ok(false) if invalid
    pub fn make_move(&mut self, from : &str, to : &str) -> Result<bool, String> {
        let (i1, j1) = alg_notation_to_indx(from)?;
        let (i2, j2) = alg_notation_to_indx(to)?;

        self.board[i2][j2] = self.board[i1][j1];
        self.board[i1][j1] = None;

        Ok(true)
    }

    fn get_pseudo_legal_moves(&self, i : usize, j : usize) -> Result<Vec<(usize, usize)>, String>{
        if !is_valid_pos(i as i32, j as i32) {
            return Err(format!("Invalid index : Cannot compute pseudo-legal moves for index {i}, {j}"))
        }

        match self.board[i][j] {
            None => return Ok(Vec::new()),
            Some(piece) => match piece.piece_type {
                PieceType::Pawn => Ok(self.pawn_pseudo_legal_moves(i, j)),
                PieceType::Rook => Ok(self.directional_pseudo_legal_moves(i, j, &self.rook_move_directions, 8)),
                PieceType::Bishop =>  Ok(self.directional_pseudo_legal_moves(i, j, &self.bishop_move_directions, 8)),
                PieceType::Knight => Ok(self.directional_pseudo_legal_moves(i, j, &self.knight_move_directions, 1)),
                PieceType::Queen => Ok(self.directional_pseudo_legal_moves(i, j, &self.queen_move_directions, 8)),
                PieceType::King => Ok(self.directional_pseudo_legal_moves(i, j, &self.queen_move_directions, 1)),
            }
        }
    }

    //compute pseudo-legal moves for pieces that move in given directions
    //max_moves indicates how far a piece can "slide"
    //used for calculating pseudo-legal moves for every piece except for the pawn and king*
    fn directional_pseudo_legal_moves(&self, i : usize, j : usize, directions : &Vec<(i32, i32)>, max_moves : u32) -> Vec<(usize, usize)> {
        let piece_color = self.board[i][j].unwrap().color;

        let mut moves_vec : Vec<(usize, usize)> = Vec::new();

        //loop thorugh all directions the piece can move in
        for direction in directions {
            //create new mutable indicies, i32 to allow for negative values
            //movement directions may include negative values, so usize is not suitable
            let mut i_m = i as i32;
            let mut j_m = j as i32;

            let (d_i, d_j) = direction;
            let mut moves_made = 0;

            while moves_made < max_moves {

                i_m += d_i;
                j_m += d_j;

                if is_valid_pos(i_m, j_m) {
                    //convert to usize for indexing
                    let i_m = i_m as usize;
                    let j_m = j_m as usize;

                    //check if there is a piece at the given index i, j
                    // No piece -> add index to moves vec
                    // Piece of other color -> add piece to moves vec and break loop (go to next direction)
                    // Piece of same color -> break loop (go to next direction)
                    match self.board[i_m][j_m] {
                        None => moves_vec.push((i_m, j_m)),
                        Some(piece) => {
                            if piece.color == piece_color {
                                break;
                            } else {
                                moves_vec.push((i_m, j_m));
                                break;
                            }
                        }
                    }
                }

                moves_made += 1;
            }
        };

        
        return moves_vec;
    }

    fn pawn_pseudo_legal_moves(&self, i : usize, j : usize)-> Vec<(usize, usize)> {
        let pawn_color = self.board[i][j].unwrap().color;

        let d : i32 = match pawn_color {
            Color::White => -1,
            Color::Black => 1,
        };

        let mut moves_vec : Vec<(usize, usize)> = Vec::new();

        let i_indx = i as i32 + d;

        //check squares in front of the pawn
        if is_valid_pos(i_indx, j as i32){
            let i_indx = i_indx as usize;
            //square 1 in front
            if self.board[i_indx][j].is_none(){
                moves_vec.push((i_indx, j));

                //2 squares in front
                //only possible if pawn is on 2nd or 7th rank depending on color
                match pawn_color {
                    Color::White => {
                        if i == 6 && self.board[4][j].is_none(){
                            moves_vec.push((4, j));
                        }
                    },
        
                    Color::Black => {
                        if i == 1 && self.board[3][j].is_none(){
                            moves_vec.push((3, j));
                        }
                    },
                };
            }
        }

        //check squares that the pawn can capture
        if is_valid_pos(i_indx, (j + 1) as i32){
            let i_indx = i_indx as usize;
            if self.pawn_can_capture(i_indx, j + 1, pawn_color) {
                moves_vec.push((i_indx, j + 1))
            }
        }

        if is_valid_pos(i_indx, (j - 1) as i32){
            let i_indx = i_indx as usize;
            if self.pawn_can_capture(i_indx, j - 1, pawn_color) {
                moves_vec.push((i_indx, j - 1))
            }
        }

        return moves_vec;
    }

    fn pawn_can_capture(&self, i : usize, j : usize, pawn_color : Color) -> bool {
        //checks if en passant is allowed
        if self.en_passant_square.is_some(){
            if self.en_passant_square.unwrap() == (i, j){
                if can_en_passant(i).unwrap() == pawn_color{
                    return true;
                }
            }
        } else {
            //checks if pawn can move to given index
            match self.board[i][j] {
                None => (),
                Some(piece) => {
                    if piece.color != pawn_color {
                        return true;
                    }
                }
            }
        }

        return false;
    }
    
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
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
#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    White,
    Black,
}

fn is_valid_pos(i : i32, j : i32) -> bool {
    i >= 0 && i <= 7 && j >= 0 && j <= 7
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

fn alg_notation_to_indx(notation : &str) -> Result<(usize , usize), String> {
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

    // 8 - n since ranks in the array are mirrored, and the first rank is at index 7
    let row : usize = 8 - chr_vec[1].to_digit(10).unwrap() as usize;

    return Ok((row, col));
}

// returns which colored pawn is allowed to en passant on the given rank
// solves conflict where 2 pawns of opposite color can move to en passant square
fn can_en_passant(i : usize) -> Option<Color> {
    match i {
        2 => Some(Color::White),
        6 => Some(Color::Black),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let mut board = Game::new_starting_pos();

        // board.make_move("f7", "f3").unwrap();
        // // board.make_move("g1", "f3").unwrap();

        
        let board = Game::from("rq1r2k1/6pp/b1nNp3/p1B1Pp2/2P1pP2/8/1P2Q1PP/R4R1K w - f6 0 24").unwrap();

        let legal_moves =  board.get_pseudo_legal_moves(3, 4).unwrap();

        println!("{:?}", legal_moves);

        board.print_with_highlights(legal_moves);
    }

    // #[test]

    // fn play_game(moves_str : &str) {
    //     for (i, mv) in moves_str.split_whitespace().enumerate() {
    //         if !(i % 3 == 0) {
    //                                                                                      // from u/burntsushi on reddit
    //             let last_two_at = s.char_indices().rev().map(|(i, _)| i).nth(1).unwrap();// gets last 2 letters of the str
    //             let last_two = &s[last_two_at..];                            // 


    //         }
    //     }
    // }
}
