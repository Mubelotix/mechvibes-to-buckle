# Mechvibes to bucklespring

This is a simple tool to download keyboard sounds from [mechvibes](https://mechvibes.com) and convert them to a [bucklespring](https://github.com/zevv/bucklespring) wav library.

## Usage

Generate bucklespring wav libraries (optional as they are already included in the repository):
```bash
cargo run
```

Use the generated folder with bucklespring:
```bash
buckle -p mechvibes-to-buckle/packs/sound-pack-1200000000012 &
```

## Notes

- Stereo works great
- Not all mechvibes sound packs are supported
- Mechvibes soundpacks only have downstroke sounds, no upstroke sounds
