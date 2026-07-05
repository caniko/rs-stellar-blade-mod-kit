game-root := "/data/nvme0/can/games/steamapps/steamapps/common/StellarBlade"

install-ue4ss:
    nix run .#install-ue4ss -- --game-root "{{game-root}}"

install-ue4ss-to game_root:
    nix run .#install-ue4ss -- --game-root "{{game_root}}"

export-runtime:
    nix run .#export-runtime-binaries -- --game-root "{{game-root}}"

export-runtime-to game_root:
    nix run .#export-runtime-binaries -- --game-root "{{game_root}}"
