package com.octaviantocan.mymind.ui.library

import android.content.ClipData
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.platform.ClipEntry
import androidx.compose.ui.platform.LocalClipboard
import com.octaviantocan.mymind.data.Track
import com.octaviantocan.mymind.ui.menu.spaceAgentLink
import kotlinx.coroutines.launch
import com.octaviantocan.mymind.ui.theme.themeBackground
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.Image
import androidx.compose.foundation.border
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.semantics.Role
import com.octaviantocan.mymind.ui.menu.CardMenuGroup
import com.octaviantocan.mymind.ui.menu.HoldToDeleteRow
import com.octaviantocan.mymind.ui.menu.PullySheet
import com.octaviantocan.mymind.ui.menu.SheetRow
import com.octaviantocan.mymind.ui.menu.SheetRowDivider
import com.octaviantocan.mymind.ui.menu.RenameSpaceContent
import com.octaviantocan.mymind.ui.settings.glassSource
import com.octaviantocan.mymind.ui.settings.rememberGlassBackdrop
import androidx.compose.ui.semantics.clearAndSetSemantics
import com.octaviantocan.mymind.ui.shell.FootCircleButton
import com.octaviantocan.mymind.ui.theme.LibraryColors
import com.octaviantocan.mymind.ui.theme.LibraryStyle
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.draw.scale
import com.octaviantocan.mymind.R
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.layout.Layout
import androidx.compose.ui.unit.Constraints
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.octaviantocan.mymind.data.Card
import com.octaviantocan.mymind.data.Space
import com.octaviantocan.mymind.ui.errors.WebErrorScreen
import com.octaviantocan.mymind.ui.theme.MymindTheme

/** The colours a new space can take, in the order the web's new space modal offered them. */
val SPACE_COLORS = listOf(
    "#75ffc0", "#c0c2ce", "#ff8fa4", "#000", "#19aad1", "#17c37b", "#ffdcd0", "#166ff4",
    "#cdff06", "#f96", "#bfb5d7", "#96cbd1", "#cba0aa", "#fdf06f", "#7a30cf", "#ef3e4a",
)

/** Height of the title row of the Spaces list and of an opened space. */
val VIEW_HEADER_HEIGHT = GALLERY_CONTROLS_HEIGHT

private val SPACE_NAME_INSET = 50.dp
private val SPACE_BACK_INSET = 72.dp
private val SPACE_MENU_INSET = 68.dp
private val STACK_SHADOW_LIGHT = Color(0xFF3C465A)
private const val STACK_SCALE = 0.9f

/** Width over height of a space's cell at one column: its stack about the width of the screen, its name under it. */
private const val SPACE_ONE_COLUMN_RATIO = 0.9f

/** The size a cover with words is laid out at before it is drawn scaled to its stack, so its words never rewrap. */
private val COVER_FRAME = 160.dp

/**
 * The Spaces tab on the gallery's own paper: the list of spaces, or one space after the user opened it.
 *
 * @param viewModel Library behind the tab.
 * @param openSpace Space on show, or null for the list.
 * @param onOpenSpace Called with a tapped space, or null to go back to the list.
 * @param onOpenCard Called with a card tapped inside a space and that space's id.
 * @param onDeleteSpace Called with a space when "Delete space" is picked in its options.
 * @param onAddToSpace Called with a space when "Add to this space" is picked in its options.
 * @param modifier Modifier for the page.
 * @param onRenameSpace Called with a space and its trimmed new name when a rename is saved in its options.
 * @param onLongPressCard Called with a card long-pressed inside a space and that space's id.
 * @param onNewSpace Called when the New space tile at the end of the list is tapped.
 * @param controls The gallery's fixed controls, drawn over the list of spaces.
 */
@Composable
fun SpacesScreen(
    viewModel: LibraryViewModel,
    openSpace: Space?,
    onOpenSpace: (Space?) -> Unit,
    onOpenCard: (Card, String) -> Unit,
    onDeleteSpace: (Space) -> Unit,
    onAddToSpace: (Space) -> Unit,
    modifier: Modifier = Modifier,
    onRenameSpace: (Space, String) -> Unit = { _, _ -> },
    onLongPressCard: ((Card, String) -> Unit)? = null,
    onNewSpace: () -> Unit = {},
    controls: @Composable BoxScope.() -> Unit = {},
) {
    val spaces by viewModel.spaces.collectAsStateWithLifecycle()
    // The list keeps its size and place while a space is open.
    val listMosaic = rememberMosaicState()
    val listScroll = rememberLazyListState()
    LaunchedEffect(Unit) { if (spaces.spaces.isEmpty()) viewModel.loadSpaces() }
    BackHandler(enabled = openSpace != null) { onOpenSpace(null) }
    if (openSpace == null) {
        SpaceListContent(
            spaces,
            onOpenSpace = onOpenSpace,
            onRetry = viewModel::loadSpaces,
            modifier = modifier,
            onRefresh = viewModel::refreshSpaces,
            controls = controls,
            mosaic = listMosaic,
            scroll = listScroll,
            onNewSpace = onNewSpace,
            options = SpaceActions(onAddToSpace, onRenameSpace, onDeleteSpace),
        )
    } else {
        val feed = viewModel.spaceFeed(openSpace.id)
        val state by feed.state.collectAsStateWithLifecycle()
        SpaceContent(
            openSpace,
            state,
            onBack = { onOpenSpace(null) },
            onLoadMore = feed::loadMore,
            onOpenCard = { onOpenCard(it, openSpace.id) },
            onAdd = { onAddToSpace(openSpace) },
            onDelete = { onDeleteSpace(openSpace) },
            onRename = { onRenameSpace(openSpace, it) },
            modifier = modifier,
            onRefresh = { viewModel.refreshSpaceFeed(openSpace.id) },
            onLongPressCard = onLongPressCard?.let { press -> { card: Card -> press(card, openSpace.id) } },
        )
    }
}

/**
 * The list of spaces on the gallery paper: a title that scrolls away under the fixed controls, then card stacks with
 * each space's name, two across until a pinch changes their size as it does in the gallery, then a New space tile.
 *
 * @param state Spaces and their first cards.
 * @param onOpenSpace Called with a tapped space.
 * @param onRetry Called to read the spaces again after a failure.
 * @param modifier Modifier for the page.
 * @param onRefresh Called when the list is pulled down to refresh.
 * @param controls The gallery's fixed controls, drawn over the list.
 * @param mosaic The list's size, kept by the caller so it survives opening a space.
 * @param scroll The list's scroll position, kept for the same reason.
 * @param onNewSpace Called when the New space tile after the spaces is tapped.
 * @param options What a held space's options sheet does, or null to leave holds alone.
 */
@Composable
fun SpaceListContent(
    state: SpacesState,
    onOpenSpace: (Space) -> Unit,
    onRetry: () -> Unit,
    modifier: Modifier = Modifier,
    onRefresh: () -> Unit = {},
    controls: @Composable BoxScope.() -> Unit = {},
    mosaic: MosaicState = rememberMosaicState(),
    scroll: LazyListState = rememberLazyListState(),
    onNewSpace: () -> Unit = {},
    options: SpaceActions? = null,
) {
    val backdrop = rememberGlassBackdrop(rememberUpdatedState(1f))
    // The held space by id, read from the list so a deleted one closes its sheet.
    var heldId by remember { mutableStateOf<String?>(null) }
    Box(modifier.fillMaxSize().themeBackground(LibraryStyle.colors.background, backdrop = false)) {
        when {
            state.spaces.isEmpty() && state.error != null -> WebErrorScreen(onRetry = onRetry)
            state.spaces.isEmpty() && !state.loaded -> PageSpinner()
            else -> {
                PullRefresh(state.refreshing, onRefresh, Modifier.fillMaxSize().glassSource(backdrop), top = VIEW_HEADER_HEIGHT) {
                    // The New space tile follows the spaces, so making one is always at the end of the list.
                    val cards = remember(state.spaces) { state.spaces.map(::spaceCard) + NEW_SPACE_CARD }
                    val tiles = remember(state.spaces, state.covers) { SpaceMosaicTiles(state.spaces.associateBy { it.id }, state.covers) }
                    LibraryMosaic(FeedState(cards, endReached = true), onLoadMore = {},
                        onOpenCard = { card ->
                            if (card.id == NEW_SPACE_CARD.id) onNewSpace() else state.spaces.firstOrNull { it.id == card.id }?.let(onOpenSpace)
                        },
                        state = scroll, mosaic = mosaic, header = { GalleryTitle("All Spaces") }, tiles = tiles, label = "All spaces",
                        // Holding a space opens the same options as its opened page's dot menu (Tavi, 5 November 2026).
                        onLongPressCard = options?.let { { card -> if (card.id != NEW_SPACE_CARD.id) heldId = card.id } })
                }
                GalleryScrollEdge(backdrop, LibraryStyle.colors.background, Modifier.align(Alignment.TopCenter), visible = { scroll.scrolledBy() })
            }
        }
        controls()
    }
    val held = state.spaces.firstOrNull { it.id == heldId }
    if (options != null && held != null) {
        SpaceOptionsSheet(held, onClose = { heldId = null }, onAdd = { options.add(held) }, onRename = { options.rename(held, it) },
            onDelete = { options.delete(held) })
    }
}

/**
 * What a space's options sheet does with the space it was opened for.
 *
 * @property add Called with the space when "Add to this space" is picked.
 * @property rename Called with the space and its trimmed new name when a rename is saved.
 * @property delete Called with the space once "Delete space" has been held.
 */
class SpaceActions(
    val add: (Space) -> Unit,
    val rename: (Space, String) -> Unit,
    val delete: (Space) -> Unit,
)

/** The card that stands for the New space tile after the spaces; no space has this id. */
private val NEW_SPACE_CARD = Card(id = "new-space-tile", type = "Space", flags = 0, created = "", modified = "", bumped = "", title = "New space")

/** The card that stands for [space] in the mosaic: only its id and name, which the mosaic keys and lays it out by. */
private fun spaceCard(space: Space): Card =
    Card(id = space.id, type = "Space", flags = 0, created = "", modified = "", bumped = "", title = space.name)

/**
 * Draws the Spaces list in the gallery's mosaic: every space one square, a stack measured to its cell so its name keeps
 * its size, and nothing in the blurred fill, since paper already shows around each stack.
 */
private class SpaceMosaicTiles(private val spaces: Map<String, Space>, private val covers: Map<String, Card>) : MosaicTiles {
    override fun span(card: Card): MosaicSpan = MosaicSpan(1, 1)

    override fun aspectRatio(card: Card): Float = SPACE_ONE_COLUMN_RATIO

    override fun isPicture(card: Card): Boolean = true

    override val corner: Dp = 0.dp

    @Composable
    override fun Tile(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, enabled: Boolean, imageRequestSizePx: Int, modifier: Modifier) {
        if (card.id == NEW_SPACE_CARD.id) return NewSpaceStack(onClick, modifier, enabled)
        val space = spaces[card.id] ?: return
        SpaceStack(space, covers[card.id], onClick, modifier, enabled, imageRequestSizePx, onLongClick)
    }

    /** The Spaces list has no reels; a page would show the space's stack. */
    @Composable
    override fun ReelPage(card: Card, onClick: () -> Unit, onLongClick: (() -> Unit)?, imageRequestSizePx: Int, modifier: Modifier) =
        Tile(card, onClick, onLongClick, enabled = true, imageRequestSizePx = imageRequestSizePx, modifier = modifier)

    @Composable
    override fun Copy(card: Card, imageRequestSizePx: Int) {
        if (card.id == NEW_SPACE_CARD.id) return NewSpaceStack({}, Modifier.fillMaxSize(), enabled = false)
        val space = spaces[card.id] ?: return
        SpaceStack(space, covers[card.id], {}, Modifier.fillMaxSize(), enabled = false, imageRequestSizePx = imageRequestSizePx)
    }

    override fun backdrop(card: Card, colors: LibraryColors): MosaicBackdrop? = null
}

/**
 * An opened space in the gallery's photo-style mosaic, with floating Back and options controls over a title that
 * scrolls with its cards.
 *
 * @param space The space.
 * @param feed Its cards.
 * @param onBack Called when Back is tapped.
 * @param onLoadMore Called when the mosaic nears its end.
 * @param onOpenCard Called with a tapped card.
 * @param onAdd Called when "Add to this space" is picked in the options.
 * @param modifier Modifier for the page.
 * @param onDelete Called when "Delete space" is picked in the options; null leaves it out.
 * @param onRename Called with the trimmed new name when a rename is saved in the options; null leaves it out.
 * @param onRefresh Called when the mosaic is pulled down to refresh.
 * @param onLongPressCard Called with a long-pressed card, or null to ignore long presses.
 */
@Composable
fun SpaceContent(
    space: Space,
    feed: FeedState,
    onBack: () -> Unit,
    onLoadMore: () -> Unit,
    onOpenCard: (Card) -> Unit,
    onAdd: () -> Unit,
    modifier: Modifier = Modifier,
    onDelete: (() -> Unit)? = null,
    onRename: ((String) -> Unit)? = null,
    onRefresh: () -> Unit = {},
    onLongPressCard: ((Card) -> Unit)? = null,
) {
    val mosaic = rememberMosaicState(reels = true)
    val scroll = rememberLazyListState()
    val backdrop = rememberGlassBackdrop(rememberUpdatedState(1f))
    Box(modifier.fillMaxSize().themeBackground(LibraryStyle.colors.background, backdrop = false)) {
        if (feed.empty) {
            PageSpinner()
        } else {
            PullRefresh(feed.refreshing, onRefresh, Modifier.fillMaxSize().glassSource(backdrop), top = VIEW_HEADER_HEIGHT) {
                LibraryMosaic(feed, onLoadMore, onOpenCard, state = scroll, mosaic = mosaic, onLongPressCard = onLongPressCard, header = {
                    GalleryTitle(space.name.orEmpty(), start = SPACE_BACK_INSET, end = SPACE_MENU_INSET)
                })
            }
            GalleryScrollEdge(backdrop, LibraryStyle.colors.background, Modifier.align(Alignment.TopCenter).graphicsLayer { alpha = 1f - mosaic.reel },
                visible = { scroll.scrolledBy() })
        }
        FootCircleButton(R.drawable.ic_sheet_back, "Back", onBack,
            Modifier.align(Alignment.TopStart).awayInReels(mosaic).padding(start = 16.dp, top = 12.dp), size = 44.dp)
        SpaceOptions(space, onAdd, onRename, onDelete, Modifier.align(Alignment.TopEnd).awayInReels(mosaic).padding(end = 16.dp, top = 12.dp))
    }
}

/** The opened space's dot menu: a button that opens [SpaceOptionsSheet], with the long-press haptic the web gives the menu. */
@Composable
private fun SpaceOptions(space: Space, onAdd: () -> Unit, onRename: ((String) -> Unit)?, onDelete: (() -> Unit)?, modifier: Modifier = Modifier) {
    val haptic = LocalHapticFeedback.current
    var open by remember { mutableStateOf(false) }
    FootCircleButton(R.drawable.ic_sheet_more, "Space options", {
        haptic.performHapticFeedback(HapticFeedbackType.LongPress)
        open = true
    }, modifier, size = 44.dp)
    if (open) SpaceOptionsSheet(space, onClose = { open = false }, onAdd, onRename, onDelete)
}

/**
 * A space's options as a Pully sheet under its name: adding to it, copying its link for AI agents, renaming it on a
 * page of the same sheet, and deleting it with a hold, as a card is deleted. The opened space's dot menu and a hold on the Spaces list open it.
 *
 * @param space The space.
 * @param onClose Called when the sheet closes, including after a pick.
 * @param onAdd Called when "Add to this space" is picked.
 * @param onRename Called with the trimmed new name when Save name is pressed; null leaves Rename space out.
 * @param onDelete Called once "Delete space" has been held; null leaves it out.
 */
@Composable
internal fun SpaceOptionsSheet(space: Space, onClose: () -> Unit, onAdd: () -> Unit, onRename: ((String) -> Unit)?, onDelete: (() -> Unit)?) {
    val name = space.name.orEmpty()
    var renaming by remember { mutableStateOf(false) }
    val clipboard = LocalClipboard.current
    val scope = rememberCoroutineScope()
    PullySheet(if (renaming) "Rename space" else name.ifBlank { "Space" }, onClose = onClose) {
        if (renaming && onRename != null) {
            RenameSpaceContent(name, onRename = { onClose(); onRename(it) })
            return@PullySheet
        }
        CardMenuGroup(Modifier.padding(top = 20.dp)) {
            SheetRow(painterResource(R.drawable.ic_library_add), "Add to this space", { onClose(); onAdd() },
                detail = "Photos, files or the camera")
            SheetRowDivider()
            SheetRow(painterResource(R.drawable.ic_library_search_spark), "Link for AI agents", {
                onClose()
                Track.event("share", detail = "space agent link")
                scope.launch { clipboard.setClipEntry(ClipEntry(ClipData.newPlainText("Inventory space link", spaceAgentLink(space)))) }
            }, detail = "Copies a link your connected AI can open")
            if (onRename != null) {
                SheetRowDivider()
                SheetRow(painterResource(R.drawable.ic_text), "Rename space", { renaming = true })
            }
        }
        if (onDelete != null) CardMenuGroup(Modifier.padding(top = 20.dp)) {
            HoldToDeleteRow(onDelete = { onClose(); onDelete() }, label = "Delete space", detail = "Its cards stay in your inventory")
        }
    }
}

/**
 * The tile after the spaces that makes a new one: a stack's square outlined on the page with a plus in it, and "New
 * space" where a space shows its name (Tavi, 5 November 2026).
 *
 * @param onClick Called when the tile is tapped.
 * @param modifier Modifier for the cell.
 * @param enabled False for a drawing that takes no touches.
 */
@Composable
private fun NewSpaceStack(onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val colors = LibraryStyle.colors
    val shape = RoundedCornerShape(TILE_RADIUS)
    val press = if (enabled) Modifier.clickable(role = Role.Button, onClickLabel = "New space", onClick = onClick) else Modifier
    Column(modifier.then(press), horizontalAlignment = Alignment.CenterHorizontally) {
        BoxWithConstraints(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
            Box(Modifier.size(minOf(maxWidth, maxHeight)).scale(STACK_SCALE).clip(shape)
                .background(colors.muted.copy(alpha = 0.08f)).border(1.5.dp, colors.muted.copy(alpha = 0.35f), shape),
                contentAlignment = Alignment.Center) {
                Box(Modifier.size(56.dp).clip(CircleShape).background(colors.raised), contentAlignment = Alignment.Center) {
                    Image(painterResource(R.drawable.ic_library_add), null, Modifier.size(26.dp), colorFilter = ColorFilter.tint(colors.text))
                }
            }
        }
        Text("New space", Modifier.padding(vertical = 10.dp), style = LibraryStyle.body, color = colors.text, maxLines = 1)
    }
}

/**
 * One space (`mymind-stack`) filling its cell: its first card over three tilted blank cards, as large as the cell leaves
 * room for, then the space's ring and name.
 *
 * @param space The space.
 * @param cover Its first card, or null for an empty stack.
 * @param onClick Called when the stack is tapped.
 * @param modifier Modifier for the cell.
 * @param enabled False for a drawing that takes no touches.
 * @param imageRequestSizePx Size, in pixels, to decode the cover's picture at, or null for the stack's own size.
 * @param onLongClick Called when the stack is held, or null when holding does nothing.
 */
@Composable
private fun SpaceStack(
    space: Space,
    cover: Card?,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    imageRequestSizePx: Int? = null,
    onLongClick: (() -> Unit)? = null,
) {
    val colors = LibraryStyle.colors
    val shape = RoundedCornerShape(TILE_RADIUS)
    val shadows = stackShadows(MymindTheme.colors.dark)
    val angles = remember(space.id) { stackAngles(space.id) }
    val press = if (enabled) {
        Modifier.combinedClickable(interactionSource = null, indication = null, onLongClickLabel = "Space options", onLongClick = onLongClick,
            onClick = onClick)
    } else {
        Modifier
    }
    BoxWithConstraints(modifier.then(press)) {
        val nameWidth = maxWidth - minOf(SPACE_NAME_INSET, maxWidth * 0.3f)
        Column(Modifier.fillMaxSize(), horizontalAlignment = Alignment.CenterHorizontally) {
            BoxWithConstraints(Modifier.weight(1f).fillMaxWidth(), contentAlignment = Alignment.Center) {
                Box(Modifier.size(minOf(maxWidth, maxHeight)).scale(STACK_SCALE)) {
                    angles.asReversed().forEach { angle ->
                        Box(Modifier.matchParentSize().rotate(angle).stackShadow(shape, shadows).background(colors.raised, shape))
                    }
                    Box(Modifier.fillMaxSize().stackShadow(shape, shadows).clip(shape).background(colors.raised)) {
                        if (cover != null) StackCover(cover, imageRequestSizePx)
                    }
                }
            }
            Row(Modifier.padding(vertical = 10.dp), verticalAlignment = Alignment.CenterVertically) {
                SpaceRing(parseHexColor(space.color) ?: colors.muted)
                Spacer(Modifier.width(10.dp))
                Text(
                    space.name.orEmpty(),
                    Modifier.widthIn(max = nameWidth),
                    style = LibraryStyle.body,
                    color = colors.text,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}

/**
 * A space's first card on top of its stack. A picture is cut to the stack at its size; a cover with words is laid out
 * once at [COVER_FRAME] and drawn scaled, so its words keep their lines while a pinch resizes the stack.
 */
@Composable
private fun StackCover(cover: Card, imageRequestSizePx: Int?) {
    val tile = @Composable { GalleryTile(cover, onClick = {}, modifier = Modifier.clearAndSetSemantics {}, enabled = false,
        imageRequestSizePx = imageRequestSizePx, playVideo = false) }
    if (galleryTileIsPicture(cover)) return Box(Modifier.fillMaxSize()) { tile() }
    Layout(content = tile, modifier = Modifier.fillMaxSize()) { measurables, constraints ->
        val frame = COVER_FRAME.roundToPx()
        val laid = measurables.single().measure(Constraints.fixed(frame, frame))
        layout(constraints.maxWidth, constraints.maxHeight) {
            laid.placeWithLayer(0, 0) {
                transformOrigin = TransformOrigin(0f, 0f)
                scaleX = constraints.maxWidth / frame.toFloat()
                scaleY = constraints.maxHeight / frame.toFloat()
            }
        }
    }
}

/** Turns of the three blank cards under a space's first card, seeded by the space id as the web page does. */
private fun stackAngles(id: String): List<Float> {
    var angle = 0
    return (0 until 3).map { i ->
        val step = (seededUnit("${id}d$i") * 10).toInt() + 1
        angle += if (i % 2 == 0) step else -step
        angle.toFloat()
    }
}

/** A number in [0, 1) that depends only on [seed]: FNV-1a over the UTF-16 units, then two mixing rounds. */
private fun seededUnit(seed: String): Double {
    var h = -2128831035
    for (unit in seed) {
        h = h xor unit.code
        h *= 16777619
    }
    h = h xor (h ushr 13)
    h *= 0x5bd1e995
    h = h xor (h ushr 15)
    return h.toUInt().toDouble() / 4294967296.0
}

/** The two layers of `--stack-shadow` in the page's appearance (`app.css` light, `dark.css` dark). */
private fun stackShadows(dark: Boolean): List<Shadow> = if (dark) {
    listOf(
        Shadow(radius = 6.dp, color = Color.Black, offset = DpOffset(0.dp, 2.dp), alpha = 0.35f),
        Shadow(radius = 30.dp, color = Color.Black, offset = DpOffset(0.dp, 12.dp), alpha = 0.45f),
    )
} else {
    listOf(
        Shadow(radius = 5.dp, color = STACK_SHADOW_LIGHT, offset = DpOffset(0.dp, 2.dp), alpha = 0.07f),
        Shadow(radius = 28.dp, color = STACK_SHADOW_LIGHT, offset = DpOffset(0.dp, 12.dp), alpha = 0.13f),
    )
}

/** Draws [shadows] behind the content, the first one on top. */
private fun Modifier.stackShadow(shape: RoundedCornerShape, shadows: List<Shadow>) =
    shadows.asReversed().fold(this) { modifier, shadow -> modifier.dropShadow(shape, shadow) }

/** The 14 dp ring in the space's colour (`mm-circle`, radius 7, stroke 3) in its 8 dp layout box. */
@Composable
private fun SpaceRing(color: Color) {
    Canvas(Modifier.size(8.dp)) {
        val stroke = 3.dp.toPx()
        drawCircle(color, radius = 7.dp.toPx() - stroke / 2, style = Stroke(stroke))
    }
}
