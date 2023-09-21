use std::fmt;
use std::collections::HashMap;
use std::hash::Hash;


///TODO
/// - Check which functions should be public
/// - Change name of functions for usability
/// - Documentation
/// - more testing

///DONE (may still include bugs)
/// - Create Game object from FEN string
/// - Create FEN string from Game object
/// - Piece movement
/// - alg notation to index converter and vice versa
/// - Update game variables after each move
/// - Legal move validation, all pieces + en passant + casteling
/// - Promotion
/// - Function to get all legal moves for a player
/// - Game state function : In progress, Checkmate, Stalemate, Insufficient material, fifty move rule
/// - Undo moves

#[derive(Clone)]
pub struct Game {
    board : [[Option<Piece>; 8] ; 8],
    turn : Color,
    kingside_castle : HashMap<Color, bool>,
    queenside_castle : HashMap<Color, bool>,
    en_passant_square : Option<(usize, usize)>,
    half_moves : u32,
    full_moves : u32,
    rook_move_directions : Vec<(i32, i32)>,
    bishop_move_directions : Vec<(i32, i32)>,
    queen_move_directions : Vec<(i32, i32)>,
    knight_move_directions : Vec<(i32, i32)>,
    previous_state : Option<Box<Game>>,
    white_attacked_squares : Vec<(usize, usize)>,
    black_attacked_squares : Vec<(usize, usize)>,
    insufficient_material : Vec<Vec<PieceType>>,
    captures : Vec<Piece>,
    promotion_piece : PieceType
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

        for row in self.board {
            let mut j = 0;
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

        let unwinnable_states = vec![
            vec![PieceType::King],
            vec![PieceType::King, PieceType::Knight],
            vec![PieceType::Knight, PieceType::King],
            vec![PieceType::King, PieceType::Bishop],
            vec![PieceType::Bishop, PieceType::King],
            vec![PieceType::King, PieceType::Knight, PieceType::Knight],
            vec![PieceType::Knight, PieceType::King, PieceType::Knight],
            vec![PieceType::Knight, PieceType::Knight, PieceType::King],
        ];

        Game {
            board : [[None ; 8] ; 8],
            turn : Color::White,
            kingside_castle : HashMap::from([
                (Color::White, true),
                (Color::Black, true),
            ]),
            queenside_castle : HashMap::from([
                (Color::White, true),
                (Color::Black, true),
            ]),
            en_passant_square : None,
            half_moves : 0,
            full_moves : 0,
            rook_move_directions,
            bishop_move_directions,
            queen_move_directions,
            knight_move_directions,
            previous_state : None,
            white_attacked_squares : Vec::new(),
            black_attacked_squares : Vec::new(),
            insufficient_material: unwinnable_states,
            captures : Vec::new(),
            promotion_piece : PieceType::Queen
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
                'K' => {board.kingside_castle.insert(Color::White, true); },
                'Q' => {board.queenside_castle.insert(Color::White, true); },
                'k' => {board.kingside_castle.insert(Color::Black, true); },
                'q' => {board.queenside_castle.insert(Color::Black, true); },
                '-' => {
                    board.kingside_castle.insert(Color::White, false);
                    board.queenside_castle.insert(Color::White, false); 
                    board.kingside_castle.insert(Color::Black, false);
                    board.queenside_castle.insert(Color::Black, false);
                },
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

        board.update_attacked_squares();
        // board.update_state();

        return Result::Ok(board);
    }

    pub fn to_fen(&self) -> String {
        let mut fen_str = String::new();

        let mut empty_squares = 0;

        //field 1 - piece placement
        for i in 0..8 {
            for j in 0..8 {
                if self.board[i][j].is_some() {
                    if empty_squares != 0 {
                        fen_str.push(char::from_digit(empty_squares, 10).unwrap());
                    }

                    fen_str.push(get_piece_notation(self.board[i][j].unwrap()));

                    empty_squares = 0;
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares != 0 {
                fen_str.push(char::from_digit(empty_squares, 10).unwrap());
            }
            if i != 7{fen_str.push('/');}
            empty_squares = 0;
        }

        fen_str.push(' ');

        //field 2 - turn

        match self.turn {
            Color::White => fen_str.push('w'),
            Color::Black => fen_str.push('b'),
        }

        fen_str.push(' ');

        //field 3 - casteling
        let mut add_dash = true;

        if *self.kingside_castle.get(&Color::White).unwrap() {
            fen_str.push('K');
            add_dash = false;
        }
        if *self.queenside_castle.get(&Color::White).unwrap() {
            fen_str.push('Q');
            add_dash = false;
        }
        if *self.kingside_castle.get(&Color::Black).unwrap() {
            fen_str.push('k');
            add_dash = false;
        }
        if *self.queenside_castle.get(&Color::Black).unwrap() {
            fen_str.push('q');
            add_dash = false;
        }

        if add_dash {
            fen_str.push('-');
        }

        //field 4 - en passant
        fen_str.push(' ');

        let en_passant_square = match self.en_passant_square {
            Some(square) => indx_to_alg_notation(square).unwrap(),
            None => String::from("-"),
        };

        fen_str.push_str(&en_passant_square);

        fen_str.push(' ');

        //field 5 - halfmoves
        fen_str.push_str(&self.half_moves.to_string());

        fen_str.push(' ');

        //field 6 - fullmoves
        fen_str.push_str(&self.full_moves.to_string());

        return fen_str;
    }

    /// Move piece on the board. "from" and "to" are in algebraic notation, with turn number and piece type omitted
    /// eg from = e2, to = e4
    /// For algebraic notation see https://www.chess.com/terms/chess-notation#readalgebraic
    /// Returns Ok(true) if move is valid, Ok(false) if invalid
    pub fn make_move(&mut self, from : &str, to : &str) -> Result<bool, String> {
        let from = alg_notation_to_indx(from)?;
        let to = alg_notation_to_indx(to)?;

        self.make_move_with_index(from, to, true)
    }

    /// Same functionality as make_move, but uses array indicies instead of algebraic notation
    pub fn make_move_array_index(&mut self, from : (usize, usize), to : (usize, usize)) -> Result<bool, String> {
        if is_valid_move(from, to){
            return self.make_move_with_index(from, to, true);
        }

        Ok(false)
    }

    fn make_move_with_index(&mut self, from : (usize, usize), to : (usize, usize), check_legal : bool) -> Result<bool, String> {
        let (i1, j1) = from;
        let (i2, j2) = to;

        //abort if move is illegal
        //ignored if check_legal is false
        if check_legal{
            if !(self.get_legal_moves_square(i1, j1).contains(&(i2, j2))) {
                return Ok(false);
            }
        }

        //save board state
        self.previous_state = Some(Box::new(self.clone()));

        //increment half moves, if there is a capture or pawn move this will be reset
        self.half_moves += 1;
        self.en_passant_square = None;

        //Capture logic
        if self.board[i2][j2].is_some() {
            self.captures.push(self.board[i2][j2].unwrap());
            self.half_moves = 0; //piece captured : resets half moves
        }

        //Check if casteling
        if self.board[i1][j1].unwrap().piece_type == PieceType::King {
            let d = j1 as i32 - j2 as i32;

            //check if king is moved 2 squares
            if d.abs() == 2 {
                //remove casteling rights
                let king_color = self.board[i1][j1].unwrap().color;
                self.kingside_castle.insert(king_color, false);
                self.queenside_castle.insert(king_color, false);

                //kingside castle
                if d < 0 {
                    self.board[i1][5] = self.board[i1][7];
                    self.board[i1][7] = None;
                } else { //queenside castle
                    self.board[i1][3] = self.board[i1][0];
                    self.board[i1][0] = None;
                }
            }
        } else if self.board[i1][j1].unwrap().piece_type == PieceType::Rook {
            //remove casteling rights if the rook is moved

            let rook_color = self.board[i1][j1].unwrap().color;

            let starting_rank = match rook_color {
                Color::White => 7,
                Color::Black => 0,
            };

            if i1 == starting_rank {
                match j1 {
                    0 => {self.queenside_castle.insert(rook_color, false);},
                    7 => {self.kingside_castle.insert(rook_color, false);},
                    _ => (),
                }
            }
        } else if self.board[i1][j1].unwrap().piece_type == PieceType::Pawn {
            let pawn_color = self.board[i1][j1].unwrap().color;
            self.half_moves = 0; //pawn moved : reset half moves

            //check if pawn is moved two squares
            let d = i1 as i32 - i2 as i32;

            if d.abs() == 2 {
                self.en_passant_square = Some(((i1 + i2) / 2, j1))
            }

            if self.is_promotion_move(from, to) {
                self.board[i1][j1] = Some(Piece::new(self.promotion_piece, pawn_color));
            }
        }

        //make move
        self.board[i2][j2] = self.board[i1][j1];
        self.board[i1][j1] = None;

        self.update_attacked_squares();

        if self.turn == Color::Black {
            self.full_moves += 1;
        }

        self.turn = self.turn.opposite();

        Ok(true)
    }

    /// Checks wether or not a move is a promotion move
    pub fn is_promotion_move(&self, from : (usize, usize), to : (usize, usize)) -> bool {
        if is_valid_move(from, to){
            let pawn_color = self.board[from.0][from.1].unwrap().color;
        
            let promotion_rank = match pawn_color {
                Color::White => 0,
                Color::Black => 7,
            };

            return to.1 == promotion_rank;
        }

        return false;
    }

    pub fn undo_last_move(&mut self){
        if self.previous_state.is_none() {return;}

        let mut binding = self.previous_state.clone().unwrap();
        let prev = binding.as_mut();
        self.board = prev.board;
        self.kingside_castle = prev.kingside_castle.clone();
        self.queenside_castle = prev.queenside_castle.clone();
        self.en_passant_square = prev.en_passant_square;
        self.half_moves = prev.half_moves;
        self.full_moves = prev.full_moves;
        self.previous_state = prev.previous_state.clone();
        self.turn = prev.turn;
    }

    fn get_pseudo_legal_moves_for_square(&self, i : usize, j : usize, only_attacked : bool) -> Result<Vec<(usize, usize)>, String>{
        if !is_valid_pos(i as i32, j as i32) {
            return Err(format!("Invalid index : Cannot compute pseudo-legal moves for index {i}, {j}"))
        }

        match self.board[i][j] {
            None => return Ok(Vec::new()),
            Some(piece) => match piece.piece_type {
                PieceType::Pawn => Ok(self.pawn_pseudo_legal_moves(i, j, only_attacked)),
                PieceType::Rook => Ok(self.directional_pseudo_legal_moves(i, j, &self.rook_move_directions, 8, only_attacked)),
                PieceType::Bishop =>  Ok(self.directional_pseudo_legal_moves(i, j, &self.bishop_move_directions, 8, only_attacked)),
                PieceType::Knight => Ok(self.directional_pseudo_legal_moves(i, j, &self.knight_move_directions, 1, only_attacked)),
                PieceType::Queen => Ok(self.directional_pseudo_legal_moves(i, j, &self.queen_move_directions, 8, only_attacked)),
                PieceType::King => Ok(self.king_pseudo_legal_moves(i, j, only_attacked)),
            }
        }
    }

    //compute pseudo-legal moves for pieces that move in given directions
    //max_moves indicates how far a piece can "slide"
    //used for calculating pseudo-legal moves for every piece except for the pawn and king*
    // *the king has it's own function to include casteling, but uses this function as well
    fn directional_pseudo_legal_moves(&self, i : usize, j : usize, directions : &Vec<(i32, i32)>, max_moves : u32, include_all_attacked : bool) -> Vec<(usize, usize)> {
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
                                if include_all_attacked {
                                    moves_vec.push((i_m, j_m));
                                }
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

    fn pawn_pseudo_legal_moves(&self, i : usize, j : usize, only_attacked : bool)-> Vec<(usize, usize)> {
        let pawn_color = self.board[i][j].unwrap().color;

        let d : i32 = match pawn_color {
            Color::White => -1,
            Color::Black => 1,
        };

        let mut moves_vec : Vec<(usize, usize)> = Vec::new();

        let i_indx = i as i32 + d;

        if !only_attacked {
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
    
            if is_valid_pos(i_indx, j as i32 - 1){
                let i_indx = i_indx as usize;
                if self.pawn_can_capture(i_indx, j - 1, pawn_color) {
                    moves_vec.push((i_indx, j - 1))
                }
            }
        } else {
            //check squares that the pawn can capture
            if is_valid_pos(i_indx, (j + 1) as i32){
                let i_indx = i_indx as usize;
                moves_vec.push((i_indx, j + 1))
            }
    
            if is_valid_pos(i_indx, j as i32 - 1){
                let i_indx = i_indx as usize;
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

    fn king_pseudo_legal_moves(&self, i : usize, j : usize, include_all_attacked : bool) -> Vec<(usize, usize)> {
        let king_color = self.board[i][j].unwrap().color;
        let mut move_vec = self.directional_pseudo_legal_moves(i, j, &self.queen_move_directions, 1, include_all_attacked);

        let kingside = self.kingside_castle.get(&king_color).unwrap();
        let queenside = self.queenside_castle.get(&king_color).unwrap();

        //Casteling logic
        
        if *kingside {
            //checks if squares between king and rook are empty, and are not attacked
            if self.board[i][j + 1].is_none() && self.board[i][j + 2].is_none() {
                let attacked_squres = self.get_attacked_squares(king_color.opposite());

                if !attacked_squres.contains(&(i, j)) && !attacked_squres.contains(&(i, j + 1)) && !attacked_squres.contains(&(i, j + 2))
                {
                     move_vec.push((i, j + 2));
                }
            }
        } 

        if *queenside {
            //checks if squares between king and rook are empty, and are not attacked
            if self.board[i][j - 1].is_none() && self.board[i][j - 2].is_none() {
                let attacked_squres = self.get_attacked_squares(king_color.opposite());

                if !attacked_squres.contains(&(i, j)) && !attacked_squres.contains(&(i, j - 1)) && !attacked_squres.contains(&(i, j - 2))
                {
                     move_vec.push((i, j - 2));
                }
            }
        }

        return move_vec;
    }
    
    //get all attacked squares
    //does not specify which piece is attacking what
    fn get_attacked_squares(&self, color : Color) -> &Vec<(usize, usize)> {
        match color {
            Color::White => &(self.white_attacked_squares),
            Color::Black => &(self.black_attacked_squares),
        }
    }

    fn update_attacked_squares(&mut self) {
        let mut white_attack_vec : Vec<(usize, usize)> = Vec::new();
        let mut black_attack_vec : Vec<(usize, usize)> = Vec::new();

        for i in 0..8 {
            for j in 0..8 {
                if self.board[i][j].is_some() {
                    match self.board[i][j].unwrap().color{
                        Color::White => white_attack_vec.append(&mut self.get_pseudo_legal_moves_for_square(i, j, true).unwrap()),
                        Color::Black => black_attack_vec.append(&mut self.get_pseudo_legal_moves_for_square(i, j, true).unwrap()),
                    }
                }
            }
        }

        self.white_attacked_squares = white_attack_vec;
        self.black_attacked_squares = black_attack_vec;
    }

    //get legal moves for a given square
    pub fn get_legal_moves_square(&mut self, i : usize, j : usize) -> Vec<(usize, usize)>{
        let color = self.board[i][j].unwrap().color;

        let pos = (i, j);
        let pseudo_legal_moves = self.get_pseudo_legal_moves_for_square(i, j, false).unwrap();
        let mut legal_moves = Vec::new();

        for mve in pseudo_legal_moves {
            self.make_move_with_index(pos, mve, false).unwrap();

            if !self.in_check(color) {
                legal_moves.push(mve);
            }
            
            self.undo_last_move();
        }

        return legal_moves;
    }

    //returns vector of tuples
    //Each tuple is structured as (from, to), where "from" is the index of the piece that moves
    //And "to" is a vector of the legal moves for that piece
    pub fn get_all_legal_moves(&mut self, color : Color) -> HashMap<(usize, usize), Vec<(usize, usize)>> {
        let mut move_hash : HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

        for i in 0..8 {
            for j in 0..8 {
                if self.board[i][j].is_some() {
                    if self.board[i][j].unwrap().color == color {
                        let legal_moves = self.get_legal_moves_square(i, j);

                        move_hash.insert((i, j), legal_moves);
                    }
                }
            }
        }

        return move_hash;
    }

    /// Returns wether a player is in check or not
    pub fn in_check(&self, color : Color) -> bool {
        let attacked_squares = self.get_attacked_squares(color.opposite());

        //Find king position
        for i in 0..8 {
            for j in 0..8 {
                if self.board[i][j].is_some(){
                    if self.board[i][j].unwrap().piece_type == PieceType::King
                    && self.board[i][j].unwrap().color == color
                    {
                        return attacked_squares.contains(&(i, j));
                    }
                }
            }
        }

        return false;
    }

    fn num_of_legal_moves(&mut self, color : Color) -> u32 {
        let mut res = 0;

        for moves in self.get_all_legal_moves(color).values(){
            res += moves.len();
        }

        return res as u32;
    }

    //checks if "color" has enough pieces to win
    fn can_win(&self, color : Color) -> bool {

        let mut pieces = Vec::new();

        for i in 0..8 {
            for j in 0..8 {
                if self.board[i][j].is_some(){
                    if self.board[i][j].unwrap().color == color {
                        pieces.push(self.board[i][j].unwrap().piece_type);
                    }
                }
            }
        }

        return !self.insufficient_material.contains(&pieces);
    }

    pub fn get_state(&mut self) -> GameState{
        let current_turn_legal_moves = match self.turn {
            Color::White => self.num_of_legal_moves(Color::White),
            Color::Black => self.num_of_legal_moves(Color::Black),
        };

        if current_turn_legal_moves == 0 {
            if self.in_check(self.turn) {
                return GameState::Win(WinState::Checkmate(self.turn.opposite()));
            } else {
                return GameState::Draw(DrawState::Stalemate);
            }
        }

        if self.half_moves >= 100 {
            return GameState::Draw(DrawState::FiftyMoveRule);
        }

        if !self.can_win(Color::White) && !self.can_win(Color::Black) {
            return GameState::Draw(DrawState::InsufficientMaterial);
        }

        return GameState::InProgress;
    }

    /// Set which type of piece a pawn should be promoted to
    /// 
    /// The promotion piece type is set to Piecetype::Queen by default
    pub fn set_promotion_type(&mut self, piece_type : PieceType) {
        self.promotion_piece = piece_type;
    }
}

#[derive(Debug, Clone)]
pub enum GameState {
    InProgress,
    Win(WinState),
    Draw(DrawState),
}
#[derive(Debug, Clone)]
pub enum DrawState {
    Stalemate,
    InsufficientMaterial,
    FiftyMoveRule
}
#[derive(Debug, Clone)]

//Color represents winner
pub enum WinState {
    Checkmate(Color)
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    fn opposite(&self) -> Color {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

fn is_valid_pos(i : i32, j : i32) -> bool {
    i >= 0 && i <= 7 && j >= 0 && j <= 7
}

fn is_valid_move(from : (usize, usize), to : (usize, usize)) -> bool {
    let (i1, j1) = from;
    let (i2, j2) = to;
    
    is_valid_pos(i1 as i32, j1 as i32) && is_valid_pos(i2 as i32, j2 as i32)
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

fn get_piece_notation(piece : Piece) -> char {
    let mut letter = match piece.piece_type {
        PieceType::Pawn=> 'P',
        PieceType::Knight => 'N',
        PieceType::Bishop => 'B',
        PieceType::Rook => 'R',
        PieceType::Queen => 'Q',
        PieceType::King => 'K',
    };

    if piece.color == Color::Black {
        letter = letter.to_ascii_lowercase();
    }

    return letter;
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

pub fn alg_notation_to_indx(notation : &str) -> Result<(usize , usize), String> {
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

pub fn indx_to_alg_notation(indx : (usize, usize)) -> Result<String, String> {
    let rank : char = match indx.1 {
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _c => return Err(format!("Invalid column {}", _c)),
    };

    // 8 - n since ranks in the array are mirrored, and the first rank is at index 7
    let col = match char::from_digit(8 - indx.0 as u32, 10) {
        Some(c) => c,
        _ => return Err(format!("Invalid row {}", indx.0)),
    };

    let mut alg_notation = String::new();

    alg_notation.push(rank);
    alg_notation.push(col);

    return Ok(alg_notation);
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
        let mut board = Game::from("4k3/1P6/4K3/8/8/8/8/8 w - - 0 1").unwrap();

        println!("{:?}", board);

        println!("{:?}", board);
        println!("{:?}", board.get_state());
    }

    
    #[test]

    //Make a board from FEN string
    //Also displays functionality of get_state()
    fn fen_to_board_test(){
        let mut board = Game::from("k1Q2b2/pp6/1qp2p2/3P3p/2p2B1P/2P5/PP4r1/1K1R4 b - - 1 34").unwrap();

        println!("{:?}", board);
        println!("{:?}", board.get_state());
    }

    #[test]

    //Prints FEN string of starting position
    fn board_to_fen_test(){
        let board = Game::new_starting_pos();

        println!("{}", board.to_fen());
    }

    #[test]

    //Magnus Carlsen (white) vs Eric Rosen (black) - Titled Tuesday Blitz May 16
    //https://www.chess.com/games/view/16402405 
    fn real_game_test() {
        let mut board = Game::new_starting_pos();

        let moves = vec![
            ("f2", "f4"),
            ("d7", "d5"),
            ("g1", "f3"),
            ("g7", "g6"),
            ("d2", "d3"),
            ("f8", "g7"),
            ("e2", "e4"),
            ("c7", "c6"),
            ("e4", "e5"),
            ("g8", "h6"),
            ("d3", "d4"),
            ("c8", "g4"),
            ("h2", "h3"),
            ("g4", "f3"),
            ("d1", "f3"),
            ("h6", "f5"),
            ("c2", "c3"),
            ("e7", "e6"),
            ("g2", "g4"),
            ("f5", "h4"),
            ("f3", "f2"),
            ("h7", "h5"),
            ("c1", "e3"),
            ("b8", "d7"),
            ("b1", "d2"),
            ("g7", "f8"),
            ("e1", "c1"),
            ("f8", "e7"),
            ("f1", "d3"),
            ("d8", "a5"),
            ("c1", "b1"),
            ("e8", "c8"),
            ("f4", "f5"),
            ("g6", "f5"),
            ("g4", "f5"),
            ("e6", "f5"),
            ("d3", "f5"),
            ("h4", "f5"),
            ("f2", "f5"),
            ("d8", "f8"),
            ("h1", "g1"),
            ("a5", "d8"),
            ("g1", "g7"),
            ("f7", "f6"),
            ("e5", "e6"),
            ("d7", "b6"),
            ("e3", "f4"),
            ("f8", "g8"),
            ("g7", "f7"),
            ("g8", "g2"),
            ("h3", "h4"),
            ("b6", "c4"),
            ("d2", "c4"),
            ("d5", "c4"),
            ("d4", "d5"),
            ("d8", "b6"),
            ("f4", "c1"),
            ("e7", "a3"),
            ("f7", "f8"),
            ("h8", "f8"),
            ("e6", "e7"),
            ("c8", "b8"),
            ("c1", "f4"),
            ("b8", "a8"),
            ("e7", "f8"),
            ("a3", "f8"),
            ("f5", "c8"),
        ];

        for (from, to) in moves {
            board.make_move(from, to).unwrap();
        }
        
        println!("{:?}", board);
        println!("{:?}", board.get_state());
    }
}
