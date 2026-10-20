# Ambient keyboard focus regression

The owner reported that TextEdit could not receive typing while the city overlay was displayed. Menu and global-shortcut recovery had passed manually on the preceding build.

The native mode transition was exercised through Settings → Ambient overlay using the computer-use tool. Temporary main-thread instrumentation inspected the actual AppKit window and application. Before the fix, the city remained active and key after the window became non-focusable. The immediate mouse flag was also still false because the framework queues that update.

The fix uses a separate macOS presentation path. It releases key and main window status, enables mouse pass-through, and shows the window without calling `makeKeyAndOrderFront`. When Agent City is active, AppKit hides it to hand control to the next app. An observer restores the overlay without activation after `NSApplicationDidHideNotification`. Restoring it immediately proved too early because hiding finishes asynchronously. See Apple's [hide documentation](https://developer.apple.com/documentation/appkit/nsapplication/hide(_:)) and [unhideWithoutActivation documentation](https://developer.apple.com/documentation/appkit/nsapplication/unhidewithoutactivation()).

`check-native-observation.py` checks captured native flags from the same transition. It does not type through the physical keyboard and cannot replace the owner's final input check. The computer-use tool can deliver text directly to an application's control, which bypasses normal keyboard focus. There is no automated physical-input regression test at that boundary in this environment.

The settled native check passed after the fix: `active=false`, `key=false`, `can_key=false`, `ignores_mouse=true`, and `visible=true`. A direct native screenshot also showed the 420 × 280 city overlay. The foreground process differed from Agent City. The owner subsequently confirmed that typing in TextEdit works with the corrected small overlay open. Direct clicks through city pixels have not been separately confirmed.

The diagnostic probes were removed from the packaged source. The raw before and after observations are retained here as evidence. Run `python3 docs/evidence/ambient-focus-2026-10-18/check-native-observation.py docs/evidence/ambient-focus-2026-10-18/after.txt` to check the recorded result. The same checker fails on `before.txt`.
