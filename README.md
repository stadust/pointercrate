# Pointercrate ![build](https://github.com/stadust/pointercrate/actions/workflows/test.yml/badge.svg) [![codecov](https://codecov.io/gh/stadust/pointercrate/branch/master/graph/badge.svg?token=C7B5LU2IF5)](https://codecov.io/gh/stadust/pointercrate)


As of March 2nd 2019 this is the official repository for pointercrate. It does not contain all the code required to run a local copy of pointercrate, however, as parts of the code remain private. In particular, this repository does not contain

- a `main.rs` file stitching together the code in the different libraries
- various assets such as graphics used by pointercrate
- code specific to pointercrate that has no place on custom copies of pointercrate (such as the pointercrate homepage)

This has both upsides and downsides. Since you'll have to write those components yourself, it will be very complicated to run custom pointercrate copies (especially since we do not actually support such endeavours). However, there are various advantages:

- No code in this repository explicitly references pointercrate. Everything from the logo in the navigation bar to the site metadata in the headers is configurable. This means I wont have to shout at you 7 times for failing to remove references to pointercrate on your website
- Each component is as independent as possible. For instance, you could run a pointercrate copy that does not use the `pointercrate-demonlist*` libraries and it would work just fine.

## Special thanks

The following people have helped with development of pointercrate, either through code contributions or other things:

- [cos8o](https://github.com/cos8o): Reverse engineered parts of the Geometry Dash source that allows pointercrate to display accurate object counts and level lengths
- [zmx](https://github.com/kyurime) and [mgostIH](https://github.com/mgostIH) and everyone else over in my discord server  
- [Nimbus](https://github.com/NimbusGD): Development of various discord bots integrating with the pointercrate API
- Aquatias, Deltablu and Moosh: My trusty admins that click checkboxes for me (love you guys)
- rSteel, zMarc, Stiluetto and Zipi: My beloved staff
- and of course the developers of all the dependencies pointercrate uses
