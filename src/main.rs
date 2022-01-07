#[macro_use] extern crate rocket;

use rocket::fs::FileServer;
use rocket_dyn_templates::{Template};
use rocket::serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;
// use rocket::http::CookieJar;

#[derive(Debug, Serialize)]
struct PlayPageContext {
    game: Game,
    challenges: Vec<Challenge>,
}

#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
struct Game {
    join_code: String,
    players: Vec<Player>,
}

#[derive(Debug)]
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
    fn default() -> Self { ChallengeState::Active }
}

type Prompt = String;

#[get("/")]
fn index() -> Template {
    let game = Game{
        join_code: String::from("foo"),
        players: Vec::new(),
    };
    Template::render("index", game) // TODO: I shouldn't need an empty context here
}

#[get("/play")]
fn play() -> Template {
    let game = Game{
        join_code: String::from("A1B2C3"),
        players: vec![
            Player{
                name: String::from("Chandler"),
                challenges: vec![
                    Challenge{
                        is_special_challenge: false,
                        state: ChallengeState::Active,
                        prompt: String::from("Wear an article of clothing inside out. Get another player to comment."),
                    },
                    Challenge{
                        is_special_challenge: false,
                        state: ChallengeState::Active,
                        prompt: String::from("Get another player to help you up off the ground."),
                    },
                    Challenge{
                        is_special_challenge: true,
                        state: ChallengeState::Failed,
                        prompt: String::from("Ask another player \"Guess what?\" and get them to respond with \"what\"."),
                    },
                ],
            },
            Player{
                name: String::from("Jeff"),
                challenges: vec![
                    Challenge{
                        is_special_challenge: false,
                        state: ChallengeState::Succeeded,
                        prompt: String::from("Get another player to talk to something that isn't voice activated."),
                    },
                    Challenge{
                        is_special_challenge: false,
                        state: ChallengeState::Failed,
                        prompt: String::from("Tie another player's shoes together without them noticing."),
                    },
                    Challenge{
                        is_special_challenge: true,
                        state: ChallengeState::Active,
                        prompt: String::from("Ask another player \"Guess what?\" and get them to respond with \"what\".Clone"),
                    },
                ],
            },
        ],
    };
    Template::render("play", PlayPageContext{
        challenges: game.players[0].challenges.clone(),
        game: game,
    })
}

#[launch]
fn rocket() -> _ {
    // TODO: should I use a Rocket builtin rather than std::fs for this?
    // let raw_input = fs::read_to_string("challenges.txt").expect("Something went wrong reading the file");
    // let challenges = raw_input.trim().split("\n").collect::<Vec<&str>>();

    rocket::build().mount("/", routes![index, play])
        .attach(Template::fairing())
        .mount("/", FileServer::from("static"))
}
