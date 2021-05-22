const WIDTH: usize = 8;
const HEIGHT: usize = 8;

#[derive(Clone, Copy)]
enum Color {
  Black,
  White
}
#[derive(Clone, Copy)]
enum PieceKind {
  Pawn,
  Knight,
  Bishop,
  Rook,
  Queen,
  King
}
#[derive(Clone, Copy)]
struct Piece {
  color: Color,
  kind: PieceKind
}
#[derive(Clone, Copy)]
enum SquareContent {
  Empty,
  Filled(Piece)
}

struct BoardState {
  squares: [[SquareContent; HEIGHT]; WIDTH]
}
impl BoardState {
  fn new() -> Self {
    BoardState {
      squares: [[SquareContent::Empty; HEIGHT]; WIDTH]
    }
  }
}

struct CastlingState {
  wk: bool,
  wq: bool,
  bk: bool,
  bq: bool
}
type Square = [usize; 2];

struct GameState {
  board: BoardState,
  castling: CastlingState,
  active: Color,
  en_passant: Option<Square>
  // TODO: move counters
}
impl GameState {
  fn new() -> Self {
    GameState {
      board: BoardState::new(),
      castling: CastlingState {
        wk: true,
        wq: true,
        bk: true,
        bq: true
      },
      active: Color::White,
      en_passant: None
    }
  }
}

fn parse_fen_board(board_str: &str) -> Result<BoardState, &'static str> {
  let rows: Vec<_> = board_str.split("/").collect();
  if rows.len() != HEIGHT {
    return Err("FEN board state has wrong number of rows");
  }
  let mut board_state = BoardState::new();
  for (i, r) in rows.iter().enumerate() {
    let mut j = 0;
    for c in r.chars() {
      if "prnbqkPRNBQK".contains(c) {
        if j + 1 > WIDTH {
          return Err("Too many squares in FEN board row");
        }
        board_state.squares[i][j] = SquareContent::Filled(match c {
          'p' => Piece {
            color: Color::Black,
            kind: PieceKind::Pawn
          },
          'r' => Piece {
            color: Color::Black,
            kind: PieceKind::Rook
          },
          'n' => Piece {
            color: Color::Black,
            kind: PieceKind::Knight
          },
          'b' => Piece {
            color: Color::Black,
            kind: PieceKind::Bishop
          },
          'q' => Piece {
            color: Color::Black,
            kind: PieceKind::Queen
          },
          'k' => Piece {
            color: Color::Black,
            kind: PieceKind::King
          },
          'P' => Piece {
            color: Color::White,
            kind: PieceKind::Pawn
          },
          'R' => Piece {
            color: Color::White,
            kind: PieceKind::Rook
          },
          'N' => Piece {
            color: Color::White,
            kind: PieceKind::Knight
          },
          'B' => Piece {
            color: Color::White,
            kind: PieceKind::Bishop
          },
          'Q' => Piece {
            color: Color::White,
            kind: PieceKind::Queen
          },
          'K' => Piece {
            color: Color::White,
            kind: PieceKind::King
          },
          _ => {
            panic!("Unknown piece")
          }
        });
        j += 1;
      }
      else {
        let s: String = [c].iter().collect();
        match s.parse::<usize>() {
          Ok(num_spaces) => {
            if j + num_spaces > WIDTH {
              return Err("Too many squares in FEN board row");
            }
            board_state.squares[i][j..j + num_spaces].fill(SquareContent::Empty);
            j += num_spaces;
          }
          Err(_) => {
            return Err("Could not parse FEN board row");
          }
        }
      }
    }
  }
  return Ok(board_state);
}

fn parse_fen_castling(castle_str: &str) -> Result<CastlingState, &'static str> {
  let mut castling_state = CastlingState {
    wk: false,
    wq: false,
    bk: false,
    bq: false
  };
  if castle_str == "-" {
    return Ok(castling_state);
  }
  for c in castle_str.chars() {
    match c {
      'K' => castling_state.wk = true,
      'Q' => castling_state.wq = true,
      'k' => castling_state.bk = true,
      'q' => castling_state.bq = true,
      _ => {
        return Err("Invalid FEN castling state");
      }
    }
  }
  return Ok(castling_state);
}

fn parse_square(sq_str: &str) -> Result<Square, &'static str> {
  if sq_str.len() < 2 {
    return Err("Invalid square string");
  }
  let mut col: char = sq_str.chars().next().unwrap().to_ascii_lowercase();
  if !col.is_ascii_alphabetic() {
    return Err("Invalid square column");
  }
  let mut buf = [0; 1];
  let col_i: usize = (col.encode_utf8(&mut buf).as_bytes()[0] - b'a').into();
  if col_i < 1 || col_i > WIDTH {
    return Err("Invalid square column");
  }
  let row = &sq_str[1..];
  let mut row_i: usize;
  match row.parse::<usize>() {
    Ok(i) => {
      row_i = i;
    }
    Err(_) => {
      return Err("Invalid square row");
    }
  }
  if row_i < 1 || row_i > HEIGHT {
    return Err("Invalid square row");
  }
  return Ok([col_i, row_i]);
}

fn parse_fen_en_passant(en_passant_str: &str) -> Result<Option<Square>, &'static str> {
  if en_passant_str == "-" {
    return Ok(None);
  }
  let sq = parse_square(en_passant_str)?;
  return Ok(Some(sq));
}

fn parse_fen(fen: &str) -> Result<GameState, &'static str> {
  let mut tok_iter = fen.split_whitespace();
  let mut game_state = GameState::new();

  let board_str = match tok_iter.next() {
    Some(s) => Ok(s),
    None => Err("FEN missing board string")
  }?;
  game_state.board = parse_fen_board(board_str)?;

  let active = match tok_iter.next() {
    Some(s) => Ok(s),
    None => Err("FEN missing active string")
  }?;
  match active {
    "w" => {
      game_state.active = Color::White;
    }
    "b" => {
      game_state.active = Color::Black;
    }
    _ => {
      return Err("FEN has invalid active color");
    }
  };

  let castle_str = match tok_iter.next() {
    Some(s) => Ok(s),
    None => Err("FEN missing castling string")
  }?;
  game_state.castling = parse_fen_castling(castle_str)?;

  let en_passant_str = match tok_iter.next() {
    Some(s) => Ok(s),
    None => Err("FEN missing en passant string")
  }?;
  game_state.en_passant = parse_fen_en_passant(en_passant_str)?;

  // TODO: halfmove clock?
  match tok_iter.next() {
    None => {
      return Ok(game_state);
    }
    _ => {}
  };
  match tok_iter.next() {
    None => {
      return Ok(game_state);
    }
    _ => {}
  };

  return Ok(game_state);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_fen() {
    let gs: GameState =
      parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let gs: GameState =
      parse_fen("rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2").unwrap();
  }
}

fn main() {
  // TODO
}
