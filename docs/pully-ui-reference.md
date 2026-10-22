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
Card reading continues to use its full-screen reader.
