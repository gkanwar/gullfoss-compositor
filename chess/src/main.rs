const WIDTH: usize = 8;
const HEIGHT: usize = 8;

#[derive(Clone)]
enum Color {Black, White}
#[derive(Clone)]
enum PieceKind {Pawn, Knight, Bishop, Rook, Queen, King}
#[derive(Clone)]
struct Piece {
  color: Color,
  kind: PieceKind
}

#[derive(Clone)]
enum SquareContent {
  Empty,
  Filled(Piece)
}

struct BoardState {
  // indexed as [col][row], to match chess notation
  squares: Vec<Vec<SquareContent>>
}

fn main() {
  let bs = BoardState{
    squares: vec![vec![SquareContent::Empty; HEIGHT]; WIDTH]
  };
}
