# Pointercrate

As of march 2nd 2019 this is the official repository for pointercrate. It contains all the backend (Rust), database (SQL) and frontend (JavaScript/CSS and html templating in form of rust macros) code that runs pointercrate.

## Running pointercrate copies

### Prereqs

- PostgreSQL (database used by pointercrate)
- pandoc     (to build the markdown documentation)

### Configuration

Pointercrate is mainly configured via optional environment variables. It will attempt to load them from an `.env` file. These are:

- `DATABASE_URL`: The URL to the postgres database to connect to, including authentication
- `GDCF_URL`: The URL to the postgres database for GDCF to connect to
- `PORT`: The port to run on (defaults to `8088`)
- `DOCUMENTATION`: The directory with the compiled documentation html files (defaults to `env!("OUT_DIR")`)

Additionally, you'll need a `.secret` file containing the secret to sign access tokens with.

### Getting it running

Even though pointercrate no longer uses diesel as it's database driver, it still uses diesel's migration system. To get a database instance running, run `diesel migration run`. 

Since pointercrate uses `sqlx`, compilation requires you to be running a postgres database with the pointercrate schema. This is because `sqlx` validates all SQL queries at compile time (syntactically _and_ semantically) by sending them over to a locally running postgres server.
 
### Disclaimer:

While I'm generally OK (in fact, its pretty awesome) with people running their own copies of this code on their own servers (note: running the code. I'm not OK with people copying the content from pointercrate), doing so is **completely unsupported** from my side beyond these instructions. If you have enough knowledge to be capable to run a server, I fully believe in you to be able to figure out how to get it running from these. 

Furthermore, if you _do_ run a pointercrate copy, I do ask you to remove all references to pointercrate from it and state somewhere that you are not associated with pointercrate in any way or form. I'd appreciate some credit, but it's not required.

I am under no obligation to keep any potential modifications you have made to the code base from working if you decide to merge in upstream changes. I also advice you to maintain a fork of GDCF as well since I _will_ be introducing breaking changes there quite often.

## Special thanks

The following people have helped with development of pointercrate, either through code contributions or other things:

- cos8o: Reverse engineered parts of the Geometry Dash source that allows pointercrate to display accurate object counts and level lengths
- GunnerBones and PoisoN: Development of various discord bots integrating with the pointercrate API
- Aquatias and Deltablu: My trusty admins that click checkboxes for me (love you guys)
- and of course the developers of all the dependencies pointercrate uses
