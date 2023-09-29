# README

API for chess logic.

## Table Of Contents

- [Initializing the board](#initializing-the-board)
- [From FEN](#from-fen)
- [Fetching Data From the Board](#fetching-data-from-the-board)
    - [Pieces](#pieces)
    - [Game State](#game-state)
        - [Board State](#board-state)
        - [Current Turn](#current-turn)
    - [Legal Moves](#legal-moves)
- [Moving](#moving)
    - [Notation](#notation)
    - [Making Moves](#making-moves)
    - [Promotion](#promotion)
- [Piece](#piece)
- [Game Loop Example](#game-loop-example)
- [Contact](#contact)

# API Usage

## Initializing the board

To initialize to a board, you create an instance of the `Game` struct. Use the provided constructor `Game::new_starting_pos()` to create a board with the standard starting position, this can be done like this:

```rust
let mut game = Game::new_starting_pos();
```

### From FEN

Initializing the board from a FEN-string is also possible using the `Game::from_fen()` constructor. 

Example illustrating how to load a chess puzzle into a `Game` instance

```rust
let mut game = Game::from_fen("5r2/8/1R6/ppk3p1/2N3P1/P4b2/1K6/5B2 w - - 0 1");
```

For details on FEN notation, refer to: https://en.wikipedia.org/wiki/Forsyth–Edwards_Notation

## Fetching data from the board

### Pieces

To read piece data from the board use the `piece_at_alg_notation()` method or the `piece_at_array_index` method. Both methods return what is currently at the given location on the board. 

Each position on the board is an `Option<Piece>`, being `None` if the position is empty. Note that `piece_at_alg_notation()` and `piece_at_array_index` will return a `Result` containing the `Option<Piece>`.

Example

```rust
let mut game = Game::new_starting_pos();

let piece_data = game.piece_at_alg_notation("e2");

println!("{:?}", piece_data);
```

This will output

```
Ok(Some(Piece { piece_type: Pawn, color: White }))
```

The exact same functionality can be achieved using the `piece_at_array_index` like so:

```rust
let mut game = Game::new_starting_pos();

let piece_data = game.piece_at_array_index((6, 4));

println!("{:?}", piece_data);
```

### Game state

#### Board state
You can get the board state by calling the `get_state()` method on an instance of `Game`.
This will return a `GameState` enum that can take one of the following values:
- `InProgress`
- `AwaitPromotion`
- `Win(WinState)`
- `Draw(DrawState)`

#### Current turn

Getting the color of the active player (current turn) can be done by using the `get_active_color()` method. 

### Legal Moves

Getting the legal moves for a piece can be done using the `get_legal_moves_alg_notation()` or the
`get_legal_moves_array_index()` method. These will return the possible moves for _a single_ piece

Getting all possible moves in a position can be done with `get_all_legal_moves(color)`. This will return
all possible moves for the player corresponding to `color`. 

## Moving
### Notation
The API provides support for both algebraic notation as well as array index notation. 

- For algebraic notation, refer to: https://www.chess.com/terms/chess-notation#readalgebraic
- Array index notation is expressed as `(i, j)` where `i` represents the rank and `j` represents the file. Note that `i = 0` is the top row (equivalent to 8 in algebraic notation)

Algebraic notation is only meant for input, all output from methods will always be expressed as array indices. For converting between notations there are two provided functions `indx_to_alg_notation()` and `alg_notation_to_indx()`.
### Making moves

Move a piece on the board using either the `make_move()` method or `make_move_array_index()`.

- `make_move()` uses algebraic notation e.g
```rust
game.make_move("e2", "e4", false);
```
- `make_move_array_index()` uses array indices, equivalent to the example above: 
```rust
game.make_move((4, 6), (4, 4), false);
```
### Promotion

Promotion can either be done automatically, or handled by the user. 

Promotion will be automatic if the `auto_promote` parameter in `make_move()` is set to `true`. This will automatically promote a pawn to a queen when it reaches the end of the board.

In order to choose the promotion piece, the `auto_promote` parameter in `make_move()` should be set to `flase`. This will cause the `get_state()` method to return `GameState::AwaitPromotion` when a piece needs to be promoted. For examples on this see [Game Loop Example](#game-loop-example)

## Piece

The `Piece` struct has two fields:
- `piece_type` being an instance of the `PieceType` enum. 
- `color` being an instance of the `Color` enum.

## Game Loop Example

This is an example of how a generic game loop may look like. 

```rust
let mut game = Game::new_starting_pos();

let mut running = true;

while running {
    input = ... //get user input

    if let move_data = game.make_move(input.from, input.to) {
        if !move_data {continue;}

        match game.get_state(){
            AwaitPromotion => {
                let promote_piece_type = ... //prompt user to choose piece type
                game.promote_to_piece(promote_piece_type);
            }
            //handle the remaining game states...
        }

        //Display board and other logic...
    }
}

```

## Contact

For bug reports, feature requests, or other issues, please submit an issue at https://github.com/IndaPlus23/nohhy-chess/issues