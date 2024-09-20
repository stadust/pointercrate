# These migrations will not work! Set the source directory to /pointercrate-example/sample/migrations/_new/

the reason these will not work is.. i'm an idiot!  past me decided to make changes to the database schema however i pleased without adding migrations. lol! when i open sourced the site, i was gettign errors saying the migrations were run but changed afterward. idk, but commenting out all these migrations and re-running them seemed to work. so now i just need to live with this, unless i think of a solution.

as instructed in the readme, you should run the migrations in the _new folder to get a clone of the database, and add any new ones there if youre making a pull request or something.

let me know if you have an idea on how i can fix this!