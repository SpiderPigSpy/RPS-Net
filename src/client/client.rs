use rps::{Player};
use rps::moves::Move;

use std::sync::{Arc, Mutex};
use std::io::{Error};
use std::ops::Drop;

use game_state::GameState;
use super::inner::Inner;

pub struct Client {
    inner: Arc<Mutex<Inner>>,
}

impl Client {
    
    ///Tries to connect to the server, blocks the execution while connecting
    pub fn new(adress: &str) -> Result<Client, Error>{
        let inner = Arc::new(Mutex::new( try!(Inner::connect(adress)) ));
        
        let spawn_inner = inner.clone();
        ::std::thread::spawn(move || {
            loop {
                {
                    let mut inner = spawn_inner.lock().unwrap();
                    if inner.is_shot_down() { break; }
                    inner.one_cycle();
                }
                ::std::thread::sleep_ms(100);
            }        
        });
        
        Ok(Client {
            inner: inner,
        })
    }
    
    ///Returns true is any game is currently running
    pub fn game_in_progress(&self) -> bool {
        self.inner.lock().unwrap().game_in_progress()
    }
    
    ///Abandons current game and sends request for a new game
    pub fn join_new_game(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.join();
    }
    
    ///Returns current game state,
    ///None if client didn't connect or didn't join a game yet
    pub fn game_state(&self) -> Option<GameState> {
        let inner = self.inner.lock().unwrap();
        
        inner.game_state()
    }
    
    ///Returns current player's color,
    ///None if client didn't connect or didn't join a game yet
    pub fn player_color(&self) -> Option<Player> {
        let inner = self.inner.lock().unwrap();
        if inner.game_in_progress() {
            Some(inner.pov())
        } else {
            None
        }
    }
    
    ///Sends move to server
    ///returns false if it isn't the players turn, 
    ///or the game was over, 
    ///or the move was already sent this turn and it is unknown if it was valid or not
    pub fn send_move(&self, movement: Move) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.can_send_move() {
            inner.send_move(movement);
            true
        } else {
            false
        }
    }
    
    ///Whether does or not server waits for your turn
    pub fn can_send_move(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.can_send_move()
    }
    
    ///Sends some dumb move to server
    ///returns false if it isn't the players turn, 
    ///or the game was over, 
    ///or the turn was already sent this turn and it is unknown if it was valid or not
    pub fn send_dumb_move(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.can_send_move() {
            let possible_moves = inner.possible_moves();
            let move_index: usize = ::rand::random::<usize>() % possible_moves.len();
            let mov = possible_moves[move_index];
            inner.send_move(mov);
            true
        } else {
            false
        }
    }
    
    ///Shuts down tcp stream, this struct is no longer usable after calling this function
    pub fn shut_down(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.shut_down();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        debug!("Client dropped");
        let mut inner = self.inner.lock().unwrap();
        inner.shut_down();
    }
}
