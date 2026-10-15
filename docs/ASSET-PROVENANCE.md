# Prototype asset provenance

| Asset | Origin | Distribution |
| --- | --- | --- |
| Cyberpunk architecture, streets, props, couriers, and lighting | Original procedural artwork in `src/prototype/scene.ts` | Included as source code. |
| Shadow architecture, rune courts, rock formations, and messengers | Original procedural artwork in `src/prototype/scene.ts` | Included as source code. |
| Resident portraits | Original inline SVG in `src/prototype/main.ts` | Included as source code. |
| Interface icons and favicon | Original SVG path definitions in `src/prototype/icons.ts` and `favicon.svg` | Included as source code. |
| DM Sans, Latin variable font | Google Fonts, `google/fonts/ofl/dmsans` | SIL Open Font License. License in `src/prototype/fonts/DM-Sans-OFL.txt`. |
| Space Grotesk, Latin variable font | Google Fonts, `google/fonts/ofl/spacegrotesk` | SIL Open Font License. License in `src/prototype/fonts/Space-Grotesk-OFL.txt`. |
| Supplied reference images 01 through 12 | User's handoff archive | Inspected for composition and behavior. No screenshots or extracted game assets are shipped. |

Fonts are stored as local WOFF2 files. The frontend makes no font-service, image-service, provider, or analytics requests.
