# Changelog

## 0.6.41 (52), 2026-11-01
The bar at the foot of Everything and Spaces is smaller: two icons in the middle, with Search beside them.

### Changed
- Everything and Spaces are now two icons in a small pill in the middle of the bar, the one you are on filled and highlighted. The corners of the screen are left to your cards.
- When you switch between Everything and Spaces, the name of the place you switched to shows beside its icon for a moment, then folds away.
- Tapping Search widens the same pill into the search field across the bar, and closing search shrinks it back to the two icons.

## 0.6.40 (51), 2026-10-30
Opening a card now has motion: the card rises as a sheet and the gallery steps back behind it.

### Changed
- Tapping a card raises it from below as a sheet with round top corners, while the gallery behind shrinks a little, dims and rounds its corners, its top edge still showing above the card.
- Back sinks the card the way it came and brings the gallery forward again.
- Pull the card down by its top, or from the top of its page, to close it: let go about a third of the way down or flick it down and it goes; let go sooner and it springs back.

## 0.6.39 (50), 2026-10-30
One column is now full screen, and each card there shows more of itself.

### Changed
- Pinching in from two columns goes straight to full screen; there is no one-column grid before it on Everything, search results or an opened space. Pinch your fingers together, or press Back, to come back to two columns. The Spaces list keeps its one column.
- Each card in full screen is a fuller card: its picture or video whole, and at the foot where it is from and when you saved it, its title, its words or description, and its tags. A note or post without a picture shows its words large. Tap to open the card, hold for its actions.

## 0.6.38 (49), 2026-10-30
Pinch past one column to see one thing at a time, full screen, and pinching runs smoother.

### Added
- Pinch past one column on Everything, search results or an opened space and the gallery goes full screen: one card fills the screen on black, a photo whole at its own shape. Swipe up or down to move one card at a time. Pinch your fingers together, or press Back, to come back to the grid.
- The bars slide out of the way as the page goes full screen, and come back as you pinch out.

### Fixed
- Pinching no longer skips frames as your fingers come down or let go, and the end of a pinch no longer snaps: the tiles stay as they are while they move, and pictures switch to their new size only once the cards have come to rest.

## 0.6.37 (48), 2026-10-29
Saved links bring more of their post with them.

### Changed
- A Reddit post you save now keeps the post's own words and its top three comments. Opening it shows them under the post in Top comments, each with who wrote it.
- A YouTube video shows its length in the corner of its tile, as other videos do.
- An X post that carries an X article shows the article as X does: its cover, its title and Article in a box under the post, two squares tall. Opening it shows the article's opening lines too, and tapping the article opens the post on X to read the rest.
- Bluesky and Mastodon links you save now become posts in the network's own colour, with the author, the words and the pictures, instead of a screenshot of the page.
- These come from the server when it saves a link, so they show for links saved after the server is updated.

## 0.6.36 (47), 2026-10-29
The gallery's tiles are laid out like the showcase's.

### Changed
- Every tile has rounder corners, the same 12 dp as the showcase, and they stay that round while you pinch.
- The cards sit a little in from the sides of the screen, in line with Top of Mind, with more room between them: 8 dp at two columns, 6 at three and four, and 16 at one.
- More cards take a bigger shape. An article, link or note with a picture is now and then shown big, two squares by two, and one with a tall picture stands two squares high, as photos already did. Words alone stay one square.

## 0.6.34 (45), 2026-10-29
PDFs show their pages.

### Changed
- A PDF's tile is now its whole first page, two squares tall, with its name and PDF under it.
- Opening a PDF shows every page, one under another, with "3 of 48" under each. The phone fetches the PDF once and draws the pages itself, so PDFs you saved before work too.
- A PDF says how many pages it has and how long it takes to read, and Read in Reader Mode sets its words at reading size (Android 15 and later).
- Open the PDF at the foot hands it to your phone's PDF viewer.

## 0.6.33 (44), 2026-10-29
Products look like pins, apps show their icon, and videos say what they are.

### Changed
- A product with a photo is now a pin, as on Pinterest: the photo with round corners, two squares tall, with the price on it in a small white pill (such as £24), and the name and brand under it.
- An app shows its icon whole in the middle of its tile, with its name and the store it is on, Google Play or the App Store, under it.
- A YouTube or Instagram video shows its title and channel under its picture. A video with your own note keeps the note instead.
- Opening a product shows its price on the photo and its brand or shop under the name. When your account has Same Vibe, it also shows up to three things from your mind that are like it, and tapping one opens it.

## 0.6.32 (43), 2026-10-29
Saved posts look like the network they came from.

### Changed
- An X post's tile is now drawn in X's own black, with the X mark at the top, the author's name and @handle under it, and the post's words. A post with a picture, or more words than one square holds, stands two squares tall, so the whole post fits and the picture gets room. Posts show no likes or reposts.
- Reddit posts take Reddit's orange, with the subreddit and u/author at the top and the post's title. Threads, Bluesky, Mastodon and Substack posts take their own colours the same way.
- Opening a post shows it as one section in the same colours: who posted it with the handle and the day, the words, the picture or video, and when it was posted and saved. It is ThyMind's own copy of the post, so it doesn't change if the original does.

## 0.6.31 (42), 2026-10-29
Sharing something into ThyMind shows what you saved and goes away by itself.

### Changed
- Sharing a link, words, photos, a video or a PDF into ThyMind now opens a small sheet over the app you shared from, instead of a full grey page. It shows what you saved, with its picture, and fills in the page's title and picture as ThyMind reads it.
- One field takes everything: type a word and the spaces and tags that match show up under it, type a sentence and use it as your note. With nothing typed it offers the space ThyMind suggests and your recent tags. Enter adds the first one, and whatever you add sits above the field with a cross to take it off.
- The sheet fades away by itself 5 seconds after saving if you don't touch it. Touching it keeps it open until you tap Done, and tapping outside closes it at once. The lighter part of Done shows the time left.
- When something can't be saved (too big, more than 20 things, a PDF with a password, signed out), the sheet says why in ThyMind's own words and fades after 8 seconds.

## 0.6.30 (41), 2026-10-28
Search your mind understands what you mean and answers questions.

### Changed
- Pressing search on the keyboard, with Claude Code or Codex signed in, now has it read what you mean first: the kind of thing, what it is about, when you saved it, and which of your tags and spaces fit. It searches for that, ranks what it finds, and shows what it looked for, with your matching tags and spaces as chips: a tag narrows the search, a space opens it.
- Ask a question, such as "when did I save that essay about walking", and it answers in a few sentences from your own cards, with the cards it used under the answer.
- If reading the search fails, it still ranks what your words found. This works once your server is updated for 0.6.30.

## 0.6.29 (40), 2026-10-28
Letting go of a pinch partway lands smoothly.

### Fixed
- Letting go of a pinch before the next size no longer pops. The cards carry on at the speed they had under your fingers, or start gently if you held still, instead of leaping off at full speed.
- The gallery no longer jumps a row, or further, just after a pinch lands on a new size.
- A landing takes a quarter of a second for a whole size and less for a short way, without snapping.

## 0.6.28 (39), 2026-10-28
An open card now does everything the chosen design does: hold any part for its own actions, write your note, read in Reader Mode, and share the picture or the whole card as an image.

### Added
- Holding a part of an open card opens a small sheet named after it, with only what that part can do. The picture offers Save picture, Copy picture and Share picture; a summary, post or note offers Copy and Share; Tags offers Edit tags and Copy tags; the page's text offers Read in Reader Mode and Copy text. The page ends with a line saying any part can be held.
- Your note shows on every card that is not itself a note, even before you write one. Holding it offers Write a note, or Edit note and Copy note, and Done saves it.
- Reader Mode shows only the page's text at reading size, under the card's title and who wrote it, and the text can be selected. A thin line in the name's pill fills as you read, and Back (or Back to the card at the end) returns to the card. It opens from Choose one and from the page text's sheet.
- Share now also offers The picture and The card as an image. The card as an image shows the picture, title and, if you want, your note and tags, and Share image sends it.

### Changed
- Reader Mode in Choose one is no longer locked; it shows for any card with page text.
- The Share button is in the reader's header for every card.
## 0.6.26 (37), 2026-10-28
Your Google photo shows properly.

### Fixed
- Your Google photo on the account button and in the Profile sheet now shows the moment the app opens, stays while you're offline, and no longer drops back to the plain icon each time you open the sheet.
- The photo is sharp: the app asks Google for one big enough for the button and the sheet instead of the small one that looked blurred.
- The phone keeps your photo from the moment you sign in with Google. If you signed in before 0.6.0, sign out and back in once so it can keep it.

## 0.6.25 (36), 2026-10-28
Search opens in the same bottom bar as Everything and Spaces.

### Changed
- Everything, Spaces and Search share one bottom bar, as in Pully. Tapping Search turns the tabs into the search field in the same place and the search button beside them into the close button, with a soft blur, instead of swapping in a different bar. Closing search turns it back.
- The search field is now the same height as the tabs, so the bar no longer jumps when search opens.

## 0.6.24 (35), 2026-10-28
A card's actions sheet is now Choose one, with its spaces inside it and a delete you can take back.

### Changed
- The card's actions sheet is called Choose one. It starts with Add to space, Same Vibe and Reader Mode, then Open source, Copy, Download, Top of Mind and Add tags as tiles two by two, then Delete card on its own. The Top of Mind tile turns dark while the card is pinned.
- Add to space sits in the sheet itself: up to three spaces as small mosaics of their cards, the space it most likely belongs in first, marked Suggested, and one tap adds or removes the card. All spaces shows every space with its mosaic and card count, and holding a space shows its latest cards.
- New space opens in place on that page, with a name and a colour, and Create and add makes the space and puts the card in it.
- Delete card is held, not tapped. It fills over a second and a bit while the page behind it shrinks, and letting go sooner keeps the card. Once it is deleted, a bar offers Undo for 5 seconds, and the card only goes to the trash after that.

## 0.6.23 (34), 2026-10-28
Pinch the Spaces list to see more or fewer spaces at once.

### Added
- Pinching the Spaces list makes the stacks larger or smaller, from one across to four, the same way the gallery does. Names stay the same size and shorten when the stacks get small.

### Changed
- The Spaces list keeps its size and where you scrolled to while you look inside a space.

## 0.6.22 (33), 2026-10-28
The Serendipity page is gone.

### Removed
- The shuffle button at the top of Everything and Spaces, and the Serendipity page it opened, where cards came one at a time to keep or forget. Only Add and your profile sit up there now.

## 0.6.21 (32), 2026-10-28
Pinching works as soon as the app opens, and on search results.

### Fixed
- Right after opening the app, pinching the gallery did nothing until you went to Spaces and back. It now works at once.
- Pinching search results now changes their size too, as it does everywhere else.

## 0.6.20 (31), 2026-10-28
An open card reads as a page of its own parts, with Share at the top and Open source at the foot.

### Changed
- An open card shows each of its parts in its own raised section: the picture with the title, who made it, where it is from, when you saved it and how long it takes to read; then its summary, tags, your note and the text from the page, each under a small heading.
- Tapping the card's name at the top grows it into a card of its details: Copy link, Top of Mind as a switch, the spaces it is in, and the last three cards you opened, so you can go back to one or to Everything.
- The top right button is now Share. It offers the link, a note's or post's words, and Copy link.
- Open source and the card's actions sit at the foot of the page, and + Add in the Tags section adds tags.

## 0.6.19 (30), 2026-10-28
A pinch no longer flickers when it lands.

### Fixed
- When a pinch lands on its new size, the soft blur behind the cards now fades away instead of vanishing in one frame, so the lines between the cards no longer flicker at the end.
- Landing a pinch on a size with fewer rows, in a short list or at the end of your library, could close the app. It no longer does.

## 0.6.18 (29), 2026-10-28
Pictures show at once, blurred, while they load.

### Changed
- Every picture in the gallery and on a card shows a blurred preview of itself the moment it appears, then sharpens when the picture arrives, and keeps the preview when the picture can't load, such as offline. Your server makes a preview for each new picture as it saves it, and for your older pictures, newest first, while you use the app. This works once your server is updated for 0.6.18.

## 0.6.17 (28), 2026-10-28
Every screen keeps up with your library by itself.

### Changed
- The gallery, Top of Mind, your spaces and an open card redraw from the library on your phone whenever it changes, whether the change came from your server, from another device or from a tap on this phone.
- A card you have open fills in as your server finishes reading it, such as a link you just shared getting its title and picture.

## 0.6.16 (27), 2026-10-28
Changes made elsewhere show up by themselves.

### Changed
- While the app is open, it hears from your server the moment something changes, such as a card saved on the web or a shared link your server has finished reading, and updates the gallery, Top of Mind and spaces without a pull to refresh.

## 0.6.15 (26), 2026-10-28
Links and text you share are saved the moment you tap, even offline.

### Changed
- Sharing a link or a piece of text to the app saves it on the phone at once: it leads your gallery straight away, offline search finds it, and the tags, note and spaces you add in the share sheet show on it too. Your server gets it in the background. This works once your server is updated for 0.6.15; until then sharing waits for the server as before.
- A link you saved before still keeps its own note when you share it again.
- The share sheet lists your spaces from the phone, so it shows them offline and includes spaces you just made.

## 0.6.14 (25), 2026-10-28
New and deleted spaces show the moment you tap.

### Changed
- Making or deleting a space happens on the phone at once and reaches your server in the background, even offline. A new space works this way once your server is updated for 0.6.14; until then it waits for the server as before.

## 0.6.13 (24), 2026-10-28
Cards move with your fingers while you pinch.

### Changed
- Pinching the gallery moves the cards from the moment your fingers start, instead of waiting and then gliding. Once your fingers are halfway to the next size, the rest happens in a quick hop.
- A card that moves to another line slides out past one edge and in from the other, fading as it goes, and lower lines start a moment later, like a wave.
- Each size now takes about a 30% pinch, at every size. Letting go lands in under a fifth of a second, the way your fingers were going once they were a quarter of the way there.

### Fixed
- No blank spaces show between the cards while you pinch: a soft, blurred copy of the gallery fills them.
- The card under your fingers stays with them while it changes shape, such as a screenshot going from two squares high to one, instead of sliding off one edge and back in from the other.

## 0.6.12 (23), 2026-10-28
Search can ask Claude Code or Codex what you mean, signed in from your Profile as in Pully.

### New
- Your Profile has an AI search section: pick Claude Code or Codex, its model and how much effort it puts in, with the same pages as Pully.
- Sign in to Claude Code or Codex from your Profile with your own Claude or ChatGPT subscription. Claude Code's code is picked up from the clipboard when you come back, and Codex finishes by itself once you approve its code.
- Pressing search on the keyboard keeps the matches on screen while the chosen engine ranks them, then shows its order, with a note under the Search title. Typing still answers at once without it.

## 0.6.11 (22), 2026-10-28
Your changes reach the server in one go, and never twice.

### Changed
- Changes waiting on the phone go to your server together in one request, each under its own number, so a change sent again after a dropped connection is not applied twice. A server from before this version gets them one by one, as before.

## 0.6.10 (21), 2026-10-28
Notes stop shifting while you pinch.

### Changed
- A note, post or other tile with words is now a picture of itself: it keeps the same lines at every size and simply grows or shrinks with its square, like a photo. At four columns its words are smaller than before and at two or one column larger.

### Fixed
- While you pinch the gallery, tiles with words no longer rewrap their lines at every step, so the words stop jumping around.

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
