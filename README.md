
# Tic-Tac-Toe Game
This Rust program implements a Tic-Tac-Toe game with various player strategies, including human, random, and Monte Carlo Tree Search (MCTS) players. The game can be run multiple times, and the results of the matches are displayed, including wins for each player and draws.

## Getting Started
### Prerequisites
Ensure you have Rust installed. If not, install it using the following command:

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
### Running the Game
Clone the repository (if applicable):

```sh
git clone <repository-url>
cd <repository-directory>
```

### Build and run the project:

```sh
cargo run
```

## How to Play
The game runs 10 matches between two players, which can be either human, random, or Monte Carlo AI. The default setup is one random player against an AI using Monte Carlo with 1000 iterations.

Board:
---------
0 | 1 | 2
---------
3 | 4 | 5
---------
6 | 7 | 8
---------

### Each cell is accessible through an array-Index containing an index for each "layer".
In a normal Tic-Tac-Toe game with META_DEPTH = 1 all possible indices are:

> \[0] \[1] \[2] \[3] \[4] \[5] \[6] \[7] \[8]

In a Meta-Tic-Tac-Toe game with META_DEPTH = 2 the indices look like this:

> \[0,0] \[0,1] \[0,2] ... \[1,0] ...\[8,8]   


### Changing Players
You can switch the type of players by commenting or uncommenting the relevant lines in the main.rs file. The available player types are:

- HumanPlayer: Allows a human to input moves via the console.
- RandomPlayer: Makes random moves.
- MonteCarloSync: Uses Monte Carlo Tree Search for making moves. You can specify the number of iterations for the MCTS algorithm. The game will wait until the iterations are done.
- MonteCarloAsync: Uses Monte Carlo Tree Search for making moves. You can specify the duration for each move of the MCTS algorithm. The player also thinks, when the opponent is thinking. (thinking duration less than 100ms is unstable)
### Changing the Depth
The depth of the game (the number of nested boards) can be modified in the game.rs file. Adjust the META_DEPTH constant to your desired depth:

```rust
const META_DEPTH: usize = 1; // Change this value to increase or decrease the depth
```

### Example Output
The program will display the results of the 10 matches, showing the number of wins for each player and the number of draws:

```python
Display_Size: 9
Player X chose [0, 1]
Player O chose [1, 2]
...
Player X wins!
...
Player 1: 5 | Player 2 3 | Draws 2
```

## Code Structure
- main.rs: Contains the main function, player definitions, and game loop.
- game.rs: Contains game logic, board structures, and helper functions.

## Contact
For further information or questions, please reach out to the repository owner.

Happy coding and enjoy playing Tic-Tac-Toe!
