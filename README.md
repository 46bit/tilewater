# Tilewater

### A miniature city simulator.

[![Welcome to Tilewater preview image](https://r.46b.it/welcome-to-tilewater-vidprev-2.png)](https://www.youtube.com/watch?v=Z_5WOXicQbc "Welcome to Tilewater")

## Get started

Tilewater is built upon [Rust](https://www.rust-lang.org) and [Piston](http://www.piston.rs). For trivial reasons it uses Nightly Rust at the moment, so you'll need to install that first (details omitted as I'd rather wait until it is on stable.) You may also need `SDL2` libraries.

Once you have that, `git clone https://github.com/46bit/tilewater.git` this repo and then `cargo run --release` to run the game. The `--release` flag is necessary for smooth animation, as tilewater is not heavily optimised and thus performance-hungry.

## Controls

You interact with the world using a cursor. At present this starts in the top-left. Use your arrow keys to move the cursor about (at present, this may only function on macOS).

* Newly-built roads must neighbour an existing road. Build one under the cursor using your space key.
* Buildings are built with a 1-square gap between them and a road. This gap becomes an entranceway.
  * Typing `h` will build a House (green). A cim (a simulated person) will be spawned and move to live in it.
  * Typing `g` will build a General Store (purple). These are visited regularly by cims, as if to purchase groceries.
  * Typing `s` will build a Saloon (blue). These are visited regularly by cims, as if to socialise.
  * Typing `f` will built a Factory (red). In time these will employ workers.

If you get stuck, [the preview video](https://www.youtube.com/watch?v=Z_5WOXicQbc) shows examples of using all these commands.

## A pretty little example

![I rather like this street layout.](https://r.46b.it/lil-tilewater-city.png)
