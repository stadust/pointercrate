# Pointercrate ![build](https://github.com/stadust/pointercrate/actions/workflows/test.yml/badge.svg) [![codecov](https://codecov.io/gh/stadust/pointercrate/branch/master/graph/badge.svg?token=C7B5LU2IF5)](https://codecov.io/gh/stadust/pointercrate)


As of March 2nd 2019 this is the official repository for pointercrate. It contains the main parts of the backend code of [pointercrate.com](https://pointercrate.com). Specifically, it contains all the code for the demonlist and user area pages seen on pointercrate, but does not contain the code for the home page, API documentation and demonlist guidelines. It instead aims to be a framework that can be used as a stepping stone for creating custom pointercrate-like websites. The reason the home page and similar are not open source is that we have experienced people not customizing these parts when hosting their own lists, resulting in these websites displaying pointercrate branding despite not being associated with pointercrate. As a compromise, this repository instead contains code for an example binary that shows how to use the various library components in this repository to create a demonlist website. See the [getting started section](#getting-started) below for more information.

Note that exclusion of pointercrate-specific code from this repository is still a work-in-progress. For example, a lot of SEO related metadata included in the pages served for the demonlist still hardcodes the pointercrate.com URL. If you end up using this repository as a base for your own demonlist, we ask you to please update these.

## Getting Started (Linux)

The goal of this section is to compile and successful run the `pointercrate-example` binary target to set up a locally running demonlist-clone. 

We assume that you have a rust toolchain set up. If not, install the latest stable one using [`rustup`](https://rustup.rs):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

### Database Setup

Pointercrate uses [`postgresql`](https://www.postgresql.org/) as its database. This guide assumes that you have a local postgres server running, and created both a role (user) and database for use with pointercrate. For simplicity, name both of these `pointercrate` (e.g. `psql -U pointercrate pointercrate` should drop you into a database prompt).

To connect to the postgres database, pointercrate uses [`sqlx`](https://github.com/launchbadge/sqlx). This means that even to just compile pointercrate, a database with pointercrate's database schema needs to be available (as sqlx will validate SQL queries both syntactically and semantically at compile-time by sending them to a database server for validation). For this, the `DATABASE_URL` environment variable needs to be set to a [libpq connection string](https://www.postgresql.org/docs/current/libpq-connect.html#LIBPQ-CONNSTRING), e.g. 

```bash
# If you created the pointercrate database and the pointercrate role (without password but with login permissions),
# the connection string will be
export DATABASE_URL=postgresql://pointercrate@localhost/pointercrate
```

To set up a database with all the tables used by pointercrate, this repository contains a set of migrations that can be applied to an empty database (in the [`migrations`](migrations) directory). To apply the migrations, run

```bash
# you might need your distrbutions equivalent of the libssl-dev and gcc packages
cargo install sqlx-cli --no-default-features --features native-tls,postgres
cargo sqlx migrate run
```

### Pointercrate Configuration

Pointercrate is configured via environment variables, which it reads from a `.env` file in the working directory. Additionally, it expects a secret for signing access tokens to be available in a `.secret` file. An example `.env` can be found under `pointercrate-example`. Copy this to the repository root and create the a dummy `.secret` file (for debug purposes only!) via

```bash
cp pointercrate-example/.env.sample .env
echo "insecure-do-not-use-in-prod" > .secret
```

Then, open `.env` and fill out all the fields that do not have default values (e.g. `DATABASE_URL`).

### Running `pointercrate-example`

At this point, you should be able to run `pointercrate-example` via

```bash
cargo run -p pointercrate-example
```

If everything is set up correctly, you should see `rocket`'s development server start up with output as follows:

<details>

```
    Finished dev [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/pointercrate-example`
🔧 Configured for debug.
   >> address: 127.0.0.1
   >> port: 1971
   >> workers: 12
   >> max blocking threads: 512
   >> ident: Rocket
   >> IP header: X-Real-IP
   >> limits: bytes = 8KiB, data-form = 2MiB, file = 1MiB, form = 32KiB, json = 1MiB, msgpack = 1MiB, string = 8KiB
   >> temp dir: /tmp
   >> http/2: true
   >> keep-alive: 5s
   >> tls: disabled
   >> shutdown: ctrlc = true, force = true, signals = [SIGTERM], grace = 2s, mercy = 3s
   >> log level: normal
   >> cli colors: true
📬 Routes:
   >> (home) GET /
   >> (login_page) GET /login
   >> (login) POST /login
   >> (account_page) GET /account
   >> (register) POST /register
   >> (overview) GET /demonlist/?<timemachine>&<submitter>
   >> (stats_viewer_redirect) GET /demonlist/?statsviewer=true
   >> (demon_page) GET /demonlist/<position>
   >> (stats_viewer) GET /demonlist/statsviewer
   >> (nation_stats_viewer) GET /demonlist/statsviewer/nations
   >> (demon_permalink) GET /demonlist/permalink/<demon_id>
   >> (heatmap_css) GET /demonlist/statsviewer/heatmap.css
   >> (FileServer: pointercrate-core-pages/static) GET /static/core/<path..> [10]
   >> (FileServer: pointercrate-user-pages/static) GET /static/user/<path..> [10]
   >> (FileServer: pointercrate-demonlist-pages/static) GET /static/demonlist/<path..> [10]
   >> (login) POST /api/v1/auth/
   >> (get_me) GET /api/v1/auth/me
   >> (patch_me) PATCH /api/v1/auth/me
   >> (delete_me) DELETE /api/v1/auth/me
   >> (register) POST /api/v1/auth/register
   >> (invalidate) POST /api/v1/auth/invalidate
   >> (verify_email) GET /api/v1/auth/verify_email?<token>
   >> (paginate) GET /api/v1/users/
   >> (get_user) GET /api/v1/users/<user_id>
   >> (patch_user) PATCH /api/v1/users/<user_id>
   >> (delete_user) DELETE /api/v1/users/<user_id>
   >> (paginate) GET /api/v2/demons/
   >> (post) POST /api/v2/demons/
   >> (paginate_listed) GET /api/v2/demons/listed
   >> (get) GET /api/v2/demons/<demon_id>
   >> (patch) PATCH /api/v2/demons/<demon_id>
   >> (audit) GET /api/v2/demons/<demon_id>/audit
   >> (post_creator) POST /api/v2/demons/<demon_id>/creators
   >> (movement_log) GET /api/v2/demons/<demon_id>/audit/movement
   >> (delete_creator) DELETE /api/v2/demons/<demon_id>/creators/<player_id>
   >> (paginate) GET /api/v1/records/
   >> (unauthed_pagination) GET /api/v1/records/
   >> (submit) POST /api/v1/records/
   >> (paginate) GET /api/v1/players/
   >> (paginate_claims) GET /api/v1/players/claims
   >> (ranking) GET /api/v1/players/ranking
   >> (delete) DELETE /api/v1/records/<record_id>
   >> (get) GET /api/v1/records/<record_id>
   >> (patch) PATCH /api/v1/records/<record_id>
   >> (get) GET /api/v1/players/<player_id>
   >> (patch) PATCH /api/v1/players/<player_id>
   >> (get_notes) GET /api/v1/records/<record_id>/notes
   >> (add_note) POST /api/v1/records/<record_id>/notes
   >> (audit) GET /api/v1/records/<record_id>/audit
   >> (put_claim) PUT /api/v1/players/<player_id>/claims
   >> (geolocate_nationality) POST /api/v1/players/<player_id>/geolocate
   >> (delete_note) DELETE /api/v1/records/<record_id>/notes/<note_id>
   >> (patch_note) PATCH /api/v1/records/<record_id>/notes/<note_id>
   >> (patch_claim) PATCH /api/v1/players/<player_id>/claims/<user_id>
   >> (delete_claim) DELETE /api/v1/players/<player_id>/claims/<user_id>
   >> (paginate) GET /api/v1/submitters/
   >> (get) GET /api/v1/submitters/<submitter_id>
   >> (patch) PATCH /api/v1/submitters/<submitter_id>
   >> (ranking) GET /api/v1/nationalities/ranking
   >> (nation) GET /api/v1/nationalities/<iso_code>
   >> (subdivisions) GET /api/v1/nationalities/<iso_code>/subdivisions
   >> (list_information) GET /api/v1/list_information/
🥅 Catchers:
   >> (catch_404) 404
📡 Fairings:
   >> Shield (liftoff, response, singleton)
   >> Maintenance (ignite, request)
🛡️ Shield:
   >> X-Content-Type-Options: nosniff
   >> Permissions-Policy: interest-cohort=()
   >> X-Frame-Options: SAMEORIGIN
🚀 Rocket has launched from http://127.0.0.1:1971
```

</details>

The last line will tell you the URL for accessing your local pointercrate instance (in this case, `127.0.0.1:1971`, since my `ROCKET_PORT` environment variable was set to `1971`)!

### Next Steps

If you want to use pointercrate as a framework for setting up your own demonlist-like website, check out the actual sample code contained in [`pointercrate-example/src/main.rs`](pointercrate-example/src/main.rs). As a first step, you will probably want to replace all the placeholder strings (such as replacing `"<your website>"` with your domain). You probably also want to the "Hello World" home page with a proper home page of your own, and familiarize yourself with the demonlist administration interface in the "User Area". For the latter, you will need to create an account (via the usual registration routine), and then grant yourself (list) administrator permissions via the postgres shell:

```
$ psql -U pointercrate pointercrate
psql (16.1)
Type "help" for help.

pointercrate=# -- assuming the user you just created was assigned member_id 1:
pointercrate=# UPDATE members SET permissions = '0100000000001000'::BIT(16) WHERE member_id = 1;  
```

After reloading the user area, you should be able to see all administration tabs (both for website management and demonlist management).

## Running Integration Tests

Pointercrate's test suite can be executed via `cargo test` in the repository root. As running the example binary, it requires access to a database with the pointercrate scheme loaded via the `DATABASE_URL` environment variable. You should use a separate database for tests (say, `pointercrate_test`), as during setup and tear-down of each individual test, this database is dropped and recreated from scratch. 

Integration tests are also ran as part of the CI on each pull request.

## Special thanks

The following people have helped with development of pointercrate, either through code contributions or other things:

- [cos8o](https://github.com/cos8o): Reverse engineered parts of the Geometry Dash source that allows pointercrate to display accurate object counts and level lengths
- [zmx](https://github.com/kyurime) and [mgostIH](https://github.com/mgostIH) and everyone else over in my discord server  
- [Nimbus](https://github.com/NimbusGD): Development of various discord bots integrating with the pointercrate API
- Aquatias, Deltablu and Moosh: My trusty admins that click checkboxes for me (love you guys)
- rSteel, zMarc, Stiluetto and Zipi: My beloved staff
- and of course the developers of all the dependencies pointercrate uses
