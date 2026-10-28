# Changelog

## 0.6.10 (21), 2026-10-28
Your changes reach the server in one go, and never twice.

### Changed
- Changes waiting on the phone go to your server together in one request, each under its own number, so a change sent again after a dropped connection is not applied twice. A server from before this version gets them one by one, as before.

## 0.6.9 (20), 2026-10-28
Pins, notes, tags and space changes happen the moment you tap, even online.

### Changed
- Pinning, trashing, writing a note, adding or removing tags and moving a card in or out of a space show at once and reach your server in the background, instead of waiting for it to answer first. If the server turns a change down, the card goes back to how the server has it and a short notice says so.
- A card added to a space shows at the end of it straight away.

## 0.6.8 (19), 2026-10-28
Spaces open straight from the phone instead of waiting for the server.

### Changed
- The Spaces tab, its cover pictures, the cards inside a space and the cards Serendipity deals now come from the library kept on the phone, so they show at once instead of after a trip to the server. Space pages switch over once the server is updated.
- Pulling to refresh on the Spaces tab or inside a space brings the phone's library up to date first.
- After the server update, the phone downloads its copy of the library once more, so each card knows its place in its spaces.

## 0.6.7 (18), 2026-10-23
The gallery breaks up its squares like Google Photos, and pinching moves cards instead of swapping grids.

### Changed
- At two to four columns the gallery uses square cells with a few shapes mixed in: screenshots and other tall pictures stand two squares high, about half the wide pictures lie two across, and about one picture in seven is shown big. Which pictures get which shape stays the same each time you open the app.
- Notes, posts and pictures with words stay one square, and every row is full except at the very end.
- When a pinch passes a size, each card glides from its old place to its new one instead of the whole grid fading into another.

## 0.6.6 (17), 2026-10-23
Videos play like in Google Photos, and the gallery text and bottom bar look calmer.

### New
- Opening a video kept on your server shows a real player: it starts with sound, a tap pauses it, and below it are play, the time, sound on or off and a bar to skip through.
- Videos saved without a cover show a still from the video in the gallery and play quietly there, instead of an empty "Video" tile.

### Changed
- Every tile's text has the same size and weight, so long notes no longer shrink and no tile looks bold.
- A link tile no longer repeats its address above itself.
- The video badge is the Google Photos one: the length and a white play circle on the picture, with no YouTube logo.
- The bottom bar's highlight hugs the chosen tab, the labels are a little bolder and darker, the chosen tab has a filled icon and Search shows a sparkle.

### Fixed
- A card's picture shows whole in the card instead of cut in half.

## 0.6.5 (16), 2026-10-22
Links shared with words around them are saved as links.

### Fixed
- Sharing a link with other words, such as a place name above a map link or "Check this out" before a video, saves a link card with those words as its note, instead of a note with the link inside. A link you saved before keeps its note.

## 0.6.4 (15), 2026-10-22
Changes you make offline wait on your phone, and your newest pictures come along.

### New
- Pins, trash, tags, notes and spaces changed without internet show at once and reach the server in order when you're back online, even with the app closed.
- On Wi-Fi the app keeps the pictures of Top of Mind and your newest 1,000 cards on your phone, so they show offline. This needs the updated server, like 0.6.3.
- If the server refuses a change made offline, a short notice says which card.
- Signing out also wipes kept pictures and changes still waiting. New cards, uploads and new spaces still need a connection.

## 0.6.3 (14), 2026-10-22
Your whole library on your phone, so the app works without internet.

### New
- The app keeps a copy of all your cards on your phone and brings it up to date when you open it, pull to refresh or change something.
- Without internet the app stays open on your cards with a small Offline pill under the top buttons, instead of the full-screen notice.
- Offline, search finds your words in titles, notes, tags and saved text, and cards and spaces still open.
- It needs the updated server; until then the app works as in 0.6.2.

## 0.6.2 (13), 2026-10-22
The app opens on your cards at once, even before the server answers.

### New
- Everything and Top of Mind open on the cards you saw last time, kept on your phone, then update from the server.
- Without a connection you still see those cards. Signing out wipes them from the phone.

## 0.6.1 (12), 2026-10-22
Search that finds what you mean, not only the words you typed.

### New
- Search also brings up cards close in meaning to your words, after the words that match. It needs the updated server; until then search works as before.

## 0.6.0 (11), 2026-10-22
Your review, applied: a new typeface, videos that play, posts that look like posts, and Pully sheets for adding and your profile.

### New
- Google Sans Flex, the typeface of Google's own apps, replaces Android's default font.
- Tap + for an Add sheet with Camera, Photos and Files, and New space on the Spaces tab.
- Tap your photo for a profile sheet with your Google photo, email, updates and Sign out.
- Spaces share the gallery's background and top controls; an opened space has Back and an options menu.
- Cards open with everything they hold: picture, post, description, saved page text and your note.
- Tap a card's picture or its source pill to open the original; hold the picture for card actions.
- Videos play quietly in the gallery, with a play or YouTube mark and their length instead of the camera icon.
- Posts look like posts: their words, their own picture and who wrote them, without pic.twitter.com links.
- GitHub repositories show GitHub's preview picture.

### Fixed
- Pinching holds each size for a moment, never shows empty margins and no longer pans with two fingers.
- "Your mind" scrolls away with your cards while the buttons stay put without a bar behind them.
- The bottom bar is slimmer, wider and sits lower, like Google Photos.
- The reader leads with a back arrow, carries the card's colour at the top and keeps tags quiet until you edit them.
- Long notes and posts shrink their text a little to show more of themselves.
- Pictures from other sites no longer receive your sign-in.

## 0.5.3 (10), 2026-10-22
Release notes that read like Pully, and a softer edge above your cards.

### New
- Read What's new in a Pully bottom sheet, with version badges, release headlines and grouped changes.

### Fixed
- Cards blur gently beneath the sticky top controls as you scroll.
![Cards beneath the blurred gallery edge](changelog/0.5.3/gallery.png)

## 0.5.2 (9), 2026-10-22
Sticky library controls and clearer card actions.

- Keep gallery controls visible while scrolling, with a cleaner pinned-card rail.
- Show an icon only on the selected Everything or Spaces tab, and use Phosphor interface icons.
- Read cards beneath a centered title pill with a vertical actions menu.
- Organize card sheets with Pully section headings, quick actions and softly raised groups, with only a subtle accent at the top.

## 0.5.1 (8), 2026-10-22
System type and improved gallery resizing.

- Use Android's familiar system typeface throughout the app.
- Smaller, lighter gallery text makes card previews easier to scan while retaining their words and pictures.
- Cards follow your fingers throughout a pinch, including direction changes and several size steps.

## 0.5.0 (7), 2026-10-22
A photo-style gallery for your notes.

- Geist is the app's typeface, with text and images both visible in mixed card previews.
- Browse a photo-style brick gallery and pinch to make your cards larger or smaller.
- Find your profile, add controls and Serendipity above large Top of Mind highlights.
- Switch between Everything and Spaces in the floating bottom bar, with Search beside it.
- Card actions, tags and spaces use accented Pully sheets; child pages slide in and Back returns to the previous page.
- Sheets attach to the bottom edge, carry the card's accent and follow your drag with spring motion.

## 0.4.1 (6), 2026-10-21
Full-screen reading.

- Read cards in a full-screen view with one clear title and close and action controls.
- Card action, tag and space sheets use the accent of the card they belong to.

## 0.4.0 (5), 2026-10-21
Card sheets inspired by Pully.

- Open cards in Pully-style sheets that expand, scroll and close with a drag.
- Close and action controls sit inside the card sheet header.
- Read one clear title without a second overlapping copy.

## 0.3.1 (4), 2026-10-18
More reliable shared uploads.

- Pictures and videos no longer fail to upload on a slow connection.
- A shared file keeps uploading after you close the share sheet or leave the app, with a notification while it uploads.
- An update waits for every upload to finish before it installs.

## 0.3.0 (3), 2026-10-18
Save from Android's share menu.

- Share a link, picture, video, PDF or text from any app to save it to your library.
- Add tags, a note and spaces in the share sheet, for one item or several at once.
- Tells you when you are signed out, a file is over 25 MB, a PDF is locked or more than 20 things are shared.
- Updates install by themselves after you allow it once, when you leave the app and nothing is uploading.

## 0.2.0 (2), 2026-10-17
Your library on Android.

- Sign in with your mymind account, by Google or with a password.
- Browse Everything, Spaces and Serendipity, with pull to refresh.
- Open a card and use its menu: tags, spaces, Top of Mind, copy, download and delete.
- Add photos and videos from the + button, with an upload strip that shows progress.
- Top of Mind row on Everything, kept in your pin order.
- Downloads and installs new versions by itself, and shows what changed in each.

## 0.1.0 (1), 2026-10-16
The first Android builds.

- First debug builds.
