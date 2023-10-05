use std::fmt;
use std::collections::HashMap;
use std::hash::Hash;

/// Main Game struct for chess board representation. 
/// Used to create a position, and play moves. Includes
/// move validation, reading game-state, getting captured
/// pieces and undoing moves.
/// 
/// Supports reading data from the board using either algebraic notation
/// or array indicies. Array indices `i` and `j` represent rank and file respectively,
/// and `0`, `0` representing a8 in algebraic notation. 
/// 
/// # Creation
/// * Always use one of the two provided constructors, `new_starting_pos()` 
/// or `from_fen()`. 
/// 
/// # Examples
/// 
/// * Using new_starting_pos() to create an new game
/// ```ignore
/// let game = Game::new_starting_pos();
/// 
/// println!("{:?}", game);
/// ``` 
/// * Using from_fen() to create a new game, note that this
/// is the FEN representation for the standard starting position,
/// so this will be identical to calling new_starting_pos()
/// ```ignore
/// let game = Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
/// 
/// println!("{:?}", game);
/// ``` 
/// 
/// # Errors
/// 
/// * Deviating from using the provided constructors may cause
/// a panic!
/// 
/// # Notes
/// * For algebraic notation, refer to: https://www.chess.com/terms/chess-notation#readalgebraic
/// * fmt::Debug is implemented for Game. By using debug print syntax
/// this will print a visual representation the board to the terminal
#[derive(Clone, PartialEq)]
pub struct Game {
    //2d array for board representation, each piece is represented by an Option.
    //Empty square is represented by Option::None
    board : [[Option<Piece>; 8] ; 8],
    //turn indicator
    turn : Color,
    //kingside castling rights for both players
    kingside_castle : HashMap<Color, bool>,
    //queenside castling rights for both players
    queenside_castle : HashMap<Color, bool>,
    //index of possible en passant square
    en_passant_square : Option<(usize, usize)>,
    //number of half moves for current position
    half_moves : u32,
    //number of full moves made in the game
    full_moves : u32,
    //vectors with move directions for the different pieces
    //used for generating pseudo-legal moves
    rook_move_directions : Vec<(i32, i32)>,
    bishop_move_directions : Vec<(i32, i32)>,
    queen_move_directions : Vec<(i32, i32)>,
    knight_move_directions : Vec<(i32, i32)>,
    //previous game state
    previous_state : Option<Box<Game>>,
    //squares under attack by respective player
    white_attacked_squares : Vec<(usize, usize)>,
    black_attacked_squares : Vec<(usize, usize)>,
    //states that result in draw by insufficient material
    insufficient_material : Vec<Vec<PieceType>>,
    //vector of captured pieces
    captures : Vec<Piece>,
    //possible square where pawn be promoted in current position
    promotion_square : Option<(usize, usize)>
}

//implements debug for game, using debug print will
//print visual board representation to screen
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
    //creates empty board
    //helper function for from_fen() constructor
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
            promotion_square : None,
        }
    }
    /// Create a new board with the standard starting position.
    /// 
    /// # Examples
    /// ```ignore
    /// let mut game = Game::new_starting_pos();
    /// 
    /// game.make_move("e2", "e4", true).unwrap();
    /// ```
    /// 
    /// # Notes
    /// * safe unwrap() call since fen_str is hard-coded
    pub fn new_starting_pos() -> Game {
        Game::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    /// Parses a Forsyth-Edwards Notation (FEN) string and constructs a chess Game representation.
    ///
    /// FEN is a standard notation used to describe the state of a chess game. The FEN string consists
    /// of six fields, separated by spaces, representing the following information (in order):
    ///
    /// 1. Piece placement on the board.
    /// 2. Active color ('w' for White, 'b' for Black).
    /// 3. Castling availability.
    /// 4. En passant target square.
    /// 5. Half-move clock (for the 50-move rule).
    /// 6. Full-move number (increments after each full move by Black).
    ///
    /// # Arguments
    ///
    /// * `fen_str` - A string containing the FEN representation of the chess game state.
    ///
    /// # Returns
    ///
    /// * `Result<Game, String>` - A `Result` where `Ok` contains a `Game` instance representing the
    ///   parsed chess position, and `Err` contains an error message if parsing fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use my_chess_lib::Game;
    ///
    /// let fen_string = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    ///
    /// match Game::from(fen_string) {
    ///     Ok(game) => {
    ///         // Successfully parsed FEN string into a Game instance.
    ///         // You can now work with the chess position.
    ///     },
    ///     Err(error) => {
    ///         eprintln!("Error parsing FEN: {}", error);
    ///     },
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function returns an `Err` variant with an error message if any of the FEN fields contain
    /// invalid or unexpected values.
    ///
    /// # Notes
    ///
    /// - The FEN string should adhere to the standard format for accurate parsing.
    /// - For details on FEN notation, refer to: https://en.wikipedia.org/wiki/Forsyth–Edwards_Notation
    pub fn from_fen(fen_str : &str) -> Result<Game, String> {
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

        // Map castling rights string to Board
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
                _c => return Err(format!("Invalid castling field {}", _c)),
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

    /// Generates a Forsyth-Edwards Notation (FEN) string from the current state of the chess game.
    ///
    /// FEN is a standard notation used to describe the state of a chess game. The FEN string consists
    /// of six fields, separated by spaces, representing the following information (in order):
    ///
    /// 1. Piece placement on the board.
    /// 2. Active color ('w' for White, 'b' for Black).
    /// 3. Castling availability.
    /// 4. En passant target square.
    /// 5. Half-move clock (for the 50-move rule).
    /// 6. Full-move number (increments after each full move by Black).
    ///
    /// # Returns
    ///
    /// * `String` - A string containing the FEN representation of the current chess game state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use my_chess_lib::Game;
    ///
    /// let mut game = Game::new_starting_position();
    /// game.make_move("e2", "e4", true).unwrap();
    ///
    /// let fen_string = game.to_fen();
    /// println!("FEN: {}", fen_string);
    /// ```
    ///
    /// # Notes
    ///
    /// - The generated FEN string adheres to the standard format for accurate representation.
    /// - For details on FEN notation, refer to: https://en.wikipedia.org/wiki/Forsyth–Edwards_Notation
    pub fn to_fen(&self) -> String {
        let mut fen_str = String::new();

        let mut empty_squares = 0;

        //field 1 - piece placement
        for i in 0..8 {
            for j in 0..8 {
                if let Some(piece) = self.board[i][j] {
                    if empty_squares != 0 {
                        fen_str.push(char::from_digit(empty_squares, 10).unwrap());
                    }

                    fen_str.push(get_piece_notation(piece));

                    empty_squares = 0;
                } else {
                    empty_squares += 1;
                }
            }
            if empty_squares != 0 {
                //empty_squares always in range 0-8, so from_digit will always return Some(char)
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

        //field 3 - castling
        let mut add_dash = true;

        //hardcoded get() call, unwrap will always be safe
        //given that castling fields are configured correctly
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

        //en passant square always valid index, so unwarp on indx_to_alg_notation() is safe
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

    /// Get piece at given indexed position.
    /// 
    /// # Arguments
    /// * `indx` is a tuple of type `(usize, usize)`, where the first
    /// element indexes the rank and second element the file.
    /// Note that array indicies start at 0 in contrast to algebraic notation.
    /// 
    /// # Returns
    /// * `Result<Option<Piece>, String>`, `Option<Piece>` being the corresponding
    /// value on the board (See `Game` struct for board representation).
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// let game = Game::new_starting_pos();
    /// 
    /// let piece = game.piece_at_array_index((0,0));
    /// let top_left_piece = Piece::new(PieceType::Rook, Color::Black);
    /// 
    /// assert_eq!(piece, Ok(Some(top_left_piece)));
    /// ```
    /// 
    /// # Errors
    /// * Returns `Err(String)` if input is invalid index
    pub fn piece_at_array_index(&self, indx : (usize, usize)) -> Result<Option<Piece>, String> {
        let (i, j) = indx;

        if !is_valid_pos(i as i32, j as i32) {
            return Err(format!("Invalid index {:?}", indx));
        } else {
            return Ok(self.board[i][j]);
        }
    }
    /// Get piece at given position using algebraic notation.
    /// 
    /// # Arguments
    /// * `notation` is a `&str` written in algebraic notation.
    /// 
    /// # Returns
    /// * `Result<Option<Piece>, String>`, `Option<Piece>` being the corresponding
    /// value on the board (See `Game` struct for board representation).
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// let game = Game::new_starting_pos();
    /// 
    /// let piece = game.piece_at_array_index("e2");
    /// let e_pawn = Piece::new(PieceType::Pawn, Color::White);
    /// 
    /// assert_eq!(piece, Ok(Some(e_pawn)));
    /// ```
    /// 
    /// # Errors
    /// * Returns `Err(String)` if input is invalid position
    pub fn piece_at_alg_notation(&self, notation : &str) -> Result<Option<Piece>, String> {
        let (i, j) = alg_notation_to_indx(notation)?;

        if !is_valid_pos(i as i32, j as i32) {
            return Err(format!("Invalid notation {:?}", notation));
        } else {
            return Ok(self.board[i][j]);
        }
    }
    /// Make a move on the board using algebraic notation.
    ///  
    /// 
    /// # Arguments
    /// * `from` and `to` are both in algebraic notation.
    /// * `from` refers to the square the piece you want to move is currently on, 
    /// `to` is the square you want to move it to.
    /// * `auto_promote` is a `bool` indicating wether or not a pawn, once it has 
    /// reached the end rank, should be promoted automatically. If `true`, it will
    /// be promoted to a queen automatically, if `false` it will remain a pawn and
    /// the game state will be `GameState::AwaitPromotion`. The user is then excpected
    /// to handle promotion explicitly using the `promote_to_piece` method.
    ///
    /// # Returns
    /// * `Result<bool, String>` - A `Result` where `Ok` contains a `bool` representing
    /// wether or not the move is valid, and `Err` contains an error message if the move fails, or
    /// if one or both of the provided indicies is invalid.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use my_chess_lib::Game;
    /// 
    /// let mut game = Game::new_starting_position();
    /// let move1 = game.make_move("e2", "e4"); //Moves pawn to e4
    /// let move2 = game.make_move("f2", "f5"); //Invalid move
    /// let move3 = game.make_move("", ""); //Invalid format
    /// 
    /// assert_eq!(move1, Ok(true));
    /// assert_eq!(move2, Ok(false)));
    /// assert_eq!(move3, Err("Invalid notation "));
    /// 
    /// ```
    /// 
    /// # Notes
    /// * For algebraic notation, refer to: https://www.chess.com/terms/chess-notation#readalgebraic
    pub fn make_move(&mut self, from : &str, to : &str, auto_promote : bool) -> Result<bool, String> {
        let from = alg_notation_to_indx(from)?;
        let to = alg_notation_to_indx(to)?;

        self.make_move_with_index(from, to, true, auto_promote)
    }

    /// Make a move on the board using array indicies.
    /// 
    /// # Arguments
    /// * `from` and `to` are both in algebraic notation.
    /// * `from` refers to the square the piece you want to move is currently on, 
    /// `to` is the square you want to move it to.
    /// * `auto_promote` is a `bool` indicating wether or not a pawn, once it has 
    /// reached the end rank, should be promoted automatically. If `true`, it will
    /// be promoted to a queen automatically, if `false` it will remain a pawn and
    /// the game state will be `GameState::AwaitPromotion`. The user is then excpected
    /// to handle promotion explicitly using the `promote_to_piece` method.
    ///
    /// # Returns
    /// * `Result<bool, String>` - A `Result` where `Ok` contains a `bool` representing
    /// wether or not the move is valid, and `Err` contains an error message if the move fails, or
    /// if one or both of the provided indicies is invalid.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// use my_chess_lib::Game;
    /// 
    /// let mut game = Game::new_starting_position();
    /// let move1 = game.make_move("e2", "e4"); //Moves pawn to e4
    /// let move2 = game.make_move("f2", "f5"); //Invalid move
    /// let move3 = game.make_move("", ""); //Invalid format
    /// 
    /// assert_eq!(move1, Ok(true));
    /// assert_eq!(move2, Ok(false)));
    /// assert_eq!(move3, Err("Invalid notation "));
    /// 
    /// ```
    /// 
    /// # Notes
    /// * board array indicies start at 0 as opposed to algebraic notation
    pub fn make_move_array_index(&mut self, from : (usize, usize), to : (usize, usize), auto_promote : bool) -> Result<bool, String> {
        if is_valid_move(from, to){
            return self.make_move_with_index(from, to, true, auto_promote);
        }

        Ok(false)
    }

    /// Used to promote a pawn at the final rank. This method is
    /// used to promote when using `make_move(auto_promote=false)`. Note
    /// that this method must be called _after_ calling `make_move`.
    /// 
    /// # Arguments
    /// * `piece_type` is of type `PieceType` and represents the type
    /// of piece the pawn will be promoted to.
    /// 
    /// # Returns
    /// * `bool` representing wether or not promotion was successful
    /// 
    /// # Examples
    /// * How a game loop might look
    /// ```ignore
    /// let mut game = Game::new_starting_pos();
    /// 
    /// loop {
    ///     //Get user input...
    ///     
    ///     //`from` and `to` are placeholders
    ///     game.make_move(from, to, false); //auto_promote set to false
    /// 
    ///     match game.get_state() {
    ///         GameState::AwaitPromotion => {
    ///             //Prompt user to choose piece
    /// 
    ///             //`picked_piece` is a placeholder
    ///             game.promote_to_piece(picked_piece);
    ///         }
    ///         //Handle other cases...
    ///     }
    ///}
    /// ```
    /// 
    /// # Notes
    /// * No logic preventing promoting a pawn to a pawn, 
    /// however it is not possible to promote it more than once.
    pub fn promote_to_piece(&mut self, piece_type : PieceType) -> bool {
        let res = match self.promotion_square {
            Some(indx) => {self.promote(indx, piece_type); true}
            None => false,
        };

        self.promotion_square = None;

        res
    }

    //helper function for promote_to_piece
    //places the chosen piece on the board with appropriate color
    fn promote(&mut self, indx : (usize, usize), piece_type : PieceType) {
        let (i, j) = indx;

        let piece_color = self.board[i][j].unwrap().color;

        self.board[i][j] = Some(Piece::new(piece_type, piece_color));
    }

    /// Undo the last move that was made. Reverts pieces
    /// as well as board state. Multiple calls can be chained together
    /// to undo multiple moves.
    /// 
    /// # Examples
    /// * Note that `undo_lat_move()` does not revert to previous Game object, 
    /// rather only reverts effected fields. This means the game will not
    /// be equivalent to previous game after undoing
    /// 
    /// ```ignore
    /// let mut game = Game::new_starting_pos();
    ///
    /// let previous_game = game.clone();
    ///
    /// game.make_move("e2", "e4", true).unwrap();
    ///
    /// game.undo_last_move();
    /// 
    /// //not equal!
    /// !assert_eq!(previous_game, game);
    /// ```
    pub fn undo_last_move(&mut self){
        if self.previous_state.is_none() {return;}

        //function returns if previous_state is None, so unwrap is safe
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
        self.captures = prev.captures.clone();
    }

    /// Get a `Vec` of legal moves for a given square. The vector consist 
    /// of tuples `(usize, usize)` descibing the indicies in the 2d board array.
    /// 
    /// # Arguments
    /// * `to` is algebraic notation for a square
    /// 
    /// # Returns
    /// 
    /// * Returns `Vec` of tuples `(usize, usize)` describing all array indicies
    /// that the piece at the provided index can move to.
    /// * Returns Result with empty vector if the board position is empty.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// let mut game = Game::new_starting_pos();
    /// 
    /// //print legal moves for knight on b1
    /// println!("{:?}", game.get_legal_moves_alg_notation("b1").ok().unwrap());
    /// ```
    /// This will print `[(5, 2), (5, 0)]` corresponding to c3 and a3. 
    /// 
    /// # Errors
    /// 
    /// * If the provided index is invalid the function returns Err(String)
    pub fn get_legal_moves_alg_notation(&mut self, pos : &str) -> Result<Vec<(usize, usize)>, String>{
        let indx = alg_notation_to_indx(pos)?;

        self.get_legal_moves_array_index(indx)
    }

    /// Get a `Vec` of legal moves for a given square. The vector consist 
    /// of tuples `(usize, usize)` descibing the indicies in the 2d board array.
    /// 
    /// # Arguments
    /// * array index in the board, for more detail refer to `Game` struct.
    /// 
    /// # Returns
    /// 
    /// * Returns `Vec` of tuples `(usize, usize)` describing all array indicies
    /// that the piece at the provided index can move to.
    /// * Returns Result with empty vector if the board position is empty.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// let mut game = Game::new_starting_pos();
    /// 
    /// //print legal moves for knight on 7, 1 (b1)
    /// println!("{:?}", game.get_legal_moves_array_index((7, 1)).ok().unwrap());
    /// ```
    /// This will print `[(5, 2), (5, 0)]` corresponding to c3 and a3. 
    /// 
    /// # Errors
    /// 
    /// * If the provided index is invalid the function returns Err(String)
    pub fn get_legal_moves_array_index(&mut self, index : (usize, usize)) -> Result<Vec<(usize, usize)>, String>{
        let (i, j) = index;
        
        //return err if position is invalid
        if !is_valid_pos(i as i32, j as i32){
            return Err(format!("Invalid index {:?}", index));
        }
        
        let color = match self.board[i][j] {
            Some(piece) => piece.color,
            None => return Ok(Vec::new()),
        };

        let pos = (i, j);
        //i, j already validated, so unwrap is safe
        let pseudo_legal_moves = self.get_pseudo_legal_moves_for_square(i, j, false).unwrap();
        let mut legal_moves = Vec::new();

        for mve in pseudo_legal_moves {
            //both pos and mve are valid indicies, so unwrap is sage
            self.make_move_with_index(pos, mve, false, true).unwrap();

            if !self.in_check(color) {
                legal_moves.push(mve);
            }
            
            self.undo_last_move();
        }

        return Ok(legal_moves);
    }

    /// Get all legal moves for a player (color) in a given position. 
    /// 
    /// # Arguments
    /// 
    /// * color `Color` of player who's moves should be retrieved.
    /// 
    /// # Returns
    /// 
    /// * `HashMap` containing all possible moves for each piece of given color.
    /// Each key is `(usize, usize)` representing the index of a piece. 
    /// Each value is a `Vec<(usize, usize)>` representing all indicies to which
    /// the piece can move to.
    /// 
    pub fn get_all_legal_moves(&mut self, color : Color) -> HashMap<(usize, usize), Vec<(usize, usize)>> {
        let mut move_hash : HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

        for i in 0..8 {
            for j in 0..8 {
                if let Some(piece) = self.board[i][j] {
                    if piece.color == color {
                        //i, j will always be a valid index, so unwrap is safe
                        let legal_moves = self.get_legal_moves_array_index((i, j)).unwrap();
                        move_hash.insert((i, j), legal_moves);
                    }
                }
            }
        }

        return move_hash;
    }

    /// Returns bool representing wether a player is in check or not.
    pub fn in_check(&self, color : Color) -> bool {
        let attacked_squares = self.get_attacked_squares(color.opposite());

        //Find king position
        for i in 0..8 {
            for j in 0..8 {
                if let Some(piece) = self.board[i][j]{
                    if piece.piece_type == PieceType::King
                    && piece.color == color
                    {
                        return attacked_squares.contains(&(i, j));
                    }
                }
            }
        }

        return false;
    }

    /// Returns current state of the game. For possible game states,
    /// refer to documentation for `GameState` enum.
    pub fn get_state(&mut self) -> GameState{
        if self.promotion_square.is_some() {
            return GameState::AwaitPromotion;
        }

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

    /// Returns color of active player
    pub fn get_active_player(&self) -> Color {
        self.turn
    }

    /// Returns `vec` of each `Piece` that `color` has captured
    /// during the game.
    /// 
    /// # Notes
    /// * Only records moves made through the Game object using any
    /// implementation of make_move() method. Positions generated
    /// from FEN will not have captures recorded properly. 
    pub fn get_captures(&self, color : Color) -> Vec<Piece>{
        let mut res = Vec::new();

        for p in &self.captures {
            if p.color == color.opposite() {
                res.push(*p);
            }
        }

        res
    }

    //function to handle movement logic
    fn make_move_with_index(&mut self, from : (usize, usize), to : (usize, usize), check_legal : bool, auto_promote : bool) -> Result<bool, String> {
        let (i1, j1) = from;
        let (i2, j2) = to;

        //return if move is illegal
        //ignored if check_legal is false
        if check_legal{
            if let Ok(Some(piece)) = self.piece_at_array_index((i1, j1)) {
                if piece.color != self.turn {
                    return Ok(false);
                }
            }
            //get_legal_moves_square() will always return Some() since
            //index (i1, j1) is validated in make_move_array_index()
            if !(self.get_legal_moves_array_index((i1, j1)).unwrap().contains(&(i2, j2))) {
                return Ok(false);
            }
        }

        //save board state
        self.previous_state = Some(Box::new(self.clone()));

        //increment half moves, if there is a capture or pawn move this will be reset
        self.half_moves += 1;

        //Capture logic
        if let Some(piece) = self.board[i2][j2] {
            self.captures.push(piece);
            self.half_moves = 0; //piece captured : resets half moves
        }

        //Check if castling
        //note board[i1][j1] is always Some(Piece) due to how
        //this function is called, so unwrap() wont panic
        if self.board[i1][j1].unwrap().piece_type == PieceType::King {
            let d = j1 as i32 - j2 as i32;

            //check if king is moved 2 squares
            if d.abs() == 2 {
                //remove castling rights
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
            //remove castling rights if the rook is moved

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
            self.half_moves = 0; //pawn moved : reset half moves

            let pawn_color = self.board[i1][j1].unwrap().color;

            //check if pawn is moved two squares
            let d = i1 as i32 - i2 as i32;

            if d.abs() == 2 {
                self.en_passant_square = Some(((i1 + i2) / 2, j1))
            }

            if self.is_promotion_move(from, to) {
                self.promotion_square = Some((i2, j2));
            }

            if self.en_passant_square.is_some(){
                if (i2, j2) == self.en_passant_square.unwrap() {
                    match pawn_color {
                        Color::White => self.board[i2 + 1][j2] = None,
                        Color::Black => self.board[i2 - 1][j2] = None,
                    }
                }
            }
        }

        if let Some(piece) = self.board[i2][j2] {
            if piece.piece_type == PieceType::Rook {
                //remove castling rights if the rook is captured

                let rook_color = self.board[i2][j2].unwrap().color;

                let starting_rank = match rook_color {
                    Color::White => 7,
                    Color::Black => 0,
                };

                if i2 == starting_rank {
                    match j2 {
                        0 => {self.queenside_castle.insert(rook_color, false);},
                        7 => {self.kingside_castle.insert(rook_color, false);},
                        _ => (),
                    }
                }
            }
        }

        //make move
        self.board[i2][j2] = self.board[i1][j1];
        self.board[i1][j1] = None;

        if auto_promote {
            self.promote_to_piece(PieceType::Queen);
        }

        self.update_attacked_squares();

        if self.turn == Color::Black {
            self.full_moves += 1;
        }

        self.en_passant_square = None;
        self.turn = self.turn.opposite();

        Ok(true)
    }

    /// Checks wether or not a move is a promotion move
    fn is_promotion_move(&self, from : (usize, usize), to : (usize, usize)) -> bool {
            if is_valid_move(from, to){
                let pawn_color = self.board[from.0][from.1].unwrap().color;
                
                let promotion_rank = match pawn_color {
                    Color::White => 0,
                    Color::Black => 7,
                };
                
                return to.0 == promotion_rank;
            }

            return false;
        }

    ///Returns Result, if Ok -> Vector of all legal moves (usize, usize) for the given square
    /// 
    /// Returns Err if provided index is invalid
    fn get_pseudo_legal_moves_for_square(&self, i : usize, j : usize, only_attacked : bool) -> Result<Vec<(usize, usize)>, String>{
        if !is_valid_pos(i as i32, j as i32) {
            return Err(format!("Invalid index : Cannot compute pseudo-legal moves for index {i}, {j}"))
        }

        //since i, j is validated as a position all calls to pseudo_legal_moves
        //will not panic when calling unwrap() in the respective function
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

    /// compute pseudo-legal moves for pieces that move in given directions
    /// max_moves indicates how far a piece can "slide"
    /// used for calculating pseudo-legal moves for every piece except for the pawn and king*
    /// 
    /// * the king has it's own function to include castling, but uses this function as well
    /// 
    /// # Panics
    /// Function panics if there is not a piece at index i, j
    /// 
    /// Function should only be called thorugh get_pseudo_legal_moves_for_square() 
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


    /// # Panics
    /// Function panics if there is not a piece at index i, j
    /// 
    /// Function should only be called thorugh get_pseudo_legal_moves_for_square() 
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
        if let Some(en_passant_square) = self.en_passant_square{
            if en_passant_square == (i, j){
                match can_en_passant(i) {
                    Some(color) => return color == pawn_color,
                    None => return false,
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

    /// # Panics
    /// Function panics if there is not a piece at index i, j.
    /// 
    /// Will panic if Game.kingside_castle or Game.queenside_castle fields 
    /// are missing values for Color::White or Color::Black, but this should
    /// never be an issue when using constructors in the Game struct.
    /// 
    /// Function should only be called thorugh get_pseudo_legal_moves_for_square(), 
    /// this will guarantee index i, j is a Piece.  
    fn king_pseudo_legal_moves(&self, i : usize, j : usize, include_all_attacked : bool) -> Vec<(usize, usize)> {
        let king_color = self.board[i][j].unwrap().color;
        let mut move_vec = self.directional_pseudo_legal_moves(i, j, &self.queen_move_directions, 1, include_all_attacked);

        //king_color should always be a key in kingside_castle and queenside_castle
        //unwrap is safe
        let kingside = self.kingside_castle.get(&king_color).unwrap();
        let queenside = self.queenside_castle.get(&king_color).unwrap();

        //castling logic
        
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
    
    /// Returns all squares under attack by `color`
    fn get_attacked_squares(&self, color : Color) -> &Vec<(usize, usize)> {
        match color {
            Color::White => &(self.white_attacked_squares),
            Color::Black => &(self.black_attacked_squares),
        }
    }

    /// Update `white_attacked_squares` and `black_attacked_squares` field
    /// in the Game object.
    fn update_attacked_squares(&mut self) {
        let mut white_attack_vec : Vec<(usize, usize)> = Vec::new();
        let mut black_attack_vec : Vec<(usize, usize)> = Vec::new();

        for i in 0..8 {
            for j in 0..8 {
                //check is board[i][j] is some, else get_pseudo_legal_moves_for_square will panic
                if let Some(piece) = self.board[i][j]{
                    match piece.color{
                        //get_pseudo_legal_moves_for_square will return Some(), since
                        //board[i][j] is a Piece, so the unwrap is safe
                        Color::White => white_attack_vec.append(&mut self.get_pseudo_legal_moves_for_square(i, j, true).unwrap()),
                        Color::Black => black_attack_vec.append(&mut self.get_pseudo_legal_moves_for_square(i, j, true).unwrap()),
                    }
                }
            }
        }

        self.white_attacked_squares = white_attack_vec;
        self.black_attacked_squares = black_attack_vec;
    }

    /// Returns how many legal moves player `color` has in a given position.
    fn num_of_legal_moves(&mut self, color : Color) -> u32 {
        let mut res = 0;

        for moves in self.get_all_legal_moves(color).values(){
            res += moves.len();
        }

        return res as u32;
    }

    /// Checks if `color` has enough pieces to win.
    fn can_win(&self, color : Color) -> bool {

        let mut pieces = Vec::new();

        for i in 0..8 {
            for j in 0..8 {
                if let Some(piece) = self.board[i][j]{
                    if piece.color == color {
                        pieces.push(piece.piece_type);
                    }
                }
            }
        }

        return !self.insufficient_material.contains(&pieces);
    }
}


/// Enum for representing the state of a chess game.
/// 
/// # Values
/// * `InProgress`: The game is ongoing.
/// * `AwaitPromotion`: Waiting for user to choose promotion piece. 
/// * `Win(WinState)`: One player has won, which player won and how they won
/// is defined in `WinState`.
/// * `Draw`: Position is a draw, the cause for the draw is defined in `DrawState` 
#[derive(Debug, Clone, PartialEq)]
pub enum GameState {
    InProgress,
    AwaitPromotion,
    Win(WinState),
    Draw(DrawState),
}

/// Draw states used in `GameState::Draw`
#[derive(Debug, Clone, PartialEq)]
pub enum DrawState {
    Stalemate,
    InsufficientMaterial,
    FiftyMoveRule
}
#[derive(Debug, Clone, PartialEq)]
/// Win state used in `GameState::Win`.
/// `Color` represents the color of the winner.
pub enum WinState {
    Checkmate(Color)
}
/// Struct for representing a chess piece.
/// 
/// # Creation
/// * `piece_type` represents what type of piece it is e.g. pawn, 
/// knight, bishop etc.
/// * `color` represents the color of the piece, `Color::White` or `Color::Black`
/// 
/// # Examples
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Piece {
    pub piece_type : PieceType,
    pub color : Color,
}

impl Piece {
    pub fn new(piece_type : PieceType, color : Color) -> Piece{
        Piece {
            piece_type,
            color,
        }
    }
}

/// Enum for all types of standard chess pieces
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// Enum for piece color
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[derive(Hash)]
pub enum Color {
    White,
    Black,
}

impl Color {
    /// Returns the opposite color of the piece
    pub fn opposite(&self) -> Color {
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

/// Get array indicies for a give `notation` written in
/// algebraic notation.
/// 
/// # Arguments
/// * `notation` is a `str` describing a square on the board in algebraic notation.
/// 
/// # Returns
/// * A `Result` containing the array index `(usize, usize)` corresponding
/// to the input algebraic notation.
/// 
/// # Errors
/// * Returns `Err(String)` if the provided notation is invalid
pub fn alg_notation_to_indx(notation : &str) -> Result<(usize , usize), String> {
    let chr_vec = notation
        .chars()
        .collect::<Vec<char>>();

    if chr_vec.len() != 2 {
        return Err(format!("Invalid notation {}", notation));
    }

    let col : usize = match chr_vec[0] {
        'a' => 0,
        'b' => 1,
        'c' => 2,
        'd' => 3,
        'e' => 4,
        'f' => 5,
        'g' => 6,
        'h' => 7,
        _c => return Err(format!("Invalid file {}", _c)),
    };

    // 8 - n since ranks in the array are mirrored, and the first rank is at index 7
    let row = match chr_vec[1].to_digit(10) {
        Some(digit) => 8 - digit as usize,
        None => return Err(format!("Invalid row {}", chr_vec[1]))
    };
    
    
    return Ok((row, col));
}

/// Get algebraic notation for a given `indx`.
/// 
/// # Returns
/// * `Result` containing the algebraic notation as a `String`
/// 
/// # Errors
/// * Returns `Err(String)` if provided index is invalid.
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
        5 => Some(Color::Black),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn piece_getter_test() {
        let game = Game::new_starting_pos();
     
        let piece = game.piece_at_array_index((0,0));
        let top_left_piece = Piece::new(PieceType::Rook, Color::Black);
    
        assert_eq!(piece, Ok(Some(top_left_piece)));
    }

    #[test]

    fn possible_moves_test() {
        let mut board = Game::new_starting_pos();

        let x : HashMap<(usize, usize), Vec<(usize, usize)>> = board.get_all_legal_moves(Color::White);

        let mut expected_map = HashMap::new();
        expected_map.insert((6, 6), vec![(5, 6), (4, 6)]);
        expected_map.insert((7, 2), vec![]);
        expected_map.insert((6, 4), vec![(5, 4), (4, 4)]);
        expected_map.insert((6, 2), vec![(5, 2), (4, 2)]);
        expected_map.insert((7, 0), vec![]);
        expected_map.insert((7, 4), vec![]);
        expected_map.insert((7, 5), vec![]);
        expected_map.insert((7, 3), vec![]);
        expected_map.insert((6, 7), vec![(5, 7), (4, 7)]);
        expected_map.insert((6, 1), vec![(5, 1), (4, 1)]);
        expected_map.insert((7, 1), vec![(5, 2), (5, 0)]);
        expected_map.insert((6, 3), vec![(5, 3), (4, 3)]);
        expected_map.insert((6, 0), vec![(5, 0), (4, 0)]);
        expected_map.insert((6, 5), vec![(5, 5), (4, 5)]);
        expected_map.insert((7, 6), vec![(5, 7), (5, 5)]);
        expected_map.insert((7, 7), vec![]);

        assert_eq!(x, expected_map);
    }

    #[test]
    fn legal_moves_square_test() {
        let mut game = Game::new_starting_pos();

        let expected_val : Vec<(usize, usize)> = Vec::from([(5, 2), (5, 0)]);

        //print legal moves for knight on b1
        assert_eq!(expected_val, game.get_legal_moves_alg_notation("b1").ok().unwrap());
    }

    #[test]

    //tests make_move function with different inputs
    fn move_test() {   
        let mut board = Game::new_starting_pos();

        let valid_move = board.make_move("e2", "e4", false);
        let invalid_move = board.make_move("f2", "f5", false);
        let invalid_move2 = board.make_move("f4", "f5", false);
        let invalid_input = board.make_move("aksmldkams", "poköakenjf", false);
        let empty_input = board.make_move("", "", false);

        assert_eq!(valid_move, Ok(true));
        assert_eq!(invalid_move, Ok(false));
        assert_eq!(invalid_move2, Ok(false));
        assert_eq!(invalid_input.is_err(), true);
        assert_eq!(empty_input.is_err(), true);
    }

    #[test]
    fn castling_test() {
        let mut board = Game::from_fen("r1bqkbnr/pppppppp/8/8/8/6n1/PPPPPPP1/RNBQK2R b KQkq - 0 1").unwrap();

        board.make_move("g3", "e4", true).unwrap();

        println!("{:?}", board);
        println!("{:?}", board.get_legal_moves_alg_notation("e1").unwrap());
    }

    #[test]

    fn undo_move_test() {
        let mut board = Game::new_starting_pos();

        board.make_move("e2", "e4", false).unwrap();

        board.undo_last_move();

        assert_eq!(board.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    #[test]

    //Shows promotion functionality
    //Also shows piece_at...() functionality
    fn promotion_test() {
        let mut board = Game::from_fen("8/1P6/8/8/8/8/1p6/8 w - - 0 1").unwrap();

        board.make_move("b7", "b8", false).unwrap();

        if board.get_state() == GameState::AwaitPromotion{
            board.promote_to_piece(PieceType::Queen);
        }
        
        assert_eq!(board.piece_at_alg_notation("b8").ok().unwrap(), 
            Some(Piece::new(PieceType::Queen, Color::White)))
    }
    
    #[test]

    //Make a board from FEN string
    //Also displays functionality of get_state()
    fn board_from_fen_test(){
        let board = Game::from_fen("k1Q2b2/pp6/1qp2p2/3P3p/2p2B1P/2P5/PP4r1/1K1R4 b - - 1 34").unwrap();
            
        assert_eq!(board.to_fen(), "k1Q2b2/pp6/1qp2p2/3P3p/2p2B1P/2P5/PP4r1/1K1R4 b - - 1 34");
    }

    #[test]

    //Shows functionality of new_starting_pos() and to_fen()
    fn board_to_fen_test(){
        let board = Game::new_starting_pos();

        assert_eq!(board.to_fen(), "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
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
            board.make_move(from, to, true).unwrap();
        }
        
        assert_eq!(board.get_state(), GameState::Win(WinState::Checkmate(Color::White)));
    }
}
