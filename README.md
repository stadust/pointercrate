# Hey!!!

This is the code for the new CSCL site. It took a lot of work (probably more than it should have) to get this up and running while hiding some sensitive info lol. Feel free to report bugs, request features, or help out with development here! If you are curious about anything, open an issue or ask me on Discord. (i probably won't know tho lol)

# How to use this respository to experiment with the site

Firstly, do NOT use this guide as a way to set up your own pointercrate list. Refer to the [official Pointercrate repo readme](https://github.com/stadust/pointercrate) instead. If you need any help with following that guide, join the [Pointercrate Central discord](https://discord.gg/sQewUEB).

# -----

To begin, start by downloading a .zip of this repo (or clone it with github desktop) and unzip it. 

Next, you need to download a few things: 

[rustup](https://rustup.rs), 

[Postgres](https://www.enterprisedb.com/downloads/postgres-postgresql-downloads)  (if prompted, set the password to "asdf". the security of this password won't matter since you're hosting this database locally, meaning only you can access it.), 

Shuttle (open command prompt and type `cargo install cargo-shuttle`), 

and sqlx (open command prompt and type `cargo install sqlx-cli --no-default-features --features native-tls,postgres`. 

# -----

Now search for the program "pgAdmin 4" which should be installed with Postgres, open it, and make sure there is a server called "Postgres" or something. If there isn't, right click on `servers > register > server > name it whatever you want`, then go to the `definition` tab, set `Host name/address` to "`localhost`", set the username to "`postgres`" and the password to "`asdf`", and press save.

Now we need to create a new role and database. You must make the role before the database. To make a role, right click on `Login/Group Roles` and create a Login/Group role. Name it "pointercratetest", and in the Definition tab make the password "asdf". Next, in the Privileges tab toggle the "Can login?" and "Superuser?" settings on and press Save. Next, right click "Databases" on the left and create a new one. Name it "pointercrate" and set the owner to the "pointercratetest" role you just created and press save. 

# -----

Next, we need to apply the database schema Pointercrate uses. You can think of a schema as the "files and folders" the site stores its data in. 

First, open the repository folder on your computer, then go to /pointercrate-example/sample/migrations/_new , select all files in the folder, and move them to the migrations folder in the root (TheClicksyncChallengeList-main/migrations/), and replace the files if prompted. 
(side note i know this sucks but i'm working on it lol....,...)

Next, open the command prompt/shell and navigate to the folder you downloaded this repository to via "cd". The command line usually drops you in your user folder (C:/Users/username), so to if the folder is on your Desktop, for example, type `cd desktop/[folder name]`. Next, run `cargo sqlx migrate run`. This should automatically apply the schema the database needs. 

The last thing to do is actually compile and run the site. To do this, run `cargo shuttle run` and wait for all the libraries to install and compile. At the end, it should give you a link (probably https://127.0.0.1:8001 ) to access the site in your browser!

# Next steps

If you need to be able to add data to the site, you need to give yourself administrator permissions. To do this, create a pointercrate account if you haven't already (user area -> register), go into pgadmin 4, right click on the "pointercrate" database and click "PSQL Tool". Then paste in `UPDATE members SET permissions = '0100000000001000'::BIT(16) WHERE member_id = 1;` and press enter, then refresh the site. You should be able to see all moderator tabs in the user area!

if you know what you're doing, i would love some help with development! i wrote most of the above text like a month ago, and i've since gotten """"""""better""""""" at coding in general, but still lol. thank you!
