# parton

Run Windows executables in Steam Proton environments.

## List all installed games
```sh
$ parton ls
1245620 ELDEN RING      default
1888160 ARMORED CORE VI FIRES OF RUBICON      proton_experimental
2050650 Resident Evil 4 default
2622380 ELDEN RING NIGHTREIGN   default
```

## List all users
```sh
$ parton users
59710912        Dark Empath Amogus
1516607315      zilfeejel
```

## List currently running games
```sh
$ parton ps
1245620 ELDEN RING
```

## Run an installer
```sh
$ parton run 1245620 ~/Downloads/CheatEngine76.exe
```

## Start something alongside a running game
```sh
$ parton attach 1245620 "/games/steam/steamapps/compatdata/1245620/pfx/drive_c/Program Files/Cheat Engine/Cheat Engine.exe"
```

### Hack to bypass gamescope
```sh
$ parton attach --bypass-gamescope 1245620 "/games/steam/steamapps/compatdata/1245620/pfx/drive_c/Program Files/Cheat Engine/Cheat Engine.exe"
```

## Launch another executable while preserving Steam launch options
```sh
$ parton launch 1245620 --user-id 59710912 "Game/ersc_launcher.exe"
```
