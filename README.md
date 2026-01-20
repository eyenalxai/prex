# pfx

Run Windows executables in Steam Proton environments.

### List all installed games
```sh
pfx ls
1245620 ELDEN RING      default
1888160 ARMORED CORE VI FIRES OF RUBICON      proton_experimental
2050650 Resident Evil 4 default
2622380 ELDEN RING NIGHTREIGN   default
```

### List all users
```sh
pfx users
59710912        Dark Empath Amogus
1516607315      zilfeejel
```

### List currently running games
```sh
pfx ps
1245620 ELDEN RING
```

### Run an installer
```sh
pfx run 2050650 ~/Downloads/Fluffy\ Mod\ Manager-818-3-068-1765672670/Modmanager.exe
```

### Start something alongside a running game
```sh
pfx attach 1245620 "/games/steam/steamapps/compatdata/1245620/pfx/drive_c/Program Files/Cheat Engine/Cheat Engine.exe"
```

#### Hack to bypass gamescope
```sh
pfx attach --bypass-gamescope 1245620 "/games/steam/steamapps/compatdata/1245620/pfx/drive_c/Program Files/Cheat Engine/Cheat Engine.exe"
```

### Launch another executable while preserving Steam launch options

Keeps any launch options you set in Steam, for example:
```sh
LD_PRELOAD= gamescope -f -H 1440 -h 1440 -r 75 --mangoapp -- env LD_PRELOAD="$LD_PRELOAD" gamemoderun %command%
```

#### Seamless Co-op for Elden Ring example
```sh
pfx launch 1245620 --user-id 59710912 "Game/ersc_launcher.exe" 
```
