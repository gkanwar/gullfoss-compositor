const WIDTH: usize = 8;
const HEIGHT: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
  Black,
  White,
}
fn opposite(color: Color) -> Color {
  match color {
    Color::Black => Color::White,
    Color::White => Color::Black,
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PieceKind {
  Pawn,
  Knight,
  Bishop,
  Rook,
  Queen,
  King,
}
#[derive(Clone, Copy, Debug)]
struct Piece {
  color: Color,
  kind: PieceKind,
}
#[derive(Clone, Copy, Debug)]
enum SquareContent {
  Empty,
  Filled(Piece),
}

#[derive(Clone, Debug)]
struct BoardState {
  squares: [[SquareContent; HEIGHT]; WIDTH],
}
impl BoardState {
  fn new() -> Self {
    BoardState {
      squares: [[SquareContent::Empty; HEIGHT]; WIDTH],
    }
  }
}

#[derive(Debug)]
struct CastlingState {
  wk: bool,
  wq: bool,
  bk: bool,
  bq: bool,
}
type Square = (usize, usize);

#[derive(Debug)]
struct GameState {
  board: BoardState,
  castling: CastlingState,
  active: Color,
  en_passant: Option<Square>,
  // TODO: move counters
}
impl GameState {
  fn new() -> Self {
    GameState {
      board: BoardState::new(),
      castling: CastlingState { wk: true, wq: true, bk: true, bq: true },
      active: Color::White,
      en_passant: None,
    }
  }
}

#[derive(Clone, Debug)]
enum Move {
  Normal {
    origin: Square,
    target: Square,
  },
  Promote {
    origin: Square,
    target: Square,
    kind: PieceKind,
  },
  CastleK,
  CastleQ,
}

fn in_bounds((x, y): (i32, i32)) -> bool {
  0 <= x && x < (WIDTH as i32) && 0 <= y && y < (HEIGHT as i32)
}

fn get_piece_pseudolegal_moves(
  (x, y): (usize, usize), color: Color, kind: PieceKind, board: &BoardState,
) -> Vec<Move> {
  let mut moves = Vec::<Move>::new();
  let (x, y) = (x as i32, y as i32);

  let add_normal_move = |(xp, yp): (i32, i32), moves: &mut Vec<Move>| {
    moves.push(Move::Normal {
      origin: (x as usize, y as usize),
      target: (xp as usize, yp as usize),
    });
  };

  let check_square_piece = |(xp, yp): (i32, i32)| -> Option<Color> {
    if let SquareContent::Filled(piece) = board.squares[xp as usize][yp as usize] {
      return Some(piece.color);
    }
    return None;
  };

  // add square if valid, return whether space was free
  let maybe_add_square = |(xp, yp): (i32, i32), moves: &mut Vec<Move>| -> bool {
    match check_square_piece((xp, yp)) {
      Some(piece_color) => {
        if piece_color != color {
          add_normal_move((xp, yp), moves);
        }
        return false;
      }
      None => {
        add_normal_move((xp, yp), moves);
        return true;
      }
    }
  };

  let is_occupied = |(x, y): (i32, i32)| -> bool {
    match board.squares[x as usize][y as usize] {
      SquareContent::Filled(_) => true, // TODO
      SquareContent::Empty => false,
    }
  };

  let add_pawn_moves = |moves: &mut Vec<Move>| {
    let sign = match color {
      Color::White => 1,
      Color::Black => -1,
    };
    let queen_rank: i32 = match color {
      Color::White => (HEIGHT - 1) as i32,
      Color::Black => 0,
    };
    let push_one_sq = (x, y + sign);
    let push_two_sq = (x, y + 2 * sign);
    if in_bounds(push_one_sq) && !is_occupied(push_one_sq) {
      if y + sign != queen_rank {
        add_normal_move(push_one_sq, moves);
      }
      else {
        // promotion
        for kind in &[PieceKind::Knight, PieceKind::Bishop, PieceKind::Rook, PieceKind::Queen] {
          moves.push(Move::Promote {
            origin: (x as usize, y as usize),
            target: (push_one_sq.0 as usize, push_one_sq.1 as usize),
            kind: *kind,
          });
        }
      }
      if y == 1 && !is_occupied(push_two_sq) {
        add_normal_move(push_two_sq, moves);
      }
    }
    // diagonal captures
    let diag_right_sq = (x + 1, y + sign);
    let diag_left_sq = (x - 1, y + sign);
    if in_bounds(diag_left_sq) {
      if let Some(piece_color) = check_square_piece(diag_left_sq) {
        if piece_color != color {
          add_normal_move(diag_left_sq, moves);
        }
      }
    }
    if in_bounds(diag_right_sq) {
      if let Some(piece_color) = check_square_piece(diag_right_sq) {
        if piece_color != color {
          add_normal_move(diag_right_sq, moves);
        }
      }
    }
  };

  let add_diagonal_moves = |moves: &mut Vec<Move>| {
    for xp in (x + 1)..((WIDTH as i32) - 1) {
      let yp = y - (x - xp);
      if !in_bounds((xp, yp)) {
        break;
      }
      if !maybe_add_square((xp, yp), moves) {
        break;
      }
    }
    for xp in (x + 1)..((WIDTH as i32) - 1) {
      let yp = y + (x - xp);
      if !in_bounds((xp, yp)) {
        break;
      }
      if !maybe_add_square((xp, yp), moves) {
        break;
      }
    }
    for xp in (0..x).rev() {
      let yp = y - (x - xp);
      if !in_bounds((xp, yp)) {
        break;
      }
      if !maybe_add_square((xp, yp), moves) {
        break;
      }
    }
    for xp in (0..x).rev() {
      let yp = y + (x - xp);
      if !in_bounds((xp, yp)) {
        break;
      }
      if !maybe_add_square((xp, yp), moves) {
        break;
      }
    }
  };

  let add_cardinal_moves = |moves: &mut Vec<Move>| {
    for xp in x + 1..(WIDTH as i32) - 1 {
      if !maybe_add_square((xp, y), moves) {
        break;
      }
    }
    for xp in (0..x).rev() {
      if !maybe_add_square((xp, y), moves) {
        break;
      }
    }
    for yp in y + 1..(WIDTH as i32) - 1 {
      if !maybe_add_square((x, yp), moves) {
        break;
      }
    }
    for yp in (0..y).rev() {
      if !maybe_add_square((x, yp), moves) {
        break;
      }
    }
  };

  let add_knight_moves = |moves: &mut Vec<Move>| {
    const KNIGHT_OFFSETS: &[(i32, i32)] =
      &[(-1, 2), (1, 2), (2, 1), (2, -1), (-1, -2), (1, -2), (-2, 1), (-2, -1)];
    for (dx, dy) in KNIGHT_OFFSETS.iter() {
      let square = (x + dx, y + dy);
      if in_bounds(square) {
        maybe_add_square(square, moves);
      }
    }
  };

  let add_king_moves = |moves: &mut Vec<Move>| {
    const KING_OFFSETS: &[(i32, i32)] =
      &[(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
    for (dx, dy) in KING_OFFSETS.iter() {
      let square = ((x as i32) + dx, (y as i32) + dy);
      if in_bounds(square) {
        maybe_add_square(square, moves);
      }
    }
  };

  match kind {
    PieceKind::Pawn => {
      add_pawn_moves(&mut moves);
    }
    PieceKind::Knight => {
      add_knight_moves(&mut moves);
    }
    PieceKind::Bishop => {
      add_diagonal_moves(&mut moves);
    }
    PieceKind::Rook => {
      add_cardinal_moves(&mut moves);
    }
    PieceKind::Queen => {
      add_diagonal_moves(&mut moves);
      add_cardinal_moves(&mut moves);
    }
    PieceKind::King => {
      add_king_moves(&mut moves);
      // TODO: castling
    }
  }
  return moves;
}

fn apply_move(m: &Move, bs: &BoardState) -> BoardState {
  let mut bs2 = bs.clone();
  let squares = &mut bs2.squares;
  match m {
    Move::Normal { origin, target } => {
      squares[target.0][target.1] = squares[origin.0][origin.1];
      squares[origin.0][origin.1] = SquareContent::Empty;
    }
    Move::Promote { origin, target, kind } => {
      squares[target.0][target.1] = squares[origin.0][origin.1];
      if let SquareContent::Filled(piece) = squares[target.0][target.1] {
        squares[target.0][target.1] =
          SquareContent::Filled(Piece { color: piece.color, kind: *kind });
      }
      else {
        panic!("Promotion from an empty square");
      }
      squares[origin.0][origin.1] = SquareContent::Empty;
    }
    // TODO: castling
    Move::CastleK => {
      unimplemented!();
    }
    Move::CastleQ => {
      unimplemented!();
    }
  }
  return bs2;
}

fn get_pseudolegal_moves(board: &BoardState, active: Color) -> Vec<Move> {
  let mut moves = Vec::<Move>::new();
  for (i, row) in board.squares.iter().enumerate() {
    for (j, sq) in row.iter().enumerate() {
      if let SquareContent::Filled(Piece { color, kind }) = sq {
        if *color != active {
          continue;
        }
        moves.extend(get_piece_pseudolegal_moves((i, j), *color, *kind, &board).iter().cloned());
      }
    }
  }
  return moves;
}

fn in_check(board: &BoardState, color: Color) -> bool {
  for m in get_pseudolegal_moves(board, opposite(color)) {
    match m {
      Move::Normal { origin, target } => {
        if let SquareContent::Filled(piece) = board.squares[target.0][target.1] {
          if piece.color == color && piece.kind == PieceKind::King {
            return true;
          }
        }
      }
      _ => {}
    }
  }
  return false;
}

fn get_legal_moves(gs: &GameState) -> Vec<Move> {
  return get_pseudolegal_moves(&gs.board, gs.active)
    .iter()
    .filter(|m| {
      let bs2 = apply_move(m, &gs.board);
      return !in_check(&bs2, gs.active);
    })
    .cloned()
    .collect();
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
        board_state.squares[j][HEIGHT - i - 1] = SquareContent::Filled(match c {
          'p' => Piece {
            color: Color::Black,
            kind: PieceKind::Pawn,
          },
          'r' => Piece {
            color: Color::Black,
            kind: PieceKind::Rook,
          },
          'n' => Piece {
            color: Color::Black,
            kind: PieceKind::Knight,
          },
          'b' => Piece {
            color: Color::Black,
            kind: PieceKind::Bishop,
          },
          'q' => Piece {
            color: Color::Black,
            kind: PieceKind::Queen,
          },
          'k' => Piece {
            color: Color::Black,
            kind: PieceKind::King,
          },
          'P' => Piece {
            color: Color::White,
            kind: PieceKind::Pawn,
          },
          'R' => Piece {
            color: Color::White,
            kind: PieceKind::Rook,
          },
          'N' => Piece {
            color: Color::White,
            kind: PieceKind::Knight,
          },
          'B' => Piece {
            color: Color::White,
            kind: PieceKind::Bishop,
          },
          'Q' => Piece {
            color: Color::White,
            kind: PieceKind::Queen,
          },
          'K' => Piece {
            color: Color::White,
            kind: PieceKind::King,
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
            for k in j..j + num_spaces {
              board_state.squares[k][HEIGHT - i - 1] = SquareContent::Empty;
            }
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
    bq: false,
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
  let row = &sq_str[1..];
  let mut row_i: usize;
  match row.parse::<usize>() {
    Ok(i) => {
      row_i = i - 1;
    }
    Err(_) => {
      return Err("Invalid square row");
    }
  }
  let square = (col_i, row_i);
  if !in_bounds((square.0 as i32, square.1 as i32)) {
    return Err("Square not in bounds");
  }
  return Ok(square);
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
    None => Err("FEN missing board string"),
  }?;
  game_state.board = parse_fen_board(board_str)?;

  let active = match tok_iter.next() {
    Some(s) => Ok(s),
    None => Err("FEN missing active string"),
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
    None => Err("FEN missing castling string"),
  }?;
  game_state.castling = parse_fen_castling(castle_str)?;

  let en_passant_str = match tok_iter.next() {
    Some(s) => Ok(s),
    None => Err("FEN missing en passant string"),
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

  #[test]
  fn test_legal_moves() {
    let gs: GameState =
      parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let m = get_legal_moves(&gs);
    assert_eq!(m.len(), 20);
  }
}

fn main() {
  // TODO
}
