# Pully UI reference

The Android card-action sheets follow the native sheet implementation in
[`OctavianTocan/pulls.review`](https://github.com/OctavianTocan/pulls.review/tree/ee5cf358333cb1170f1923c7118e12af3f265b74),
pinned to `ee5cf358333cb1170f1923c7118e12af3f265b74` for this port.

## Pages inside a sheet

Pully's [design rules 78 and 115](https://github.com/OctavianTocan/pulls.review/blob/ee5cf358333cb1170f1923c7118e12af3f265b74/android/DESIGN.md)
require a child destination to be a page of the existing sheet. The parent never
closes and reopens. Its cross becomes Back; both that button and Android Back
return one page. Back dismisses the sheet only from its root. The fixed header
and grabber remain outside the page transition, and each page retains its scroll.

The implementation comes from
[`PagedSheetPanel` and `SheetPage`](https://github.com/OctavianTocan/pulls.review/blob/ee5cf358333cb1170f1923c7118e12af3f265b74/android/app/src/main/kotlin/com/octaviantocan/pulls/ui/design/Sheet.kt)
and [`BlurSwitch`](https://github.com/OctavianTocan/pulls.review/blob/ee5cf358333cb1170f1923c7118e12af3f265b74/android/app/src/main/kotlin/com/octaviantocan/pulls/ui/design/Motion.kt).
The body travels 34% of its width, entering in 350ms and leaving in 250ms through
a 3dp blur and Pully's smooth-out curve. The heading changes in place through its
shorter 8dp blur-slide. Forward enters from the right; Back enters from the left
(rule 105). Disabling system animations removes these transitions.

`PullsPagedSheet` explicitly lets a panel at its normal detent grow or shrink to the
new page's content height during that transition. This does not dismiss or recreate
the sheet. At the tall detent the grabber and header keep their height. The card
sheet preserves this distinction rather than freezing a short page's natural height.

The imported obligations live beside the implementation in
[`ui/menu/CONTRACTS`](../android/app/src/main/kotlin/com/octaviantocan/mymind/ui/menu/CONTRACTS).
`CardMenuPresentationTest` exercises the actual dialog, the root cross, child Back,
Android Back, one shared panel, and independent retained page scroll positions.
Card reading continues to use its full-screen reader. The action panel keeps the selected card's accent as a faint wash at its top
and in the grabber. Its paper, raised cards, controls and primary actions stay
neutral, following Octavian's later accent correction.

Creating the first space is a child of the space selector in the same panel. Header
Back and Android Back return to the selector; creation failures keep the form
available for retry. The independent Spaces-tab creation dialog is a separate entry.

## One floating bottom bar

The bar is ported from Pully's
[`FootBar.kt`](https://github.com/OctavianTocan/pulls.review/blob/ee5cf358333cb1170f1923c7118e12af3f265b74/android/app/src/main/kotlin/com/octaviantocan/pulls/ui/design/FootBar.kt),
with its registration lifecycle, one shell-owned `FootBarLayer`, retained outgoing
contents, animated pill width, blur fade, compact focus handling, and native
haptics. The local `FootBarKit` supplies this app's palette and fonts while keeping
Pully's geometry, press treatment, and two-layer downward canvas shadow.

Design rules 192 and 196 require one placement and one drawing layer. The bar sits
16dp from the sides and 24dp above system navigation, and follows the keyboard.
Rule 198 makes the whole pill, including its padding, own its taps. Rule 67 makes
the selected segment slide. These obligations and their regression tests are
recorded in [`ui/shell/CONTRACTS`](../android/app/src/main/kotlin/com/octaviantocan/mymind/ui/shell/CONTRACTS).

Everything and Spaces are two icon tabs in a small navigation pill centred with
Search's separate circle, the chosen place's name showing for a moment after a change
(version 03 of the Everything bar, Octavian's pick on 1 November 2026); there is no
Create tab. Search replaces that arrangement through the same bar layer. Rediscover
and account settings remain available in the gallery header.

## Gallery behavior and verification

The gallery keeps server order while packing mixed image and text cards into
variable-width rows with one to four cards per row. During a pinch, two adjacent cached
grids project rigidly around the same focal card and crossfade only during a short
handoff. Cards do not fly independently through occupied rows. The focal card is drawn
once at its real size; the other projection exposes no duplicate actions. A gesture
can cross several stops and reverse; release settles at the nearest stop. The touched
card stays at the gesture's height within scroll limits. Projected outer edges may
clip or leave temporary horizontal slack while pinching; canonical stops fill the width.
Stable card keys and explicit image request sizes retain previews between pointer frames;
creating a new projection may reuse a cached painter once.
Pinned and the account header do not scale. Accessible larger/smaller actions expose the
same density settings. These are recorded in
[`ui/library/CONTRACTS`](../android/app/src/main/kotlin/com/octaviantocan/mymind/ui/library/CONTRACTS).

Notes, social posts, articles, and saved links that contain text and a picture reserve separate
areas for both at every density. Compact tiles prioritize the excerpt over the type
label. Photos and videos with authored notes also retain their text. Text-only notes
and image-only cards keep their own presentation.

The app uses Google Sans Flex, the typeface of Google's own phone apps such as Google
Photos, for all text, including the reader, settings, and Material components. The
bundled file is a Latin subset with a variable weight axis
(`android/app/src/main/assets/font-licenses/`). Octavian's recorded review of 22 October
2026 replaced the system font: "just not default Android font". Gallery excerpts keep
their smaller preview sizes independently of the reader.

Native Compose tests cover pointer gestures, card targeting, pagination, sheet
navigation, bar ownership, and retained gallery state. Dark and light Roborazzi
renders use local sample images. These checks do not measure frame pacing on a
physical phone.

## Sheet composition and reader chrome

The styling port was checked against current Pully revision
`f651a784cfca9b3730c60116832b89bbe236c9c3`, where the kit now lives under
`mobile/shared-ui/src/commonMain/kotlin/com/octaviantocan/pulls/ui/design/`.
`SheetHeader` centers the title between equal 44dp slots. `SheetHeading` supplies
small muted, non-folding section labels; `SheetCard` and `QuickAction` share 16dp
corners and the note shadow. The shadow has two canvas layers: a tight 1dp-down
edge and a 2dp-down soft drop (12dp blur at 10% ink for cards, 10dp at 8% for
controls), with Pully's 2.7 dark-mode strength. DESIGN rules 45, 70, 74 and 141
remain the source. The card accent is limited to a faint 64dp wash fading down from the top and the
grabber; paper, raised cards, controls and primary actions remain neutral.

The root mixes quick tiles for source/copy/download/pinning, organizing rows that
lead to child pages, optional discovery actions, and a separate destructive group.
Tag and space pages keep the same surface hierarchy and plain section labels.
All page motion, detents, Back behavior and retained scroll obligations above still
apply. Native renders use the shipped Phosphor assets, with license and pinned
source mappings in `android/app/src/main/assets/icon-licenses/`.

The full-screen reader follows Octavian's separate toolbar reference: one centered
title pill between the cross and vertical overflow button. This single-row title
may ellipsize while its complete accessible text remains available; the reader
body does not repeat it. Sheet titles remain plain. The gallery profile controls
stay fixed above the scrolling pinned rail and mosaic, and the rail has no visible
heading. Only the selected Everything/Spaces tab shows its Phosphor icon.

## Changelog

The changelog copies `WhatsNewSheet`, `ChangelogEntries`, `ChangelogItem` and
`NewTag` from `android/app/src/main/kotlin/com/octaviantocan/pulls/ui/update/UpdateUi.kt`
at `20e889540b7c05ed9174b038ae366191932924e9`. Settings and the after-install notice
share the same What's new sheet and existing Pully drag/window implementation.
Versions use unboxed blocks, a small version and newest badge, bold summaries,
optional groups, 12dp-spaced accent-dot bullets, 24dp/28dp version separation,
and rounded illustrations under the corresponding bullet, in the app's font. The
flat notes consumed by older updaters and the release publisher stay unchanged;
summary/group/image metadata enriches the bundled presentation. The obligations live in `ui/update/CONTRACTS`.
