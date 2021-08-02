# Pointercrate

As of march 2nd 2019 this is the official repository for pointercrate. It contains all the backend (Rust), database (SQL) and frontend (JavaScript/CSS and html templating in form of rust macros) code that runs pointercrate.

## Running pointercrate copies

### Prereqs

- PostgreSQL (database used by pointercrate)
- pandoc     (to build the markdown documentation and the guidelines)
- diesel-cli (install via `cargo install diesel_cli`. Install with only the postgres feature enabled and make sure you have the postgres library installed to link against [`cargo install diesel_cli --no-default-features --features postgres`])
- libssl-dev (one of the prereqs for diesel_cli)
- libpq-dev (one of the prereqs for diesel_cli)

### Configuration

Pointercrate is mainly configured via optional environment variables. It will attempt to load them from an `.env` file. These are:

- `DATABASE_URL`: The URL to the postgres database to connect to, including authentication
- `PORT`: The port to run on (defaults to `8088`)
- `DOCUMENTATION`: The directory with the compiled documentation html files (defaults to `env!("OUT_DIR")/documentation`)
- `GUIDELINES`: The directory with the compiled guidelines html files (defaults to `env!("OUT_DIR")/guidelines`)
- `LIST_SIZE`: Size of the main list (defaults to 50)
- `EXTENDED_LIST_SIZE`: Size of the main list + extended list (defaults to 100)
- `ADSENSE_PUBLISHER_ID`: Used for displaying advertisements on the website (optional. read [disclaimer](https://github.com/stadust/pointercrate#disclaimer).)
- `ANALYTICS_TAG`: Used for utilizing google analytics on your website (optional. read [disclaimer](https://github.com/stadust/pointercrate#disclaimer).)

Additionally, you'll need a `.secret` file containing the secret to sign access tokens with (an easy way to do that is to use openssl).

### Getting it running

Even though pointercrate no longer uses diesel as its database driver, it still uses diesel's migration system. To get a database instance running, run `diesel migration run`. You might have to mess around with the initial migrations a bit to get them working because they are partially based upon the existing scheme of a very old python version of pointercrate.

Since pointercrate uses `sqlx`, compilation requires you to be running a postgres database with the pointercrate schema. This is because `sqlx` validates all SQL queries at compile time (syntactically _and_ semantically) by sending them over to a locally running postgres server.
 
### Disclaimer:

**Please remove the adsense and google analytics scripts when hosting your own copy if you will not use them!**

While I'm generally OK (in fact, its pretty awesome) with people running their own copies of this code on their own servers (note: running the code. I'm not OK with people copying the content from pointercrate), doing so is **completely unsupported** from my side beyond these instructions. If you have enough knowledge to be capable to run a server, I fully believe in you to be able to figure out how to get it running from these. Depending on what mood I'm in on any given day, I might offer support over in [my discord server](https://discord.gg/sQewUEB).

Furthermore, if you _do_ run a pointercrate copy, I do ask you to remove all references to pointercrate from it and state somewhere that you are not associated with pointercrate in any way or form. I'd appreciate some credit, but it's not required.

I am under no obligation to keep any potential modifications you have made to the code base from working if you decide to merge in upstream changes. 

## Special thanks

The following people have helped with development of pointercrate, either through code contributions or other things:

- [cos8o](https://github.com/cos8o): Reverse engineered parts of the Geometry Dash source that allows pointercrate to display accurate object counts and level lengths
- [zmx](https://github.com/kyurime) and [mgostIH](https://github.com/mgostIH) and everyone else over in my discord server  
- [Nimbus](https://github.com/NimbusGD): Development of various discord bots integrating with the pointercrate API
- Aquatias and Deltablu: My trusty admins that click checkboxes for me (love you guys)
- and of course the developers of all the dependencies pointercrate uses
