# Prototype asset provenance

| Asset | Origin | Distribution |
| --- | --- | --- |
| Cyberpunk buildings, roofs, equipment, terrain, and streets | Original procedural geometry in `src/prototype/scene.ts` | Included as source. No extracted game assets. |
| Solo Leveling-inspired citadel, chambers, spires, and rune courts | Original procedural geometry in `src/prototype/scene.ts` | Included as source. No extracted game assets. |
| Articulated cyberpunk residents and shadow-armored residents | Original meshes and poses in `src/prototype/characters.ts` | Included as source. These are role-specific original characters. |
| Resident portraits | Original SVG in `src/prototype/main.ts`, with silhouettes and equipment matched to the models | Included as source. |
| Interface icons and favicon | Original SVG path definitions | Included as source. |
| Rajdhani, weights 500 and 700 | Google Fonts, `google/fonts/ofl/rajdhani` | SIL Open Font License in `src/prototype/fonts/Rajdhani-OFL.txt`. |
| Cinzel, variable weights 400 through 700 | Google Fonts, `google/fonts/ofl/cinzel` | SIL Open Font License in `src/prototype/fonts/Cinzel-OFL.txt`. |
| Cormorant Garamond, variable weights 400 through 700 | Google Fonts, `google/fonts/ofl/cormorantgaramond` | SIL Open Font License in `src/prototype/fonts/Cormorant-Garamond-OFL.txt`. |
| Three.js | Installed version 0.186.1 | MIT license distributed with the package. |
| Supplied reference images 01 through 12 | Owner's handoff archive | Inspected for composition and behavior. No supplied screenshots or extracted game art ship in the app. |

The previous 2.5D experiment remains in `src/prototype/scene-2d.ts` for source comparison. It is not imported by the current frontend. The first prototype is also preserved in Git commit `2ea2d26`.

Fonts are local WOFF2 files. Sign textures are small canvases drawn by the application. There are no downloaded building textures, character models, or runtime font-service requests.
