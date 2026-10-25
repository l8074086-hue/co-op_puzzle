# What is this?

Co-Op (unamed rn) is a local-multiplayer puzzle game that i'm developing to improve my skills and networking. 
It is also for the school fair. 

# What it uses
- Languague: I use Rust for our choice of language because it has a great ecosystem and I REALLY like `cargo`
- Renderer: I use raylib because it's just something I am very comfortable with coming from other languages.

# Installing dependencies
## NixOS
The repo contains the nix.shell required so just simply:
```nix
nix-shell
```
## Arch
```sh
sudo pacman -S cargo cmake rustc raylib clang
```
## Other Operating Systems
Download cargo, rustc and raylib with the respective instructions for your OS
# Running the game
Currently to run as server you do:
```sh
cargo run -- server
```
and for client:
```sh
cargo run -- client
```
but eventually we will make it so you can select inside the game itself