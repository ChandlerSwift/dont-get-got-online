#[macro_use]
extern crate rocket;

use rand::prelude::*;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;
use rocket::serde::{Serialize, Serializer};
use rocket::State;
use rocket_dyn_templates::Template;
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use std::iter;
use std::sync::{Mutex, RwLock};

#[derive(Clone, Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Game {
    join_code: String,
    players: Vec<Player>,
}

#[derive(Clone, Debug)]
struct Player {
    name: String,
    challenges: Vec<Challenge>,
}

impl Player {
    fn score(&self) -> usize {
        let mut score = 0;
        for challenge in &self.challenges {
            if challenge.state == ChallengeState::Succeeded {
                score += 1;
            }
        }
        score
    }
}

impl Serialize for Player {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Player", 3)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("challenges", &self.challenges)?;
        s.serialize_field("score", &self.score())?;
        s.end()
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(crate = "rocket::serde")]
struct Challenge {
    is_special_challenge: bool,
    state: ChallengeState,
    prompt: Prompt,
}

#[derive(Debug, Serialize, PartialEq, Clone)]
#[serde(crate = "rocket::serde")]
enum ChallengeState {
    Active,
    Succeeded,
    Failed,
}

impl Default for ChallengeState {
    fn default() -> Self {
        ChallengeState::Active
    }
}

type Prompt = String;

#[get("/")]
fn index() -> Template {
    // TODO: check if valid cookie and redirect
    let game = Game {
        join_code: String::from("foo"),
        players: Vec::new(),
    };
    Template::render("index", game) // TODO: I shouldn't need an empty context here
}

#[derive(Debug, Serialize)]
struct PlayPageContext {
    game: Game,
    player: Player,
}

#[get("/play")]
fn play(games: &State<GameList>, cookies: &CookieJar<'_>) -> Result<Template, Redirect> {
    // TODO: better error handling
    let join_code = cookies.get_private("game").unwrap_or(Cookie::new("", "")); // We won't find a game with this name
    let join_code = join_code.value();
    let games = games.games.read().unwrap();
    let game = match games.get(join_code) {
        Some(g) => g.lock().unwrap().clone(),
        None => return Err(Redirect::to(uri!(index()))),
    };
    let player_index: usize = cookies.get_private("player_index").unwrap().value().parse().unwrap();
    Ok(Template::render(
        "play",
        PlayPageContext {
            player: game.players[player_index].clone(),
            game: game,
        },
    ))
}

#[derive(Debug, FromForm)]
struct NewGame {
    name: String,
    action: String,
    join_code: String,
}

#[post("/play", data = "<new_game_form>")]
fn new(games: &State<GameList>, new_game_form: Form<NewGame>, jar: &CookieJar<'_>) -> Redirect {
    match new_game_form.action.as_str() {
        "join" => {
            let games = games.games.read().unwrap();
            let game_to_join = games.get(&new_game_form.join_code.to_uppercase().clone());
            match game_to_join {
                Some(game) => {
                    game.lock().unwrap().players.push(Player {
                        name: new_game_form.name.clone(),
                        challenges: Vec::new(), // TODO
                    });
                    jar.add_private(Cookie::new("game", new_game_form.join_code.to_uppercase()));
                    jar.add_private(Cookie::new("player_index", (game.lock().unwrap().players.len() - 1).to_string()));
                    Redirect::to(uri!(play()))
                }
                None => Redirect::to(uri!(index())),
            }
        }
        "create" => {
            // Honestly I don't have to write anything until I validate the game
            // with the given join code exists, but I'm not too concerned.
            let mut game_list = games.games.write().unwrap();
            let new_join_code: String = iter::repeat(())
                .map(|()| rand::thread_rng().sample(rand::distributions::Alphanumeric))
                .map(char::from)
                .take(6)
                .collect::<String>()
                .to_uppercase();
            game_list.insert(
                new_join_code.clone(),
                Mutex::new(Game {
                    join_code: new_join_code.clone(),
                    players: vec![Player {
                        name: new_game_form.name.clone(),
                        challenges: Vec::new(), // TODO;
                    }],
                }),
            );
            jar.add_private(Cookie::new("game", new_join_code));
            jar.add_private(Cookie::new("player_index", "0"));
            Redirect::to(uri!(play()))
        }
        _ => Redirect::to(uri!(index())),
    }
}

#[get("/status")]
fn status(games: &State<GameList>) -> Template {
    Template::render("status", games.inner())
}

#[derive(Debug, Serialize)]
struct GameList {
    games: RwLock<HashMap<String, Mutex<Game>>>,
}

#[launch]
fn rocket() -> _ {
    // TODO: should I use a Rocket builtin rather than std::fs for this?
    // let raw_input = fs::read_to_string("challenges.txt").expect("Something went wrong reading the file");
    // let challenges = raw_input.trim().split("\n").collect::<Vec<&str>>();

    let games = GameList {
        games: RwLock::new(HashMap::new()),
    };

    rocket::build()
        .mount("/", routes![index, play, new, status])
        .attach(Template::fairing())
        .manage(games)
        .mount("/", FileServer::from("static"))
}
