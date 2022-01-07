#[macro_use]
extern crate rocket;

use rand::prelude::*;
use rocket::form::Form;
use rocket::fs::FileServer;
use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use rocket::serde::{Serialize, Serializer};
use rocket::State;
use rocket_dyn_templates::Template;
use serde::ser::SerializeStruct;
use std::collections::HashMap;
use std::{fs, iter};
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
        self.challenges
            .iter()
            .filter(|c| c.state == ChallengeState::Succeeded)
            .count()
    }
}

// Implementing this explicitly (rather than via #[derive()]) allows us to
// include score
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

#[derive(Debug, Serialize, PartialEq, Clone, FromFormField)]
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
    let player_index: usize = cookies
        .get_private("player_index")
        .unwrap()
        .value()
        .parse()
        .unwrap();
    Ok(Template::render(
        "play",
        PlayPageContext {
            player: game.players[player_index].clone(),
            game: game,
        },
    ))
}

#[post("/challenge/<challenge_index>", data = "<state>")]
fn set_challenge_state(games: &State<GameList>, cookies: &CookieJar<'_>, challenge_index: usize, state: Form<ChallengeState>) -> Flash<Redirect> {
    // TODO: I should make a guard for the game
    let join_code = cookies.get_private("game").unwrap_or(Cookie::new("", "")); // We won't find a game with this name
    let join_code = join_code.value();
    let games = games.games.read().unwrap();
    let mut game = match games.get(join_code) {
        Some(g) => g.lock().unwrap(),
        None => return Flash::error(Redirect::to(uri!(index())), "Could not find game"), // TODO: display
    };
    let player_index: usize = cookies
        .get_private("player_index")
        .unwrap()
        .value()
        .parse()
        .unwrap();
    game.players[player_index].challenges[challenge_index].state = state.clone();
    Flash::success(Redirect::to(uri!(play())), "done")
}

#[derive(Debug, FromForm)]
struct NewGame {
    name: String,
    action: String,
    join_code: String,
}

fn make_challenges(prompts: &Vec<Prompt>) -> Vec<Challenge> {
    let mut challenges = Vec::new();
    for prompt in prompts.choose_multiple(&mut rand::thread_rng(), 5) {
        challenges.push(Challenge{
            prompt: prompt.clone(),
            state: ChallengeState::Active,
            is_special_challenge: false,
        })
    }

    challenges.push(Challenge{
        prompt: String::from("Say \"Guess what?\" to another player. If they respond \"What?\", say \"You got got!\"."),
        state: ChallengeState::Active,
        is_special_challenge: true,
    });
    challenges
}

#[post("/play", data = "<new_game_form>")]
fn new(games: &State<GameList>, prompts: &State<Vec<Prompt>>, new_game_form: Form<NewGame>, jar: &CookieJar<'_>) -> Redirect {
    match new_game_form.action.as_str() {
        "join" => { // TODO: player limit
            let games = games.games.read().unwrap();
            let game_to_join = games.get(&new_game_form.join_code.to_uppercase().clone());
            match game_to_join {
                Some(game) => {
                    game.lock().unwrap().players.push(Player {
                        name: new_game_form.name.clone(),
                        challenges: make_challenges(prompts),
                    });
                    jar.add_private(Cookie::new("game", new_game_form.join_code.to_uppercase()));
                    jar.add_private(Cookie::new(
                        "player_index",
                        (game.lock().unwrap().players.len() - 1).to_string(),
                    ));
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
                        challenges: make_challenges(prompts),
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
    let raw_input = fs::read_to_string("challenges.txt").expect("Something went wrong reading the file");
    let mut prompts = Vec::new();
    for s in raw_input.trim().split("\n") {
        prompts.push(s.to_string());
    }

    let games = GameList {
        games: RwLock::new(HashMap::new()),
    };

    rocket::build()
        .mount("/", routes![index, play, new, status, set_challenge_state])
        .attach(Template::fairing())
        .manage(games)
        .manage(prompts)
        .mount("/", FileServer::from("static"))
}
