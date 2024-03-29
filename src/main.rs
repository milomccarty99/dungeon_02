use std::env;
use std::fs;
use std::thread;
use std::time;
use std::cmp;
use std::time::Duration;
use std::io::{stdin, stdout, Write};
use std::collections::LinkedList;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::color;
use rand::prelude::*;

//use console::Term;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const NUMB: usize = 8;

struct Env {
    map: [char; WIDTH*HEIGHT], 
    enemy_hoard: LinkedList<Enemy>,
    x_offset: isize,
    y_offset: isize,
    x_pos: isize,
    y_pos: isize,
    health: isize,
    au: isize,
}
impl Env {
    pub fn new(room: String) -> Self {
        let data = room.split("\n").collect::<Vec<_>>();
        let w: usize = data[0].parse::<usize>().expect("integer expected");
        let h: usize = data[1].parse::<usize>().expect("integer expected");
        //println!("width of {w} and height of {h}");
        let mut this = Env {
            map: ['.'; WIDTH * HEIGHT],
            enemy_hoard: LinkedList::new(),
            x_offset: 5,
            y_offset: 5,
            x_pos: data[2].parse::<isize>().expect("integer expected for x pos"),
            y_pos: data[3].parse::<isize>().expect("integer expected for y pos"),
            health: 100,
            au: 10,
        };
        for i in 0..h {
            for j in 0..w {
                this.map[i * w + j] = data[i + 4].chars().nth(j).unwrap();
            }
        }
        let mut goblinq: Enemy = Enemy::new();
        this.enemy_hoard.push_back(goblinq);

        this // returns this Env
   }


    fn print_board(&self) {
        //println!("hello"); // println ends with a newline which causes the cursor to go down
        unsafe {
            write!(stdout(), "{}", termion::cursor::Goto(self.x_offset.try_into().unwrap(), self.y_offset.try_into().unwrap()));
        }
        for i in 0..HEIGHT {
            for j in 0..WIDTH {
                write!(stdout(),"{}{}", color::Bg(color::Reset), color::Fg(color::Reset));
                let mut enemy_occupied: bool = false;
                let mut e_h_iter = self.enemy_hoard.iter();
                let mut en = e_h_iter.next();
                while !en.is_none() && !enemy_occupied
                {
                    if en.unwrap().xy_pos() == (i, j) {
                        enemy_occupied = true;
                        let fg = color::Fg(color::Green);
                        write!(stdout(), "{}g", fg);
                        break;
                    }
                    en = e_h_iter.next();
                }
                if enemy_occupied {
                    continue;
                }

                if (i == self.y_pos.try_into().unwrap() && j == self.x_pos.try_into().unwrap()) 
                {
                    let bg = color::Bg(color::Red);
                    write!(stdout(), "{}@",bg);
                }
                else
                {
                    let current_char = self.map[i*WIDTH + j];
                    match current_char {
                        '1' => write!(stdout(), "{}",color::Bg(color::Green)),
                        '2' => write!(stdout(), "{}", color::Bg(color::Yellow)),
                        '3' => write!(stdout(), "{}", color::Bg(color::Red)),
                        'x' => write!(stdout(), "{}.", color::Bg(color::Reset)),
                        '0' => write!(stdout(), " "),
                        '$' => write!(stdout(), "{}$", color::Fg(color::Yellow)),
                        _ => write!(stdout(), "{}", self.map[i*WIDTH + j]),
                    };
                   //write!(stdout(), "{}", self.map[i*WIDTH + j]);
                }
            }
            write!(stdout(), "{}{}", termion::cursor::Left(WIDTH.try_into().unwrap()), termion::cursor::Down(1));
        }
    
    }
    fn move_player(&mut self, dir: u8) {
        let mut x_mov = self.x_pos;
        let mut y_mov = self.y_pos;
        match dir {
            0 => { x_mov -= 1},
            1 => { y_mov += 1},
            2 => { y_mov -= 1},
            3 => { x_mov += 1},
            _ => (),
        }
        // unsafe due to accessing a globale static mutable
        x_mov = cmp::max(x_mov, 0);
        y_mov = cmp::max(y_mov, 0);
        x_mov = cmp::min((WIDTH-1).try_into().unwrap(), x_mov);
        y_mov = cmp::min((HEIGHT-1).try_into().unwrap(), y_mov);
        let c_mov: char = self.map[(y_mov * WIDTH as isize + x_mov) as usize];
        if c_mov != '0' { 
            self.y_pos = y_mov;
            self.x_pos = x_mov;
        }
        match c_mov {
            '$' => {
                self.au += 1;
                self.map[(self.y_pos * WIDTH as isize + self.x_pos) as usize] = 'x';
            },
            _ => (),
        }
        print!("{}, {} total gold: {}, health: {}, number of enemies: {}", self.x_pos, self.y_pos, self.au, self.health,self.enemy_hoard.len());
    }

    pub fn next_cycle(&mut self) {
        //update enemy_hoard
        let mut new_enemy_hoard: LinkedList<Enemy> = LinkedList::new();
        while !self.enemy_hoard.is_empty() {
            let mut current_enemy: Enemy = self.enemy_hoard.pop_front().unwrap();
            if !current_enemy.is_dead() {
                let (mut en_x_mov, mut en_y_mov, mut en_atks) = current_enemy.next_pos();

                let tile_move_to: char = self.map[en_y_mov * WIDTH  + en_x_mov];
                match tile_move_to {
                    '0' => (), // out of bounds
                    'x' => {
                        current_enemy.move_to(en_x_mov, en_y_mov);
                    },
                    _ => {},
                }
                if en_atks {

                    self.health -= current_enemy.damage; // take attack damage 
                }
                new_enemy_hoard.push_back(current_enemy);
            }
        }
        self.enemy_hoard = new_enemy_hoard;
    }

}

struct Enemy {
    name: String,
    creature_type: String,
    m_b: MovementBehavior,
    health: isize,
    damage: isize,
    x_pos: usize,
    y_pos: usize,
}

impl Enemy {
    pub fn new() -> Self {
        let mut this = Enemy {
            name: "qwerty".to_string(),
            creature_type: "goblin".to_string(),
            m_b: MovementBehavior::new(),
            health: 10,
            damage: 3,
            x_pos: 25,
            y_pos: 5,
        };


        this
    }
    pub fn is_dead(&mut self) -> bool {
        return self.health < 0;
    }
    
    pub fn next_pos(&mut self) -> (usize, usize, bool) {
       self.m_b.next(self.x_pos, self.y_pos, true)
    }

    pub fn move_to(&mut self, x_pos: usize, y_pos: usize) {
        self.x_pos = x_pos;
        self.y_pos = y_pos;
    }
    
    pub fn xy_pos(&self) -> (usize, usize) {
        (self.x_pos, self.y_pos)
    }

    pub fn take_damage(&mut self, amount: isize) {
        self.health += amount;
    }
    
    pub fn attack(&mut self) {

    }
}

struct MovementBehavior {
    pattern: String,
    attack_rate: usize,
    clock: usize,
}

impl MovementBehavior {
    pub fn new() -> Self {
        let mut this = MovementBehavior {
            pattern: "jj..kk..ll..hh..1".to_string(),
            attack_rate: 3,
            clock: 0,
        };

        this
    }
    
    pub fn next(&mut self, x_pos: usize, y_pos: usize, next_to_player: bool) -> (usize, usize, bool) {
        self.clock += 1;
        let mut mv: char = self.pattern.chars().nth(self.clock % self.pattern.len()).unwrap();
        let (mut x_mov, mut y_mov) = (x_pos, y_pos);
        let mut attacks: bool = false;
        match mv {
            'h' => { 
                x_mov -= 1;
            },
            'j' => {
                y_mov += 1;
            },
            'k' => {
                y_mov -= 1;
            },
            'l' => {
                x_mov += 1;
            },
            _ => {}
        }
        (x_mov, y_mov, attacks)
    }

}

//static mut env: Env = Env { map: ['x';WIDTH*HEIGHT], x_offset: 5, y_offset: 5, x_pos: 1, y_pos: 1 };
fn main() {
    stdout().flush().unwrap();
    let splash = fs::read_to_string("src/splash.txt").expect("File not found");
    println!("{}{}",termion::clear::All, splash);
    let room1: String = fs::read_to_string("src/room1.txt").expect("File not found");
    let goblinq: Enemy = Enemy::new();
    let stdin = stdin();
    //setting up stdout and going into raw mode
    let mut stdout = stdout().into_raw_mode().unwrap(); // I need to get into raw mode, but it is
    stdout.flush().unwrap();

    let mut env = Env::new(room1);
    //detecting keydown events
    'game_loop: for c in stdin.keys() {
        stdout.flush().unwrap();
        match c.unwrap() {
            Key::Ctrl('h') => println!("Hello world"),
            Key::Ctrl('q') => break,
            //Key::Alt('t') => println!("termion is cool"),
            Key::Char('q') => { quit_game(); break},
            Key::Char('f') => {flag_map();},
            Key::Char('g') => {dig_map();},
            Key::Char('h') => {env.move_player(0);},
            Key::Char('j') => {env.move_player(1);},
            Key::Char('k') => {env.move_player(2);},
            Key::Char('l') => {env.move_player(3);},
            _ => (),
        }
        env.next_cycle();
        env.print_board();
        stdout.flush().unwrap();
    }
    println!("Exiting game");
    thread::sleep(time::Duration::from_millis(500));
    write!(stdout,"{}{}", termion::cursor::Goto(1,1), termion::clear::All);
}

fn quit_game() {
    //break 'game_loop;
}

fn flag_map() {
    // flag the x_pos y_poso
}

fn check_win() -> bool {
   return false;
}

fn dig_map() -> bool {
    return false;
}

fn dig_map_loc(x: isize, y: isize) -> bool {
    return false;
}

fn check_bombs(x: isize, y: isize) -> isize{
    return 0;
}

// fn get_board() -> String {
//     let mut board: String = "".to_string();
//     for i in 0..HEIGHT {
//         for j in 0..WIDTH {
//             unsafe {
//                 board = format!("{}{}",board,  env.map[i*WIDTH + j].to_string());
//             }
//         }
//         board = board + "\n";
//     }
//     return board;
// }

