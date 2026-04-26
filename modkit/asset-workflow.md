# Asset Workflow

Asset workflows are intentionally not implemented in v1. Stellar Blade PC is assumed to use Io Store-style cooked content, so local experiments may involve `.utoc` and `.ucas` outputs as well as `.pak` metadata.

Future asset mod support should:

- use a matching UE4.26 or UE4.26.2 editor project when verified;
- isolate mod content in a content plugin or `/Game/Mods/StellarBladeBouldyRecon`;
- use chunking or packaging settings that match the target game;
- keep cooked output and extracted game content out of git.
