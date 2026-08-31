use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{self, Stdout, Write},
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    fn delta(self) -> (i32, i32) {
        match self {
            Self::Up => (0, -1),
            Self::Down => (0, 1),
            Self::Left => (-1, 0),
            Self::Right => (1, 0),
        }
    }

    fn is_opposite(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Up, Self::Down)
                | (Self::Down, Self::Up)
                | (Self::Left, Self::Right)
                | (Self::Right, Self::Left)
        )
    }
}

#[derive(Debug, Clone)]
struct Game {
    width: i32,
    height: i32,
    snake: VecDeque<Pos>,
    dir: Dir,
    pending_dir: Dir,
    food: Pos,
    alive: bool,
    score: u32,
}

impl Game {
    fn new(width: i32, height: i32, mut rng: impl Rng) -> Self {
        assert!(width >= 10 && height >= 10);

        let start = Pos {
            x: width / 2,
            y: height / 2,
        };
        let mut snake = VecDeque::new();
        snake.push_front(start);
        snake.push_back(Pos {
            x: start.x - 1,
            y: start.y,
        });
        snake.push_back(Pos {
            x: start.x - 2,
            y: start.y,
        });

        let food = Self::spawn_food(width, height, &snake, &mut rng);

        Self {
            width,
            height,
            snake,
            dir: Dir::Right,
            pending_dir: Dir::Right,
            food,
            alive: true,
            score: 0,
        }
    }

    fn head(&self) -> Pos {
        *self.snake.front().expect("snake has at least 1 segment")
    }

    fn spawn_food(width: i32, height: i32, snake: &VecDeque<Pos>, rng: &mut impl Rng) -> Pos {
        // naive retry; fine for small boards
        loop {
            let p = Pos {
                x: rng.random_range(0..width),
                y: rng.random_range(0..height),
            };
            if !snake.contains(&p) {
                return p;
            }
        }
    }

    fn set_dir(&mut self, dir: Dir) {
        if !dir.is_opposite(self.dir) {
            self.pending_dir = dir;
        }
    }

    fn step(&mut self, mut rng: impl Rng) {
        if !self.alive {
            return;
        }

        self.dir = self.pending_dir;
        let (dx, dy) = self.dir.delta();
        let mut next = self.head();
        next.x += dx;
        next.y += dy;

        // wall collision
        if next.x < 0 || next.x >= self.width || next.y < 0 || next.y >= self.height {
            self.alive = false;
            return;
        }

        let grows = next == self.food;

        // If not growing, the tail cell will be vacated this turn, so allow moving into it.
        if self.snake.contains(&next) {
            let tail = *self.snake.back().unwrap();
            if grows || next != tail {
                self.alive = false;
                return;
            }
        }

        self.snake.push_front(next);
        if grows {
            self.score += 1;
            self.food = Self::spawn_food(self.width, self.height, &self.snake, &mut rng);
        } else {
            self.snake.pop_back();
        }
    }

    fn render_to_string(&self) -> String {
        let mut grid = vec![vec![' '; self.width as usize]; self.height as usize];
        grid[self.food.y as usize][self.food.x as usize] = '*';

        for (i, seg) in self.snake.iter().enumerate() {
            grid[seg.y as usize][seg.x as usize] = if i == 0 { '@' } else { 'o' };
        }

        let mut out = String::new();
        out.push('+');
        out.push_str(&"-".repeat(self.width as usize));
        out.push_str("+\n");

        for row in grid {
            out.push('|');
            for c in row {
                out.push(c);
            }
            out.push_str("|\n");
        }

        out.push('+');
        out.push_str(&"-".repeat(self.width as usize));
        out.push_str("+\n");
        out.push_str(&format!("Score: {}\n", self.score));
        if !self.alive {
            out.push_str("Game Over. Press 'r' to restart, 'q' to quit.\n");
        }
        out
    }
}

fn draw(stdout: &mut Stdout, s: &str) -> crossterm::Result<()> {
    execute!(
        stdout,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        Print(s)
    )?;
    stdout.flush()?;
    Ok(())
}

fn run() -> crossterm::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut rng = rand::rng();
    let mut game = Game::new(30, 20, &mut rng);

    let tick = Duration::from_millis(90);
    let mut last_tick = Instant::now();

    loop {
        // Input
        while event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => {
                        cleanup(&mut stdout)?;
                        return Ok(());
                    }
                    KeyCode::Char('r') => {
                        game = Game::new(game.width, game.height, &mut rng);
                    }
                    KeyCode::Up | KeyCode::Char('w') => game.set_dir(Dir::Up),
                    KeyCode::Down | KeyCode::Char('s') => game.set_dir(Dir::Down),
                    KeyCode::Left | KeyCode::Char('a') => game.set_dir(Dir::Left),
                    KeyCode::Right | KeyCode::Char('d') => game.set_dir(Dir::Right),
                    _ => {}
                }
            }
        }

        // Update
        if last_tick.elapsed() >= tick {
            game.step(&mut rng);
            last_tick = Instant::now();
        }

        // Render
        let frame = game.render_to_string();
        draw(&mut stdout, &frame)?;

        std::thread::sleep(Duration::from_millis(8));
    }
}

fn cleanup(stdout: &mut Stdout) -> crossterm::Result<()> {
    execute!(stdout, cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn main() -> crossterm::Result<()> {
    let res = run();
    // best-effort cleanup if `run` errored
    let mut stdout = io::stdout();
    let _ = cleanup(&mut stdout);
    res
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn dir_opposites() {
        assert!(Dir::Up.is_opposite(Dir::Down));
        assert!(Dir::Down.is_opposite(Dir::Up));
        assert!(Dir::Left.is_opposite(Dir::Right));
        assert!(Dir::Right.is_opposite(Dir::Left));
        assert!(!Dir::Up.is_opposite(Dir::Left));
    }

    #[test]
    fn spawn_food_never_on_snake() {
        let mut rng = rand::rng();
        let g = Game::new(20, 20, &mut rng);
        assert!(!g.snake.contains(&g.food));
    }

    proptest! {
        #[test]
        fn step_keeps_snake_in_bounds_and_non_overlapping(width in 10i32..40, height in 10i32..30, steps in 1usize..200) {
            let mut rng = rand::rng();
            let mut g = Game::new(width, height, &mut rng);

            for _ in 0..steps {
                if !g.alive {
                    break;
                }

                // try a random direction change
                let dir = match rng.random_range(0u8..4) {
                    0 => Dir::Up,
                    1 => Dir::Down,
                    2 => Dir::Left,
                    _ => Dir::Right,
                };
                g.set_dir(dir);

                g.step(&mut rng);

                if g.alive {
                    let head = g.head();
                    prop_assert!(head.x >= 0 && head.x < g.width);
                    prop_assert!(head.y >= 0 && head.y < g.height);

                    // ensure no duplicates in snake
                    let mut v: Vec<Pos> = g.snake.iter().copied().collect();
                    v.sort_by_key(|p| (p.y, p.x));
                    v.dedup();
                    prop_assert_eq!(v.len(), g.snake.len());

                    prop_assert!(!g.snake.contains(&g.food));
                }
            }
        }
    }
}
