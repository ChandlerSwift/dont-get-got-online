# Don't Get Got! Online client

### About

I've been playing a lot of
[Don't Get Got!](https://bigpotato.com/products/dont-get-got] by Big Potato
Games in the last few weeks. It's really fun. tl;dr: you get small silly
challenges involving other players that you have to do without them figuring out
that you're doing it for a challenge. The game is a physical game and comes with
fairly flimsy plastic wallets, as well as cards with the challenges on them that
have to be folded, kept safe, and returned to the box. This isn't a great state
of affairs, so I implemented this online wallet to allow groups of people to
play Don't Get Got! using just their phones (or computers, or smartwatches).

### Deployment

This app can be run using `cargo run`. I've chosen to run it with fly.io for
easy hosting. The fly.toml file is included.

You'll need a challenges.txt file containing a newline-delimited list of
questions. These questions are best obtained by purchasing a copy of the game,
though you can certainly make up your own.

### TODO
 - Persistence (Rocket has good support for https://diesel.rs/)
 - Push notifications on mobile (https://developer.mozilla.org/en-US/docs/Web/API/notification)
 - Argument parsing for ports/files (https://github.com/clap-rs/clap)
 - Auto refreshing the page when state changes a la https://github.com/SergioBenitez/Rocket/blob/v0.5-rc/examples/chat/src/main.rs (https://developer.mozilla.org/en-US/docs/Web/API/EventSource)
 - Clean up HTML output
 - Redirect properly (/ -> play if user is logged in, etc)
 - Ensure that no two players get the same challenge
 - Prune old games
 - Helpful error messages
 - Only allow /status in dev mode
 - Selecting game and player should be guards (https://rocket.rs/v0.5-rc/guide/requests/#request-guards)
