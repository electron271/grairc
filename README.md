# grairc
irc client for the 3ds

NOTE: currently the server is hardcoded, im changing this once i get setting storage setup so you dont have to enter it every time you launch the app

## todo
- [x] multiple channels
- [ ] private messages
- [x] setting storage for server/user config
- [x] improve UI
- [ ] switch to citro2d for hardware rendering
- [ ] tls support
- [ ] name colors
- [ ] scroll back through message history

## cia export
run `./cia.sh` to build a cia file, you will need `makerom` and `bannertool` installed and in your PATH

## docker
devkitpro is absolutely hell to deal with so ive made a dockerfile that has everything setup for you, if you want to setup devkitpro manually you can follow the instructions on https://devkitpro.org/wiki/Getting_Started (i personally run a cachyos vm specifically for 3ds development, obviously though this isnt that optimal)

run these commands to build and run the docker container, go to /grairc for the source code

remember that this project is cc0 so feel free (and i recommend) to use this dockerfile for your own 3ds projects including c projects (remove the rust parts that i commented in the dockerfile if you dont need them)

```bash
docker build -t grairc .
docker run --rm -it --network host -v $(pwd):/grairc grairc
```

## development
use this to run the app on your 3ds, itll relay all prints to your terminal. to make it send over go to the homebrew menu and press y

sometimes it says that it cant reach the 3ds just try again a few more times and if it still doesnt work try pressing b to exit the netloader and press y again

use --release because software rendering can be extremely slow when not using release mode

```bash
cargo 3ds run --server --release
```


