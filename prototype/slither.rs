use pipes::{stream, Chan};
use task::{spawn_supervised};

use board::{Board};
use solver::{solve};
use print::{print_board};

fn main() {
    
    let (chan, port) = stream();
    {
        let board = Board::from_stream(io::stdin());
        do spawn_supervised |move chan, move board| {
            solve::<Chan<~Board>>(&chan, ~board.clone());
        }
    }

    loop {
        match port.try_recv() {
            Some(answer) => {
                let mut answer = answer;
                print_board(io::stdout(), answer);
            }
            None => break
        }
    }
    io::println("completed");
}
