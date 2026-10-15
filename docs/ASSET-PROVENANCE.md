# Prototype asset provenance

| Asset | Origin | Distribution |
| --- | --- | --- |
| Cyberpunk buildings, roofs, equipment, terrain, and streets | Original procedural geometry in `src/prototype/scene.ts` | Included as source. No extracted game assets. |
| Solo Leveling-inspired citadel, chambers, spires, and rune courts | Original procedural geometry in `src/prototype/scene.ts` | Included as source. No extracted game assets. |
| Rounded cyberpunk residents and distinct shadow silhouettes | Original meshes and poses in `src/prototype/characters.ts` | Included as source. These are original stylized interpretations with procedural faces, clothing, and role equipment. |
| Resident portraits | Original SVG in `src/prototype/main.ts`, with silhouettes and equipment matched to the models | Included as source. |
| Interface icons and favicon | Original SVG path definitions | Included as source. |
| Rajdhani, weights 500 and 700 | Google Fonts, `google/fonts/ofl/rajdhani` | SIL Open Font License in `src/prototype/fonts/Rajdhani-OFL.txt`. |
| Friend or Foe Tall BB | Owner-installed font, selected by the owner | CSS `local()` only. Commercial binary is not copied into source or build output. |
| Orbitron, variable | [Google Fonts source](https://github.com/google/fonts/tree/main/ofl/orbitron) | SIL Open Font License in `src/prototype/fonts/Orbitron-OFL.txt`. Secondary Cyberpunk metadata. |
| Abel, regular | Google Fonts, `google/fonts/ofl/abel` | SIL Open Font License in `src/prototype/fonts/Abel-OFL.txt`. Fallback when Friend or Foe Tall BB is not installed. |
| Chief, message courier, indoor desks and shelves | Original procedural meshes in scene and character sources | No extracted reference art. |
| Three.js | Installed version 0.186.1 | MIT license distributed with the package. |
| Supplied reference images 01 through 12 | Owner's handoff archive | Inspected for composition and behavior. No supplied screenshots or extracted game art ship in the app. |

The previous 2.5D experiment remains in `src/prototype/scene-2d.ts` for source comparison. It is not imported by the current frontend. The first prototype is also preserved in Git commit `2ea2d26`.

Open fonts are bundled WOFF2 and TTF files. Friend or Foe Tall BB uses the installed local face with an Abel fallback. Sign textures are small canvases drawn by the application. There are no downloaded building textures, character models, or runtime font-service requests.

The owner supplied further Hamlet scenes and Solo Leveling menu screenshots on 2026-10-15. They guide inhabitance, Chief interactions, narrow sans typography, translucent windows, and blue framing. The screenshots do not ship in the application. Earlier serif font experiments remain in Git history.

The owner also supplied eight Cyberpunk menu, city-street, circuit-frame, and apartment references. Revision 4 uses their red/cyan menu hierarchy, angular controls, condensed lettering, hexagonal skill composition, layered streets, utility details, neon signage, and lantern lighting as visual guidance. The implementation is CSS, original procedural geometry, and application-drawn sign textures. No reference screenshots or extracted game assets ship in the app. Rajdhani is retained as the locally licensed approximation for this menu direction.

Revision 5 uses four further Solo Leveling reference images for notification framing, the separate title icon, silver inner borders, blue outer rails, equipment slots, and the white outline figure. All new window rails and figure paths are original SVG/CSS. The Chief's isolated estate is original procedural geometry with theme-specific architecture. Equipment diagrams represent role props already present in the prototype and do not contain extracted anime art. No new font or runtime image was introduced.

Revision 6 applies the owner’s font selection to both DOM text and generated building signs. Canvas signs wait for the theme fonts before drawing. Orbitron adds a 38.6 kB bundled variable font. No font service is contacted at runtime.

Revision 7 reuses the original window artwork and character geometry for compact house menus, a persistent Chief marker, and a static ground ring. The [official Season 2 shadow visuals](https://sololeveling-anime.net/special/shadows-visual/) informed the owner-requested naming roster. Names are display aliases. No official character images, models, or costumes were downloaded or included.

Revision 9 replaces block-shaped character construction with rounded procedural forms and original face and silhouette details. Named-shadow cues are interpretations of the owner’s existing references. No official character model, costume texture, sprite, or image was extracted. Static geometry merges use Three.js BufferGeometryUtils from the existing dependency. SVG portraits remain original code. The character-sheet evidence is a temporary browser render of the actual models, separate from the product UI.
