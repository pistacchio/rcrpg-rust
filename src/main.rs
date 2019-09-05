use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::{io, fmt};
use std::fmt::{Display, Debug};
use std::borrow::BorrowMut;
use std::ops::Add;
use rand::prelude::*;

///////////////
// CONSTANTS //
///////////////

const DIRECTION_MAPPING: [(Location, Direction); 6] = [
    (Location(0, -1, 0), Direction::North),
    (Location(0, 1, 0), Direction::South),
    (Location(-1, 0, 0), Direction::West),
    (Location(1, 0, 0), Direction::East),
    (Location(0, 0, 1), Direction::Down),
    (Location(0, 0, -1), Direction::Up),
];

///////////
// TYPES //
///////////

type Invetory = HashSet<Object>;
type CommandAliases = Vec<(HashSet<String>, Command)>;

//////////////
// LOCATION //
//////////////

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct Location(i32, i32, i32);

impl Add for Location {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Location(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Debug for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "({}, {}, {})", self.0, self.1, self.2)
    }
}

////////////
// OBJECT //
////////////

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
enum Object {
    Ladder,
    Sledge,
    Gold,
}

impl Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Object::Ladder => write!(f, "a ladder"),
            Object::Sledge => write!(f, "a sledge"),
            Object::Gold => write!(f, "some gold"),
        }
    }
}

impl Object {
    fn from_string(s: &str) -> Option<Object> {
        match s {
            "ladder" => Some(Object::Ladder),
            "sledge" => Some(Object::Sledge),
            "gold" => Some(Object::Gold),
            _ => None
        }
    }
}

////////////
// PLAYER //
////////////

struct Player {
    location: Location,
    inventory: Invetory,
    equipped: Option<Object>,
}

//////////
// ROOM //
//////////

struct Room {
    description: Option<String>,
    objects: Invetory,
}

impl Room {
    fn new() -> Self {
        Room {
            description: None,
            objects: HashSet::new(),
        }
    }

    fn with_description(mut self, descrition: &str) -> Self {
        self.description = Some(descrition.to_string());
        self
    }

    fn with_objects(mut self, objects: Vec<Object>) -> Self {
        self.objects.extend(objects);
        self
    }

    fn with_random_objects(mut self, rng: &mut ThreadRng ) -> Self {
        let objects: Vec<_> = vec![
            if rng.gen::<f32>() < 0.33 { Some(Object::Sledge) } else { None },
            if rng.gen::<f32>() < 0.33 { Some(Object::Ladder) } else { None },
            if rng.gen::<f32>() < 0.33 { Some(Object::Gold) } else { None },
        ].iter().filter_map(|o| *o).collect();

        self.objects.extend(objects);
        self
    }
}

/////////////
// DUNGEON //
/////////////

#[derive(Copy, Clone, Eq, PartialEq)]
enum Direction {
    North,
    South,
    West,
    East,
    Down,
    Up,
}

impl Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Direction::North => write!(f, "north"),
            Direction::South => write!(f, "south"),
            Direction::West => write!(f, "west"),
            Direction::East => write!(f, "east"),
            Direction::Down => write!(f, "down"),
            Direction::Up => write!(f, "up"),
        }
    }
}

impl Direction {
    fn from_string(s: &str) -> Option<Direction> {
        match s {
            "north" => Some(Direction::North),
            "south" => Some(Direction::South),
            "west" => Some(Direction::West),
            "east" => Some(Direction::East),
            "down" => Some(Direction::Down),
            "up" => Some(Direction::Up),
            _ => None
        }
    }

    fn to_location(&self) -> Location {
        DIRECTION_MAPPING.iter()
            .find(|d| d.1 == *self)
            .unwrap()
            .0
    }
}

struct Dungeon {
    rooms: HashMap<Location, Room>
}

impl Dungeon {
    fn new() -> Self {
        Dungeon {
            rooms: HashMap::from_iter(vec![
                (Location(0, 0, 0), Room::new()
                    .with_description("The room where it all started...")
                    .with_objects(vec![Object::Ladder, Object::Sledge])),
                (Location(1, 1, 5), Room::new()
                    .with_description("You found it! Lots of gold!"))
            ])
        }
    }

    fn exits_for_room(&self, location: Location) -> Vec<Direction> {
        DIRECTION_MAPPING.iter().filter_map(|d| {
            let location_to_test = location + d.0;

            if self.rooms.contains_key(&location_to_test) {
                return Some(d.1);
            }
            None
        }).collect()
    }
}

//////////////
// COMMANDS //
//////////////

#[derive(Debug, Copy, Clone)]
enum Command {
    North,
    South,
    West,
    East,
    Down,
    Up,
    Help,
    Dig,
    Look,
    Inventory,
    Take,
    Drop,
    Equip,
    Unequip,
    Alias,
}

fn default_aliases() -> CommandAliases {
    vec![
        (vec!["n".to_string(), "north".to_string()].into_iter().collect(), Command::North),
        (vec!["s".to_string(), "south".to_string()].into_iter().collect(), Command::South),
        (vec!["w".to_string(), "west".to_string()].into_iter().collect(), Command::West),
        (vec!["e".to_string(), "east".to_string()].into_iter().collect(), Command::East),
        (vec!["d".to_string(), "down".to_string()].into_iter().collect(), Command::Down),
        (vec!["u".to_string(), "up".to_string()].into_iter().collect(), Command::Up),
        (vec!["help".to_string()].into_iter().collect(), Command::Help),
        (vec!["dig".to_string()].into_iter().collect(), Command::Dig),
        (vec!["l".to_string(), "look".to_string()].into_iter().collect(), Command::Look),
        (vec!["i".to_string(), "inventory".to_string()].into_iter().collect(), Command::Inventory),
        (vec!["take".to_string()].into_iter().collect(), Command::Take),
        (vec!["drop".to_string()].into_iter().collect(), Command::Drop),
        (vec!["equip".to_string()].into_iter().collect(), Command::Equip),
        (vec!["unequip".to_string()].into_iter().collect(), Command::Unequip),
        (vec!["alias".to_string()].into_iter().collect(), Command::Alias),
    ]
}

fn find_command(command: &str, aliases: &CommandAliases) -> Option<Command> {
    let command = command.to_lowercase();

    aliases.iter()
        .find(|a| a.0.contains(&command))
        .map(|a| a.1)
}

fn help() {
    println!("You need a sledge to dig rooms and ladders to go upwards.
Valid commands are: directions (north, south...), dig, take, drop, equip, inventory and look.
Additionally you can tag rooms with the 'name' command and alias commands with 'alias'.
Have fun!")
}

fn alias(command_aliases: &mut CommandAliases, args: &[&str]) {
    if args.len() < 2 {
        println!("To assign an alias: alias CMQ NEW_ALIAS");
    } else {
        let command = args[0].to_lowercase();
        let new_alias = args[1].to_lowercase();

        let mut found = false;
        for ca in command_aliases {
            if ca.0.contains(&command) {
                ca.0.insert(new_alias.clone());
                found = true;
            }
        }

        if found {
            println!("You can use \"{}\" in lieu of \"{}\"", new_alias, command);
        } else {
            println!("The commands \"{}\" does not exist", command);
        }
    }
}

fn look(player: &Player, dungeon: &Dungeon) {
    let room = &dungeon.rooms[&player.location];

    if let Some(description) = &room.description {
        print!("{}", description);
    } else {
        print!("Room at {:?}.", player.location);
    }


    if !room.objects.is_empty() {
        print!(" On the floor you can see: {}.", room.objects
            .iter()
            .map(|o| o.to_string())
            .collect::<Vec<String>>()
            .join(", "));
    }

    let room_exits = dungeon.exits_for_room(player.location);
    match room_exits.len() {
        0 => println!(" There are no exits in this room."),
        1 => println!(" There is one exit: {}.", room_exits[0].to_string()),
        _ => println!(" Exits: {}.", room_exits.iter()
            .map(|o| o.to_string())
            .collect::<Vec<String>>()
            .join(", "))
    }
}


fn take(player: &mut Player, dungeon: &mut Dungeon, args: &[&str]) {
    if args.is_empty() {
        println!("To take something: take OBJECT|all")
    } else if dungeon.rooms[&player.location].objects.is_empty() {
        println!("There is nothing to take here")
    } else if args[0] == "all" {
        let room_objects = dungeon.rooms.get_mut(&player.location)
            .expect("The player is in a room that should not exist!")
            .objects
            .borrow_mut();

        player.inventory.extend(room_objects.iter());
        room_objects.clear();

        println!("All items taken");
    } else if let Some(object) = Object::from_string(args[0]) {
        let room_objects = dungeon.rooms.get_mut(&player.location)
            .expect("The player is in a room that should not exist!")
            .objects
            .borrow_mut();

        if room_objects.contains(&object) {
            player.inventory.insert(object);
            room_objects.remove(&object);
            println!("Taken");
        }
    } else {
        println!("You can't see anything like that here")
    }
}

fn drop(player: &mut Player, dungeon: &mut Dungeon, args: &[&str]) {
    if args.is_empty() {
        println!("To drop something: drop OBJECT|all")
    } else if player.inventory.is_empty() {
        println!("You are not carrying anything")
    } else if args[0] == "all" {
        let room_objects = dungeon.rooms.get_mut(&player.location)
            .expect("The player is in a room that should not exist!")
            .objects
            .borrow_mut();

        room_objects.extend(player.inventory.iter());
        player.inventory.clear();

        println!("All items dropped");
    } else if let Some(object) = Object::from_string(args[0]) {
        let room_objects = dungeon.rooms.get_mut(&player.location)
            .expect("The player is in a room that should not exist!")
            .objects
            .borrow_mut();

        if player.inventory.contains(&object) {
            player.inventory.remove(&object);
            room_objects.insert(object);
            println!("Dropped");
        }
    } else {
        println!("You don't have anything like that")
    }
}

fn inventory(player: &Player) {
    if player.inventory.is_empty() {
        println!("You are not carrying anything")
    } else {
        println!("You are carrying: {}", player.inventory
            .iter()
            .map(|o| o.to_string())
            .collect::<Vec<String>>()
            .join(", "));
    }
}

#[allow(clippy::map_entry)]
fn dig(player: &Player, dungeon: &mut Dungeon, rng: &mut ThreadRng, args: &[&str]) {
    if args.is_empty() {
        println!("To dig a tunnel: dig DIRECTION");
    } else if let Some(direction) = Direction::from_string(args[0]) {
        if let Some(equipped) = player.equipped {
            if equipped == Object::Sledge {
                let target_location = player.location + direction.to_location();

                if dungeon.rooms.contains_key(&target_location) {
                    println!("There is already an exit, there!");
                }

                dungeon.rooms.entry(target_location).or_insert_with(|| {
                    println!("There is now an exit {}ward", direction);

                    Room::new().with_random_objects(rng)
                });
            } else {
                println!("You cannot dig with {}", equipped);
            }
        } else {
            println!("With your bare hands?");
        }
    } else {
        println!("That is not a direction I recognize");
    }
}

fn goto(player: &mut Player, dungeon: &Dungeon, direction: &Direction) {
    if direction == &Direction::North && !dungeon.rooms[&player.location].objects.contains(&Object::Ladder) {
        println!("You can't go upwards without a ladder!");
    } else {
        let target_location = player.location + direction.to_location();
        if !dungeon.rooms.contains_key(&target_location) {
            println!("There's no exit in that direction!");
        } else {
            player.location = target_location;
            look(player, dungeon);
        }
    }
}

fn equip(player: &mut Player, args: &[&str]) {
    if args.is_empty() {
        println!("To equip something: equip OBJECT");
    } else if let Some(object) = Object::from_string(args[0]) {
        if player.inventory.contains(&object) {
            player.equipped = Some(object);
            println!("Item equipped");
        } else {
            println!("You don't have such object");
        }
    } else {
        println!("You don't have such object");
    }
}

fn unequip(player: &mut Player) {
    if player.equipped.is_some() {
        player.equipped = None;
        println!("Unequipped");
    } else {
        println!("You are already not using anything");
    }
}

//////////
// MAIN //
//////////

fn main() {
    let mut command_aliases = default_aliases();
    let mut dungeon = Dungeon::new();
    let mut player = Player {
        location: Location(0, 0, 0),
        inventory: HashSet::from_iter(vec![Object::Sledge]),
        equipped: None,
    };
    let mut rng = rand::thread_rng();

    // init
    println!("Grab the sledge and make your way to room 1,1,5 for a non-existant prize!\n");
    help();

    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Cannot read from stdin");
        let input: &str = &input.trim().to_lowercase();

        let splitted = input.split_whitespace().collect::<Vec<&str>>();

        if !splitted.is_empty() {
            match find_command(splitted[0], &command_aliases) {
                Some(Command::Help) => help(),
                Some(Command::Alias) => alias(&mut command_aliases, &splitted[1..]),
                Some(Command::Look) => look(&player, &dungeon),
                Some(Command::Take) => take(&mut player, &mut dungeon, &splitted[1..]),
                Some(Command::Drop) => drop(&mut player, &mut dungeon, &splitted[1..]),
                Some(Command::Inventory) => inventory(&player),
                Some(Command::Dig) => dig(&player, &mut dungeon, &mut rng, &splitted[1..]),
                Some(Command::Equip) => equip(&mut player, &splitted[1..]),
                Some(Command::Unequip) => unequip(&mut player),
                Some(Command::North) => goto(&mut player, &dungeon, &Direction::North),
                Some(Command::South) => goto(&mut player, &dungeon, &Direction::South),
                Some(Command::West) => goto(&mut player, &dungeon, &Direction::West),
                Some(Command::East) => goto(&mut player, &dungeon, &Direction::East),
                Some(Command::Down) => goto(&mut player, &dungeon, &Direction::Down),
                Some(Command::Up) => goto(&mut player, &dungeon, &Direction::Up),
                _ => println!("I don't know what you mean.")
            }
        }
    }
}
